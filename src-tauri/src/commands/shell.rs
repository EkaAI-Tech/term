use std::process::Command;
use std::fs;
use std::path::Path;

#[tauri::command]
pub async fn run_shell(command: String) -> Result<String, String> {
    let cmd = command.trim();

    if cmd == "exit" {
        return Ok("__EXIT_SHELL__".to_string());
    }

    // Special handling for ls/dir commands to add file type indicators
    if cmd == "ls" || cmd.starts_with("ls ") || cmd == "dir" || cmd.starts_with("dir ") {
        let enhanced_command = if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
            if cmd == "ls" {
                "ls -la".to_string()
            } else if cmd.starts_with("ls ") {
                if !command.contains(" -l") && !command.contains(" -a") {
                    format!("{} -la", command)
                } else if !command.contains(" -l") {
                    format!("{} -l", command)
                } else if !command.contains(" -a") {
                    format!("{} -a", command)
                } else {
                    command.clone()
                }
            } else {
                command.clone()
            }
        } else {
            command.clone()
        };

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", &enhanced_command]).output()
        } else {
            Command::new("sh").arg("-c").arg(&enhanced_command).output()
        };

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);

                if !stderr.is_empty() && stdout.is_empty() {
                    return Ok(stderr.to_string());
                }

                // Process the output to add file type indicators
                return Ok(crate::utils::format_directory_listing(&stdout));
            }
            Err(e) => return Err(format!("Failed to run command: {}", e)),
        }
    }

    // Regular command execution (non-ls commands)
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", &command]).output()
    } else {
        Command::new("sh").arg("-c").arg(&command).output()
    };

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            if !stderr.is_empty() && stdout.is_empty() {
                Ok(stderr.to_string())
            } else if !stdout.is_empty() {
                Ok(stdout.to_string())
            } else {
                Ok(String::from(
                    "Command executed successfully with no output.",
                ))
            }
        }
        Err(e) => Err(format!("Failed to run command: {}", e)),
    }
}

#[tauri::command]
pub async fn run_sudo_command(command: String, password: String) -> Result<String, String> {
    if !cfg!(target_os = "linux") && !cfg!(target_os = "macos") {
        return Err("Sudo is only supported on Linux and macOS".to_string());
    }

    let cmd = if command.starts_with("sudo ") {
        command[5..].to_string()
    } else {
        command
    };

    let temp_dir = std::env::temp_dir();
    let output_file = temp_dir.join(format!("term_sudo_{}", std::process::id()));
    let output_path = output_file.to_string_lossy();

    let script = format!(
        r#"#!/bin/bash
echo "{}" | sudo -S {} > "{}" 2>&1
exit_code=$?
if [ $exit_code -ne 0 ]; then
    echo "Command failed with exit code $exit_code" >> "{}"
fi
"#,
        password.replace("\"", "\\\""),
        cmd.replace("\"", "\\\""),
        output_path,
        output_path
    );

    let script_file = temp_dir.join(format!("term_sudo_script_{}", std::process::id()));
    std::fs::write(&script_file, script).map_err(|e| format!("Failed to create script: {}", e))?;

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_file)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_file, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    let _status = tokio::process::Command::new(&script_file)
        .status()
        .await
        .map_err(|e| format!("Failed to execute script: {}", e))?;

    let _ = std::fs::remove_file(&script_file);

    let output = std::fs::read_to_string(&output_file)
        .map_err(|e| format!("Failed to read output: {}", e))?;

    let _ = std::fs::remove_file(&output_file);

    if output.contains("incorrect password")
        || output.contains("Sorry, try again")
        || output.contains("Authentication failure")
        || output.contains("authentication failure")
        || output.contains("sudo: no password was provided")
        || output.contains("sudo: 1 incorrect password attempt")
    {
        return Err("Incorrect password provided".to_string());
    }

    Ok(output)
}

#[tauri::command]
pub fn get_current_dir() -> Result<String, String> {
    std::env::current_dir()
        .map(|path| {
            let mut normalized = path.to_string_lossy().replace("\\", "/");
            if let Some(index) = normalized.find(':') {
                normalized = format!(".{}", &normalized[index + 1..]);
            }
            normalized
        })
        .map_err(|e| format!("Failed to get current directory: {}", e))
}

#[tauri::command]
pub fn get_home_dir() -> Result<String, String> {
    if let Some(home) = dirs::home_dir() {
        Ok(home.to_string_lossy().into_owned())
    } else {
        Err("Failed to get home directory".to_string())
    }
}


#[tauri::command]
pub async fn list_directory_contents(path: Option<String>) -> Result<Vec<String>, String> {
    let dir_path = match path {
        Some(p) if !p.is_empty() => p,
        _ => ".".to_string(),
    };

    let ls_command = if cfg!(target_os = "windows") {
        format!("dir /b \"{}\"", dir_path)
    } else {
        format!("ls -la \"{}\"", dir_path)
    };

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", &ls_command]).output()
    } else {
        Command::new("sh").arg("-c").arg(&ls_command).output()
    };

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let lines: Vec<&str> = stdout.lines().collect();
            let mut file_list = Vec::new();

            // Skip the first line if it starts with "total" (ls summary)
            let start_idx = if lines.get(0).map_or(false, |l| l.starts_with("total ")) {
                1
            } else {
                0
            };

            for line in lines.iter().skip(start_idx) {
                let line_trim = line.trim();
                if line_trim.is_empty() {
                    continue;
                }

                // Unix-style ls output with permissions
                if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
                    if line_trim.len() < 10 {
                        continue;
                    }

                    let parts: Vec<&str> = line_trim.split_whitespace().collect();
                    if parts.len() < 9 {
                        continue;
                    }

                    // Join all parts from index 8 to handle filenames with spaces
                    let filename = parts[8..].join(" ");

                    // Skip . and .. entries
                    if filename == "." || filename == ".." {
                        continue;
                    }

                    // Check file type and add appropriate suffix
                    let file_type = line_trim.chars().next().unwrap_or('?');
                    if file_type == 'd' {
                        file_list.push(format!("{}/", filename));
                    } else if line_trim.contains("x") && file_type == '-' {
                        file_list.push(format!("{}*", filename));
                    } else {
                        file_list.push(filename);
                    }
                } else {
                    // Windows directory listing (simpler format)
                    if line_trim != "." && line_trim != ".." {
                        let path = std::path::Path::new(line_trim);
                        if path.is_dir() {
                            file_list.push(format!("{}/", line_trim));
                        } else if line_trim.ends_with(".exe")
                            || line_trim.ends_with(".bat")
                            || line_trim.ends_with(".cmd")
                        {
                            file_list.push(format!("{}*", line_trim));
                        } else {
                            file_list.push(line_trim.to_string());
                        }
                    }
                }
            }

            Ok(file_list)
        }
        Err(e) => Err(format!("Failed to list directory: {}", e)),
    }
}

#[tauri::command]
pub fn change_directory(path: String) -> Result<String, String> {
    let expanded_path = if path.starts_with("~") {
        if let Ok(home) = std::env::var("HOME") {
            path.replacen("~", &home, 1)
        } else {
            path
        }
    } else {
        path
    };

    match std::env::set_current_dir(expanded_path) {
        Ok(_) => {
            let new_dir = std::env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {}", e))?;
            Ok(new_dir.to_string_lossy().into_owned())
        },
        Err(e) => Err(format!("Failed to change directory: {}", e))
    }
}


#[tauri::command]
pub fn read_file_for_editor(path: String) -> Result<String, String> {
    let expanded_path = if path.starts_with("~") {
        if let Ok(home) = std::env::var("HOME") {
            path.replacen("~", &home, 1)
        } else {
            path
        }
    } else {
        path
    };

    // Make path absolute if it's relative
    let absolute_path = if Path::new(&expanded_path).is_absolute() {
        expanded_path
    } else {
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        current_dir.join(&expanded_path)
            .to_string_lossy()
            .into_owned()
    };

    fs::read_to_string(&absolute_path)
        .map_err(|e| format!("Failed to read file '{}': {}", absolute_path, e))
}

#[tauri::command]
pub fn write_file_from_editor(path: String, content: String) -> Result<(), String> {
    let expanded_path = if path.starts_with("~") {
        if let Ok(home) = std::env::var("HOME") {
            path.replacen("~", &home, 1)
        } else {
            path
        }
    } else {
        path
    };

    // Make path absolute if it's relative
    let absolute_path = if Path::new(&expanded_path).is_absolute() {
        expanded_path
    } else {
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        current_dir.join(&expanded_path)
            .to_string_lossy()
            .into_owned()
    };

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(&absolute_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directories: {}", e))?;
    }

    fs::write(&absolute_path, content)
        .map_err(|e| format!("Failed to write file '{}': {}", absolute_path, e))
}
