use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use ajen_engine::director::Director;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateCompanyRequest {
    pub description: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApproveResponse {
    pub company_id: String,
    pub status: String,
}

async fn list_companies(State(state): State<AppState>) -> impl IntoResponse {
    match state.engine.company_store.list().await {
        Ok(records) => {
            let companies: Vec<CompanyResponse> = records
                .into_iter()
                .map(|r| CompanyResponse {
                    id: r.id,
                    name: r
                        .plan
                        .as_ref()
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| "Planning...".to_string()),
                    description: r.description,
                    status: format!("{:?}", r.phase).to_lowercase(),
                    created_at: r.created_at.to_rfc3339(),
                })
                .collect();
            (StatusCode::OK, Json(companies)).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn create_company(
    State(state): State<AppState>,
    Json(body): Json<CreateCompanyRequest>,
) -> impl IntoResponse {
    let director = Director::new(state.engine.clone());

    match director.start_company(body.description.clone()).await {
        Ok(company_id) => {
            let company = CompanyResponse {
                id: company_id,
                name: "Planning...".to_string(),
                description: body.description,
                status: "planning".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            (StatusCode::CREATED, Json(company)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_company(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let director = Director::new(state.engine.clone());

    match director.get_status(&id).await {
        Ok(status) => (StatusCode::OK, Json(serde_json::to_value(status).unwrap())).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Company not found" })),
        )
            .into_response(),
    }
}

async fn approve_company(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let director = Director::new(state.engine.clone());

    match director.approve_company(&id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApproveResponse {
                company_id: id,
                status: "approved".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/companies", get(list_companies).post(create_company))
        .route("/companies/{id}", get(get_company))
        .route("/companies/{id}/approve", post(approve_company))
}
