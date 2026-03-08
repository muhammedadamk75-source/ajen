use std::collections::HashMap;
use std::path::Path;

use ajen_core::types::manifest::ResolvedManifest;
use tracing::{info, warn};

use super::parser::resolve_manifest;

pub struct ManifestRegistry {
    manifests: HashMap<String, ResolvedManifest>,
    by_role: HashMap<String, String>,
}

impl ManifestRegistry {
    pub fn new() -> Self {
        Self {
            manifests: HashMap::new(),
            by_role: HashMap::new(),
        }
    }

    pub async fn load_from_directory(&mut self, dir: &Path) -> anyhow::Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_file = path.join("manifest.yaml");
            if !manifest_file.exists() {
                continue;
            }
            match resolve_manifest(&path).await {
                Ok(resolved) => {
                    info!(
                        manifest_id = %resolved.manifest.metadata.id,
                        role = %resolved.manifest.spec.role,
                        "loaded manifest"
                    );
                    self.register(resolved);
                }
                Err(e) => {
                    warn!(path = %path.display(), error = %e, "failed to load manifest");
                }
            }
        }
        Ok(())
    }

    pub fn register(&mut self, manifest: ResolvedManifest) {
        let id = manifest.manifest.metadata.id.clone();
        let role = manifest.manifest.spec.role.clone();
        self.by_role.insert(role, id.clone());
        self.manifests.insert(id, manifest);
    }

    pub fn get(&self, manifest_id: &str) -> Option<&ResolvedManifest> {
        self.manifests.get(manifest_id)
    }

    pub fn get_by_role(&self, role: &str) -> Option<&ResolvedManifest> {
        self.by_role.get(role).and_then(|id| self.manifests.get(id))
    }

    pub fn list(&self) -> Vec<&ResolvedManifest> {
        self.manifests.values().collect()
    }

    pub fn has(&self, manifest_id: &str) -> bool {
        self.manifests.contains_key(manifest_id)
    }
}

impl Default for ManifestRegistry {
    fn default() -> Self {
        Self::new()
    }
}
