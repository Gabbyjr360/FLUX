// Diff module for showing file differences.
// Supports text diff, binary, and minimal object-level for .blend files.
use anyhow::anyhow;
use hex;
use crate::commit::get_committed_file_hash;
use crate::storage::{get_object, hash_content};
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use sha2::{Digest, Sha256};
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::fs;

pub fn show_diff(file: &str) -> anyhow::Result<()> {
    // Read current working file content.
    let new_content = fs::read(file)?;
    let new_hash = hash_content(&new_content);

    // Get committed hash if exists.
    let old_hash_opt = get_committed_file_hash(file)?;
    if old_hash_opt.is_none() {
        println!("New file: {}", file);
        return Ok(());
    }
    let old_hash = old_hash_opt.unwrap();
    let old_content = get_object(&old_hash)?;

    if old_hash == new_hash {
        println!("No changes in {}.", file);
        return Ok(());
    }

    // Handle .blend specially.
    if file.ends_with(".blend") {
        // Note: Assumes non-compressed .blend files for V1.
        let old_objs = parse_blender_objects_bytes(&old_content)?;
        let new_objs = parse_blender_objects_bytes(&new_content)?;

        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut modified = Vec::new();

        for (name, hash) in &new_objs {
            match old_objs.get(name) {
                Some(old_hash) if old_hash == hash => {}
                Some(_) => modified.push(name.clone()),
                None => added.push(name.clone()),
            }
        }
        for (name, _) in &old_objs {
            if !new_objs.contains_key(name) {
                removed.push(name.clone());
            }
        }

        if added.is_empty() && removed.is_empty() && modified.is_empty() {
            println!("No object changes in {}.", file);
        } else {
            if !added.is_empty() {
                println!("Added objects: {:?}", added);
            }
            if !removed.is_empty() {
                println!("Removed objects: {:?}", removed);
            }
            if !modified.is_empty() {
                println!("Modified objects: {:?}", modified);
            }
        }
    } else {
        // Try as text, fallback to binary.
        if let (Ok(old_text), Ok(new_text)) = (String::from_utf8(old_content), String::from_utf8(new_content)) {
            let diff = TextDiff::from_lines(&old_text, &new_text);
            for change in diff.iter_all_changes() {
                match change.tag() {
                    ChangeTag::Delete => print!("-{}", change),
                    ChangeTag::Insert => print!("+{}", change),
                    ChangeTag::Equal => print!(" {}", change),
                }
            }
        } else {
            println!("Binary file {} differs.", file);
        }
    }

    Ok(())
}

/// Parse .blend file bytes to extract object names and data hashes.
/// Minimal parser for OB blocks; assumes non-compressed files.
fn parse_blender_objects_bytes(data: &[u8]) -> anyhow::Result<HashMap<String, String>> {
    if data.len() < 12 || &data[0..7] != b"BLENDER" {
        return Err(anyhow!("Not a valid Blender file"));
    }

    let pointer_size = if data[7] == b'_' { 4 } else if data[7] == b'-' { 8 } else { return Err(anyhow!("Invalid pointer size")); };
    let endian: &str = if data[8] == b'v' { "little" } else if data[8] == b'V' { "big" } else { return Err(anyhow!("Invalid endian")); };

    let mut pos = 12;
    let mut objects = HashMap::new();

    while pos < data.len() {
        let code = &data[pos..pos + 4];
        pos += 4;

        let size = if endian == "little" {
            LittleEndian::read_u32(&data[pos..pos + 4]) as usize
        } else {
            BigEndian::read_u32(&data[pos..pos + 4]) as usize
        };
        pos += 4;

        pos += pointer_size; // Skip oldptr

        pos += 4; // Skip sdna
        pos += 4; // Skip count (assume 1 for OB)

        if pos + size > data.len() {
            break;
        }
        let block_data = &data[pos..pos + size];
        pos += size;

        // Check for OB block (code starts with 'OB')
        if code[0] == b'O' && code[1] == b'B' {
            // Extract name: first 66 bytes, null-terminated.
            let name_end = block_data.iter().take(66).position(|&b| b == 0).unwrap_or(66);
            let name = String::from_utf8(block_data[0..name_end].to_vec())?;

            // Hash the block data for change detection.
            let mut hasher = Sha256::new();
            hasher.update(block_data);
            let data_hash = hex::encode(hasher.finalize());

            objects.insert(name, data_hash);
        }
    }

    Ok(objects)
}