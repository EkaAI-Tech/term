use std::fs;
use std::path::PathBuf;

fn get_history_file_path() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| "Failed to get home directory".to_string())?;
    
    Ok(home_dir.join(".term_history"))
}

#[tauri::command]
pub fn save_command_to_history(command: String) -> Result<(), String> {
    let history_file = get_history_file_path()?;
    
    // Read existing history
    let mut history = if history_file.exists() {
        fs::read_to_string(&history_file)
            .map_err(|e| format!("Failed to read history file: {}", e))?
    } else {
        String::new()
    };
    
    // Append new command with newline
    if !history.is_empty() && !history.ends_with('\n') {
        history.push('\n');
    }
    history.push_str(&command);
    history.push('\n');
    
    // Write back to file
    fs::write(&history_file, history)
        .map_err(|e| format!("Failed to write history file: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub fn load_command_history() -> Result<Vec<String>, String> {
    let history_file = get_history_file_path()?;
    
    if !history_file.exists() {
        return Ok(Vec::new());
    }
    
    let content = fs::read_to_string(&history_file)
        .map_err(|e| format!("Failed to read history file: {}", e))?;
    
    let history: Vec<String> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string())
        .collect();
    
    Ok(history)
}

#[tauri::command]
pub fn clear_command_history() -> Result<(), String> {
    let history_file = get_history_file_path()?;
    
    if history_file.exists() {
        fs::remove_file(&history_file)
            .map_err(|e| format!("Failed to clear history file: {}", e))?;
    }
    
    Ok(())
}
