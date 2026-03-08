use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use tracing::{error, info, warn};
use uuid::Uuid;

use ajen_core::types::director::*;
use ajen_core::types::employee::EmployeeTier;
use ajen_core::types::event::{AjenEvent, EventType};

use crate::context::EngineContext;
use crate::employee::factory::{CreateEmployeeOptions, create_employee};
use crate::employee::runtime::RuntimeDeps;

pub struct Director {
    ctx: Arc<EngineContext>,
}

impl Director {
    pub fn new(ctx: Arc<EngineContext>) -> Self {
        Self { ctx }
    }

    /// Start a new company from a description. Spawns CEO planning in background.
    /// Returns the company_id immediately.
    pub async fn start_company(&self, description: String) -> Result<String> {
        let company_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let record = CompanyRecord {
            id: company_id.clone(),
            description: description.clone(),
            phase: CompanyPhase::Planning,
            plan: None,
            ceo_employee_id: None,
            employees: vec![],
            error: None,
            created_at: now,
            updated_at: now,
        };
        self.ctx.company_store.insert(record).await?;

        self.ctx.event_bus.emit(AjenEvent {
            id: Uuid::new_v4().to_string(),
            company_id: company_id.clone(),
            employee_id: None,
            event_type: EventType::CompanyCreated,
            data: Some(serde_json::json!({ "description": &description })),
            created_at: now,
        });

        // Ensure workspace directory
        let work_dir = format!("{}/{}", self.ctx.config.workspace_dir, &company_id);
        tokio::fs::create_dir_all(&work_dir).await?;

        let ctx = self.ctx.clone();
        let cid = company_id.clone();
        tokio::spawn(async move {
            if let Err(e) = run_planning(ctx, cid.clone(), description).await {
                error!(company_id = %cid, error = %e, "planning failed");
            }
        });

        Ok(company_id)
    }

    /// Approve a company plan and begin execution.
    pub async fn approve_company(&self, company_id: &str) -> Result<()> {
        let mut record = self
            .ctx
            .company_store
            .get(company_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Company not found: {}", company_id))?;

        if record.phase != CompanyPhase::PlanReady {
            anyhow::bail!(
                "Company {} is in phase {:?}, expected PlanReady",
                company_id,
                record.phase
            );
        }

        let plan = record
            .plan
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No plan found for company {}", company_id))?;

        record.phase = CompanyPhase::Approved;
        record.updated_at = Utc::now();
        self.ctx.company_store.update(record).await?;

        self.ctx.event_bus.emit(AjenEvent {
            id: Uuid::new_v4().to_string(),
            company_id: company_id.to_string(),
            employee_id: None,
            event_type: EventType::CompanyApproved,
            data: None,
            created_at: Utc::now(),
        });

        let ctx = self.ctx.clone();
        let cid = company_id.to_string();
        tokio::spawn(async move {
            if let Err(e) = run_execution(ctx, cid.clone(), plan).await {
                error!(company_id = %cid, error = %e, "execution failed");
            }
        });

        Ok(())
    }

    /// Get the current status of a company.
    pub async fn get_status(&self, company_id: &str) -> Result<CompanyStatus> {
        let record = self
            .ctx
            .company_store
            .get(company_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Company not found: {}", company_id))?;

        let usage = self
            .ctx
            .budget_tracker
            .get_usage_summary(company_id)
            .await?;

        let tasks_total: u32 = record
            .plan
            .as_ref()
            .map(|p| p.milestones.iter().map(|m| m.tasks.len() as u32).sum())
            .unwrap_or(0);

        let tasks_completed = match record.phase {
            CompanyPhase::Completed => tasks_total,
            _ => 0,
        };

        Ok(CompanyStatus {
            company_id: record.id,
            name: record
                .plan
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Planning...".to_string()),
            status: format!("{:?}", record.phase).to_lowercase(),
            employees: record.employees,
            tasks_completed,
            tasks_pending: tasks_total.saturating_sub(tasks_completed),
            total_cost_cents: usage.total_cost_cents,
        })
    }
}

// --- Background tasks ---

fn build_runtime_deps(ctx: &Arc<EngineContext>, company_id: &str) -> Arc<RuntimeDeps> {
    let work_dir = format!("{}/{}", ctx.config.workspace_dir, company_id);
    Arc::new(RuntimeDeps {
        event_bus: ctx.event_bus.clone(),
        comms_bus: ctx.comms_bus.clone(),
        memory_store: ctx.memory_store.clone(),
        budget_tracker: ctx.budget_tracker.clone(),
        conversation_store: ctx.conversation_store.clone(),
        tool_registry: ctx.tool_registry.clone(),
        provider_registry: ctx.provider_registry.clone(),
        work_dir,
    })
}

async fn run_planning(
    ctx: Arc<EngineContext>,
    company_id: String,
    description: String,
) -> Result<()> {
    let deps = build_runtime_deps(&ctx, &company_id);

    let ceo = create_employee(
        CreateEmployeeOptions {
            company_id: company_id.clone(),
            name: "CEO".to_string(),
            title: "Chief Executive Officer".to_string(),
            role: "ceo".to_string(),
            tier: Some(EmployeeTier::Executive),
            manager_id: None,
            personality: None,
            provider_override: None,
            model_override: None,
            temperature: Some(0.7),
            max_tokens: Some(4096),
            manifest_id: None,
            system_prompt: Some(CEO_PLANNING_PROMPT.to_string()),
        },
        deps,
    );

    // Update record with CEO employee id
    if let Ok(Some(mut record)) = ctx.company_store.get(&company_id).await {
        record.ceo_employee_id = Some(ceo.config.id.clone());
        record.employees.push(CompanyEmployee {
            id: ceo.config.id.clone(),
            name: "CEO".to_string(),
            role: "ceo".to_string(),
            status: "working".to_string(),
            current_task: Some("planning".to_string()),
        });
        record.updated_at = Utc::now();
        let _ = ctx.company_store.update(record).await;
    }

    ceo.initialize().await?;

    let task_id = Uuid::new_v4().to_string();
    let instruction = format!(
        "A user wants to build the following:\n\n\
        {description}\n\n\
        Analyze this request and create a detailed company plan. \
        You MUST respond with ONLY a JSON object (no markdown fences, no extra text) \
        matching this exact schema:\n\n\
        {COMPANY_PLAN_SCHEMA}\n\n\
        Choose realistic team members from these roles: {AVAILABLE_ROLES}\n\
        Break the work into logical milestones with specific tasks."
    );

    let response = ceo.execute_task(&task_id, &instruction).await?;

    // Parse plan from CEO response
    let plan = match parse_plan_from_response(&response) {
        Ok(plan) => plan,
        Err(e) => {
            if let Ok(Some(mut record)) = ctx.company_store.get(&company_id).await {
                record.phase = CompanyPhase::Failed;
                record.error = Some(format!("Failed to parse plan: {e}"));
                record.updated_at = Utc::now();
                let _ = ctx.company_store.update(record).await;
            }
            ceo.terminate().await;
            return Err(e);
        }
    };

    // Update record to PlanReady
    if let Ok(Some(mut record)) = ctx.company_store.get(&company_id).await {
        record.phase = CompanyPhase::PlanReady;
        record.plan = Some(plan);
        record.updated_at = Utc::now();
        if let Some(ce) = record.employees.iter_mut().find(|e| e.role == "ceo") {
            ce.status = "idle".to_string();
            ce.current_task = None;
        }
        let _ = ctx.company_store.update(record).await;
    }

    ctx.event_bus.emit(AjenEvent {
        id: Uuid::new_v4().to_string(),
        company_id: company_id.clone(),
        employee_id: None,
        event_type: EventType::CompanyPlanReady,
        data: None,
        created_at: Utc::now(),
    });

    ceo.terminate().await;
    info!(company_id = %company_id, "company plan ready");
    Ok(())
}

async fn run_execution(
    ctx: Arc<EngineContext>,
    company_id: String,
    plan: CompanyPlan,
) -> Result<()> {
    let deps = build_runtime_deps(&ctx, &company_id);

    // Update phase to Running
    if let Ok(Some(mut record)) = ctx.company_store.get(&company_id).await {
        record.phase = CompanyPhase::Running;
        record.updated_at = Utc::now();
        let _ = ctx.company_store.update(record).await;
    }

    // Spawn employees for each team member
    let mut employees: Vec<(TeamMember, crate::employee::runtime::EmployeeRuntime)> = Vec::new();

    for member in &plan.team {
        let employee = create_employee(
            CreateEmployeeOptions {
                company_id: company_id.clone(),
                name: member.name.clone(),
                title: member.title.clone(),
                role: member.role.clone(),
                tier: None,
                manager_id: None,
                personality: None,
                provider_override: None,
                model_override: None,
                temperature: None,
                max_tokens: None,
                manifest_id: None,
                system_prompt: None,
            },
            deps.clone(),
        );
        employee.initialize().await?;

        if let Ok(Some(mut record)) = ctx.company_store.get(&company_id).await {
            record.employees.push(CompanyEmployee {
                id: employee.config.id.clone(),
                name: member.name.clone(),
                role: member.role.clone(),
                status: "idle".to_string(),
                current_task: None,
            });
            record.updated_at = Utc::now();
            let _ = ctx.company_store.update(record).await;
        }

        employees.push((member.clone(), employee));
    }

    // Execute milestones sequentially
    let mut completed_milestones: Vec<String> = Vec::new();

    for milestone in &plan.milestones {
        for dep in &milestone.depends_on {
            if !completed_milestones.contains(dep) {
                warn!(
                    milestone = %milestone.name,
                    dependency = %dep,
                    "dependency not yet completed, proceeding anyway"
                );
            }
        }

        info!(
            milestone = %milestone.name,
            tasks = milestone.tasks.len(),
            "starting milestone"
        );

        for (i, task_description) in milestone.tasks.iter().enumerate() {
            let emp_idx = find_best_employee(task_description, &employees, i);
            let (member, employee) = &employees[emp_idx];

            let task_id = Uuid::new_v4().to_string();

            info!(employee = %member.name, task = %task_description, "assigning task");

            if let Ok(Some(mut record)) = ctx.company_store.get(&company_id).await {
                if let Some(ce) = record
                    .employees
                    .iter_mut()
                    .find(|e| e.id == employee.config.id)
                {
                    ce.status = "working".to_string();
                    ce.current_task = Some(task_description.clone());
                }
                record.updated_at = Utc::now();
                let _ = ctx.company_store.update(record).await;
            }

            let instruction = format!(
                "MILESTONE: {}\nTASK: {}\n\n\
                Please complete this task. Write any files needed to the workspace. \
                When done, summarize what you accomplished.",
                milestone.name, task_description
            );

            match employee.execute_task(&task_id, &instruction).await {
                Ok(_) => {
                    info!(task = %task_description, "task completed");
                    if let Ok(Some(mut record)) = ctx.company_store.get(&company_id).await {
                        if let Some(ce) = record
                            .employees
                            .iter_mut()
                            .find(|e| e.id == employee.config.id)
                        {
                            ce.status = "idle".to_string();
                            ce.current_task = None;
                        }
                        record.updated_at = Utc::now();
                        let _ = ctx.company_store.update(record).await;
                    }
                }
                Err(e) => {
                    error!(task = %task_description, error = %e, "task failed");
                }
            }
        }

        completed_milestones.push(milestone.name.clone());
    }

    // Mark completed
    if let Ok(Some(mut record)) = ctx.company_store.get(&company_id).await {
        record.phase = CompanyPhase::Completed;
        record.updated_at = Utc::now();
        let _ = ctx.company_store.update(record).await;
    }

    for (_, employee) in &employees {
        employee.terminate().await;
    }

    ctx.event_bus.emit(AjenEvent {
        id: Uuid::new_v4().to_string(),
        company_id: company_id.clone(),
        employee_id: None,
        event_type: EventType::CompanyDeployed,
        data: None,
        created_at: Utc::now(),
    });

    info!(company_id = %company_id, "company execution completed");
    Ok(())
}

fn find_best_employee(
    task: &str,
    employees: &[(TeamMember, crate::employee::runtime::EmployeeRuntime)],
    fallback_idx: usize,
) -> usize {
    let task_lower = task.to_lowercase();

    for (i, (member, _)) in employees.iter().enumerate() {
        let matches = match member.role.as_str() {
            "cto" => {
                task_lower.contains("architect")
                    || task_lower.contains("technical")
                    || task_lower.contains("database")
            }
            "frontend_dev" => {
                task_lower.contains("frontend")
                    || task_lower.contains("ui")
                    || task_lower.contains("react")
                    || task_lower.contains("css")
            }
            "backend_dev" => {
                task_lower.contains("backend")
                    || task_lower.contains("api")
                    || task_lower.contains("server")
            }
            "fullstack_dev" => {
                task_lower.contains("implement")
                    || task_lower.contains("build")
                    || task_lower.contains("feature")
            }
            "designer" => {
                task_lower.contains("design")
                    || task_lower.contains("mockup")
                    || task_lower.contains("layout")
            }
            "content_writer" => {
                task_lower.contains("content")
                    || task_lower.contains("copy")
                    || task_lower.contains("documentation")
            }
            "devops" => {
                task_lower.contains("deploy")
                    || task_lower.contains("docker")
                    || task_lower.contains("ci")
            }
            "qa_engineer" => {
                task_lower.contains("test")
                    || task_lower.contains("quality")
                    || task_lower.contains("bug")
            }
            "seo_specialist" => {
                task_lower.contains("seo")
                    || task_lower.contains("meta")
                    || task_lower.contains("search engine")
            }
            _ => false,
        };
        if matches {
            return i;
        }
    }

    fallback_idx % employees.len()
}

fn parse_plan_from_response(response: &str) -> Result<CompanyPlan> {
    if let Ok(plan) = serde_json::from_str::<CompanyPlan>(response) {
        return Ok(plan);
    }

    // Extract JSON from markdown or surrounding text
    let trimmed = response.trim();
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            let json_str = &trimmed[start..=end];
            if let Ok(plan) = serde_json::from_str::<CompanyPlan>(json_str) {
                return Ok(plan);
            }
        }
    }

    anyhow::bail!(
        "Failed to parse CompanyPlan from CEO response: {}",
        &response[..response.len().min(500)]
    )
}

const CEO_PLANNING_PROMPT: &str = r#"You are the CEO planning agent. Your sole job is to analyze a user's product request and produce a structured company plan as a JSON object.

You must output ONLY valid JSON, no markdown code fences, no explanatory text before or after. The JSON must match the CompanyPlan schema exactly.

For each team member, choose from these roles: cto, cmo, coo, fullstack_dev, frontend_dev, backend_dev, content_writer, designer, seo_specialist, devops, qa_engineer, social_media, data_analyst.

For tools, use: filesystem.read_file, filesystem.write_file, filesystem.list_directory.

Create realistic milestones with specific, actionable tasks. Each milestone should have a clear deliverable."#;

const COMPANY_PLAN_SCHEMA: &str = r#"{
  "name": "string",
  "description": "string",
  "product": {
    "type": "string (e.g. web_app, api, cli, mobile_app)",
    "techStack": ["string"],
    "features": ["string"]
  },
  "team": [
    {
      "role": "string (role id)",
      "title": "string",
      "name": "string",
      "responsibilities": ["string"],
      "tools": ["string"]
    }
  ],
  "milestones": [
    {
      "name": "string",
      "tasks": ["string"],
      "dependsOn": ["string"]
    }
  ],
  "estimatedMinutes": number
}"#;

const AVAILABLE_ROLES: &str = "cto, cmo, coo, fullstack_dev, frontend_dev, backend_dev, content_writer, designer, seo_specialist, devops, qa_engineer, social_media, data_analyst";
