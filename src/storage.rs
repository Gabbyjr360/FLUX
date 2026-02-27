// Storage module for content-addressable storage (CAS).
// Handles hashing, storing, and retrieving objects.

use hex;
use serde_json;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub const FLUX_DIR: &str = ".flux";
const OBJECTS_DIR: &str = ".flux/objects";
const INDEX_FILE: &str = ".flux/index.json";

/// Compute SHA256 hash of content.
pub fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

/// Store content as an object in CAS, return hash.
pub fn store_object(content: &[u8]) -> anyhow::Result<String> {
    let hash = hash_content(content);
    let obj_path = Path::new(OBJECTS_DIR).join(&hash);
    if let Some(parent) = obj_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(obj_path)?;
    file.write_all(content)?;
    Ok(hash)
}

/// Retrieve object content by hash.
pub fn get_object(hash: &str) -> anyhow::Result<Vec<u8>> {
    let obj_path = Path::new(OBJECTS_DIR).join(hash);
    let mut file = File::open(obj_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}

/// Index entry for staging.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct IndexEntry {
    pub path: String,
    pub hash: String,
}

pub type Index = Vec<IndexEntry>;

/// Read the staging index from file.
pub fn read_index() -> anyhow::Result<Index> {
    if Path::new(INDEX_FILE).exists() {
        let content = fs::read_to_string(INDEX_FILE)?;
        let index: Index = serde_json::from_str(&content)?;
        Ok(index)
    } else {
        Ok(Vec::new())
    }
}

/// Write the staging index to file.
pub fn write_index(index: &Index) -> anyhow::Result<()> {
    let json = serde_json::to_string(index)?;
    fs::write(INDEX_FILE, json)?;
    Ok(())
}

/// Tree structure for commit snapshots.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Tree {
     pub entries: Vec<TreeEntry>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TreeEntry {
    pub name: String,
    pub hash: String,
}

/// Create and store a tree object from entries, return hash.
pub fn create_tree(entries: &[(String, String)]) -> anyhow::Result<String> {
    let mut tree_entries: Vec<TreeEntry> = entries
        .iter()
        .map(|(name, hash)| TreeEntry {
            name: name.clone(),
            hash: hash.clone(),
        })
        .collect();
    // Sort entries for consistent hashing.
    tree_entries.sort_by(|a, b| a.name.cmp(&b.name));
    let tree = Tree {
        entries: tree_entries,
    };
    let json = serde_json::to_string(&tree)?;
    store_object(json.as_bytes())
}