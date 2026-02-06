use std::fs;
use std::io::{self};
use std::path::PathBuf;

pub fn get_autostart_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".config").join("hypr").join("autostart.conf")
    } else {
        PathBuf::from("/tmp/autostart.conf")
    }
}

pub fn is_enabled(script_name: &str) -> bool {
    let path = get_autostart_path();
    if !path.exists() {
        return false;
    }

    if let Ok(content) = fs::read_to_string(&path) {
        let target_str = format!("scripts/{}", script_name);
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("exec-once") && line.contains(&target_str) {
                 // Check if commented out (although we remove lines when disabling, users might comment manually)
                 if !line.starts_with('#') {
                     return true;
                 }
            }
        }
    }
    false
}

/// Updates the autostart configuration for a given script.
/// 
/// If `enabled` is true:
/// - Replaces any existing entry for this script with a new `exec-once` line including `args`.
/// 
/// If `enabled` is false:
/// - Removes any existing entry for this script.
pub fn update_script(script_name: &str, args: Option<String>, enabled: bool) -> io::Result<()> {
    let path = get_autostart_path();
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        String::new()
    };

    let mut new_lines = Vec::new();
    let target_str = format!("scripts/{}", script_name);
    
    // Construct the new line
    let args_str = args.unwrap_or_default();
    let exec_line = format!("exec-once = ~/.config/alloy/scripts/{} {}", script_name, args_str).trim().to_string();
    
    let mut found = false;

    for line in content.lines() {
        let trimmed = line.trim();
        // Check if this line belongs to our script
        if trimmed.contains(&target_str) && trimmed.contains("exec-once") {
            if enabled {
                // If enabling, we replace the line with our new version (updates args if changed)
                // Only add it once
                if !found {
                    new_lines.push(exec_line.clone());
                    found = true;
                }
            } else {
                // If disabling, skip this line
                continue; 
            }
        } else {
            new_lines.push(line.to_string());
        }
    }

    // If enabled and not found in existing lines, append it
    if enabled && !found {
        new_lines.push(exec_line);
    }

    // Join with newlines
    let new_content = new_lines.join("\n");
    // Ensure trailing newline
    let final_content = if new_content.is_empty() {
        new_content
    } else {
        new_content + "\n"
    };

    fs::write(&path, final_content)?;
    Ok(())
}
