use std::path::Path;

use ajen_core::types::manifest::{EmployeeManifest, ResolvedManifest};
use tokio::fs;
use tracing::debug;

pub async fn parse_manifest_file(path: &Path) -> anyhow::Result<EmployeeManifest> {
    let content = fs::read_to_string(path).await?;
    let manifest: EmployeeManifest = serde_yaml::from_str(&content)?;
    Ok(manifest)
}

pub async fn resolve_manifest(manifest_dir: &Path) -> anyhow::Result<ResolvedManifest> {
    let manifest_path = manifest_dir.join("manifest.yaml");
    let manifest = parse_manifest_file(&manifest_path).await?;

    debug!(manifest_id = %manifest.metadata.id, "resolving manifest");

    // Load persona content
    let persona_path = manifest_dir.join(&manifest.spec.persona);
    let persona_content = fs::read_to_string(&persona_path).await?;

    // Load skill contents
    let mut skill_contents = Vec::new();
    if let Some(ref skills) = manifest.spec.skills {
        for skill_path in skills {
            let full_path = manifest_dir.join(skill_path);
            match fs::read_to_string(&full_path).await {
                Ok(content) => skill_contents.push(content),
                Err(e) => {
                    tracing::warn!(path = %full_path.display(), error = %e, "failed to load skill file");
                }
            }
        }
    }

    Ok(ResolvedManifest {
        manifest,
        persona_content,
        skill_contents,
    })
}
