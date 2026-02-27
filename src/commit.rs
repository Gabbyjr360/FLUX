// Commit module for creating and managing commits.
use anyhow::anyhow;
use serde_json;
use crate::storage::{get_object, FLUX_DIR};
use std::fs;
use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Commit {
    pub parent: Option<String>,
    pub tree: String,
    pub message: String,
}

/// Create and store a commit object, return hash.
pub fn create_commit(tree: &str, message: &str, parent: Option<String>) -> anyhow::Result<String> {
    let commit = Commit {
        parent,
        tree: tree.to_string(),
        message: message.to_string(),
    };
    let json = serde_json::to_string(&commit)?;
    crate::storage::store_object(json.as_bytes())
}

/// Get the hash of the current commit (from current branch).
pub fn get_current_commit() -> anyhow::Result<String> {
    let head = fs::read_to_string(Path::new(FLUX_DIR).join("HEAD"))?.trim().to_string();
    let ref_path = head.strip_prefix("ref: ").ok_or(anyhow!("Invalid HEAD"))?.to_string();
    let commit_hash = fs::read_to_string(Path::new(FLUX_DIR).join(ref_path))?.trim().to_string();
    Ok(commit_hash)
}

/// Get the committed file hash from the current commit tree.
pub fn get_committed_file_hash(file: &str) -> anyhow::Result<Option<String>> {
    let commit_hash = get_current_commit()?;
    let commit_content = get_object(&commit_hash)?;
    let commit: Commit = serde_json::from_slice(&commit_content)?;
    let tree_content = get_object(&commit.tree)?;
    let tree: crate::storage::Tree = serde_json::from_slice(&tree_content)?;
    for entry in tree.entries {
        if entry.name == file {
            return Ok(Some(entry.hash));
        }
    }
    Ok(None)
}