// CLI functions for Flux commands.
// Each function corresponds to a CLI subcommand and handles the logic.

use crate::commit::{get_current_commit, Commit};
use crate::diff::show_diff;
use crate::storage::{create_tree, read_index, store_object, write_index, IndexEntry};

use std::fs;
use std::path::Path;

// If serde_json is used → make sure it's imported
use serde_json;

pub fn init() -> anyhow::Result<()> {
    // Create the .flux directory structure for the repository.
    fs::create_dir(".flux")?;
    fs::create_dir(".flux/objects")?;
    fs::create_dir_all(".flux/refs/heads")?;
    // Set HEAD to point to the main branch by default.
    fs::write(".flux/HEAD", "ref: refs/heads/main\n")?;
    // Initialize an empty index.
    write_index(&Vec::new())?;
    println!("Initialized empty Flux repository.");
    Ok(())
}

pub fn add(file: &str) -> anyhow::Result<()> {
    // Read the file content to add.
    let content = fs::read(file)?;
    // Store the content as a blob object in CAS.
    let hash = store_object(&content)?;
    // Update or add to the staging index.
    let mut index = read_index()?;
    if let Some(entry) = index.iter_mut().find(|e| e.path == file) {
        entry.hash = hash;
    } else {
        index.push(IndexEntry {
            path: file.to_string(),
            hash,
        });
    }
    write_index(&index)?;
    println!("Added {} to staging.", file);
    Ok(())
}

pub fn commit(message: &str) -> anyhow::Result<()> {
    // Read the current staging index.
    let index = read_index()?;
    if index.is_empty() {
        return Err(anyhow::anyhow!("Nothing to commit."));
    }
    // Create a tree object from the index.
    let entries: Vec<(String, String)> = index.iter().map(|e| (e.path.clone(), e.hash.clone())).collect();
    let tree_hash = create_tree(&entries)?;
    // Get the parent commit if it exists.
    let parent = get_current_commit().ok();
    // Create the commit object.
    let commit = Commit {
        parent,
        tree: tree_hash,
        message: message.to_string(),
    };
    let commit_json = serde_json::to_string(&commit)?;
    let commit_hash = store_object(commit_json.as_bytes())?;
    // Update the current branch ref with the new commit hash.
    let branch = get_current_branch()?;
    update_ref(&branch, &commit_hash)?;
    println!("Committed changes: {}", message);
    Ok(())
}

pub fn diff(file: &str) -> anyhow::Result<()> {
    // Show differences between working directory file and the committed version.
    show_diff(file)?;
    Ok(())
}

pub fn sync() -> anyhow::Result<()> {
    // For V1, sync is a no-op that just confirms local save (no remote).
    println!("Changes synced locally.");
    Ok(())
}

pub fn branch(name: &str) -> anyhow::Result<()> {
    // Create a new branch pointing to the current commit.
    let current_commit = get_current_commit()?;
    let branch_path = Path::new(".flux/refs/heads").join(name);
    fs::write(branch_path, current_commit)?;
    println!("Created branch {}.", name);
    Ok(())
}

pub fn checkout(name: &str) -> anyhow::Result<()> {
    // Switch HEAD to the specified branch (no workspace changes in V1).
    let head_content = format!("ref: refs/heads/{}\n", name);
    fs::write(".flux/HEAD", head_content)?;
    println!("Checked out branch {}.", name);
    // Note: In V1, this does not update the working directory files.
    Ok(())
}

// Helper: Get the current branch name from HEAD.
fn get_current_branch() -> anyhow::Result<String> {
    let head = fs::read_to_string(".flux/HEAD")?.trim().to_string();
    if let Some(ref_path) = head.strip_prefix("ref: ") {
        if let Some(branch) = ref_path.strip_prefix("refs/heads/") {
            return Ok(branch.to_string());
        }
    }
    Err(anyhow::anyhow!("Invalid HEAD"))
}

// Helper: Update a branch ref with a commit hash.
fn update_ref(branch: &str, hash: &str) -> anyhow::Result<()> {
    let ref_path = Path::new(".flux/refs/heads").join(branch);
    fs::write(ref_path, format!("{}\n", hash))?;
    Ok(())
}