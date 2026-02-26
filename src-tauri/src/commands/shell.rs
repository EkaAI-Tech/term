// use std::process::Command;

// #[tauri::command]
// pub async fn run_shell(command: String) -> Result<String, String> {
//     let cmd = command.trim();

//     if cmd == "exit" {
//         return Ok("__EXIT_SHELL__".to_string());
//     }

//     // Handle minimal ls commands
//     if cmd == "ls" {
//         return handle_minimal_ls(None, false).await;
//     } else if cmd == "la" {
//         return handle_minimal_ls(None, true).await;
//     } else if cmd.starts_with("ls ") {
//         let path = cmd.strip_prefix("ls ").unwrap().trim();
//         let path = if path.is_empty() { None } else { Some(path.to_string()) };
//         return handle_minimal_ls(path, false).await;
//     } else if cmd.starts_with("la ") {
//         let path = cmd.strip_prefix("la ").unwrap().trim();
//         let path = if path.is_empty() { None } else { Some(path.to_string()) };
//         return handle_minimal_ls(path, true).await;
//     }

//     // Regular command execution (non-ls commands)
//     let output = if cfg!(target_os = "windows") {
//         Command::new("cmd").args(["/C", &command]).output()
//     } else {
//         Command::new("sh").arg("-c").arg(&command).output()
//     };

//     match output {
//         Ok(out) => {
//             let stdout = String::from_utf8_lossy(&out.stdout);
//             let stderr = String::from_utf8_lossy(&out.stderr);

//             if !stderr.is_empty() && stdout.is_empty() {
//                 Ok(stderr.to_string())
//             } else if !stdout.is_empty() {
//                 Ok(stdout.to_string())
//             } else {
//                 Ok(String::from(
//                     "Command executed successfully with no output.",
//                 ))
//             }
//         }
//         Err(e) => Err(format!("Failed to run command: {}", e)),
//     }
// }

// async fn handle_minimal_ls(path: Option<String>, show_hidden: bool) -> Result<String, String> {
//     let dir_path = path.unwrap_or_else(|| ".".to_string());
    
//     // Build the appropriate ls command based on OS and options
//     let ls_command = if cfg!(target_os = "windows") {
//         if show_hidden {
//             format!("dir /a /b \"{}\"", dir_path)
//         } else {
//             format!("dir /b \"{}\"", dir_path)
//         }
//     } else {
//         if show_hidden {
//             format!("ls -la \"{}\"", dir_path)
//         } else {
//             format!("ls -l \"{}\"", dir_path)
//         }
//     };

//     let output = if cfg!(target_os = "windows") {
//         Command::new("cmd").args(["/C", &ls_command]).output()
//     } else {
//         Command::new("sh").arg("-c").arg(&ls_command).output()
//     };

//     match output {
//         Ok(out) => {
//             let stdout = String::from_utf8_lossy(&out.stdout);
//             let stderr = String::from_utf8_lossy(&out.stderr);

//             if !stderr.is_empty() && stdout.is_empty() {
//                 return Ok(stderr.to_string());
//             }

//             // Format the output with colors and indicators
//             Ok(format_minimal_ls_output(&stdout, show_hidden))
//         }
//         Err(e) => Err(format!("Failed to run ls command: {}", e)),
//     }
// }

// fn format_minimal_ls_output(output: &str, show_hidden: bool) -> String {
//     let lines: Vec<&str> = output.lines().collect();
//     let mut formatted_lines = Vec::new();

//     // Skip the first line if it starts with "total" (ls summary)
//     let start_idx = if lines.get(0).map_or(false, |l| l.starts_with("total ")) {
//         1
//     } else {
//         0
//     };

//     for line in lines.iter().skip(start_idx) {
//         let line_trim = line.trim();
//         if line_trim.is_empty() {
//             continue;
//         }

//         if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
//             if let Some(formatted) = format_unix_ls_line(line_trim, show_hidden) {
//                 formatted_lines.push(formatted);
//             }
//         } else {
//             if let Some(formatted) = format_windows_ls_line(line_trim, show_hidden) {
//                 formatted_lines.push(formatted);
//             }
//         }
//     }

//     formatted_lines.join("\n")
// }

// fn format_unix_ls_line(line: &str, show_hidden: bool) -> Option<String> {
//     if line.len() < 10 {
//         return None;
//     }

//     let parts: Vec<&str> = line.split_whitespace().collect();
//     if parts.len() < 9 {
//         return None;
//     }

//     // Join all parts from index 8 to handle filenames with spaces
//     let filename = parts[8..].join(" ");

//     // Skip . and .. entries, and hidden files if not showing hidden
//     if filename == "." || filename == ".." {
//         return None;
//     }
    
//     if !show_hidden && filename.starts_with('.') {
//         return None;
//     }

//     // Get file type and permissions
//     let permissions = &parts[0];
//     let file_type = permissions.chars().next().unwrap_or('?');
    
//     // Format with colors and indicators
//     let formatted_name = match file_type {
//         'd' => {
//             // Directory - blue color with trailing /
//             format!("\x1b[94m{}/\x1b[0m", filename)
//         }
//         'l' => {
//             //Symbolic link - cyan color
//             format!("\x1b[96m{}\x1b[0m", filename)
//         }
//         '-' => {
//             // Regular file - check if executable
//             if permissions.chars().nth(3).unwrap_or('-') == 'x' ||
//                permissions.chars().nth(6).unwrap_or('-') == 'x' ||
//                permissions.chars().nth(9).unwrap_or('-') == 'x' {
//                 // Executable - green color with trailing *
//                 format!("\x1b[92m{}*\x1b[0m", filename)
//             } else {
//                 // Regular file - default color
//                 filename
//             }
//         }
//         _ => filename, // Other file types - default color
//     };

//     Some(formatted_name)
// }

// fn format_windows_ls_line(line: &str, show_hidden: bool) -> Option<String> {
//     let filename = line.trim();
    
//     if filename == "." || filename == ".." {
//         return None;
//     }

//     // Skip hidden files if not showing hidden (Windows hidden files start with .)
//     if !show_hidden && filename.starts_with('.') {
//         return None;
//     }

//     let path = std::path::Path::new(filename);
    
//     // Format based on file type
//     if path.is_dir() {
//         // Directory - blue color with trailing /
//         Some(format!("\x1b[94m{}/\x1b[0m", filename))
//     } else if filename.ends_with(".exe") || 
//               filename.ends_with(".bat") || 
//               filename.ends_with(".cmd") {
//         // Executable - green color with trailing *
//         Some(format!("\x1b[92m{}*\x1b[0m", filename))
//     } else {
//         // Regular file - default color
//         Some(filename.to_string())
//     }
// }

// #[tauri::command]
// pub async fn run_sudo_command(command: String, password: String) -> Result<String, String> {
//     if !cfg!(target_os = "linux") && !cfg!(target_os = "macos") {
//         return Err("Sudo is only supported on Linux and macOS".to_string());
//     }

//     let cmd = if command.starts_with("sudo ") {
//         command[5..].to_string()
//     } else {
//         command
//     };

//     let temp_dir = std::env::temp_dir();
//     let output_file = temp_dir.join(format!("term_sudo_{}", std::process::id()));
//     let output_path = output_file.to_string_lossy();

//     let script = format!(
//         r#"#!/bin/bash
// echo "{}" | sudo -S {} > "{}" 2>&1
// exit_code=$?
// if [ $exit_code -ne 0 ]; then
//     echo "Command failed with exit code $exit_code" >> "{}"
// fi
// "#,
//         password.replace("\"", "\\\""),
//         cmd.replace("\"", "\\\""),
//         output_path,
//         output_path
//     );

//     let script_file = temp_dir.join(format!("term_sudo_script_{}", std::process::id()));
//     std::fs::write(&script_file, script).map_err(|e| format!("Failed to create script: {}", e))?;

//     #[cfg(not(target_os = "windows"))]
//     {
//         use std::os::unix::fs::PermissionsExt;
//         let mut perms = std::fs::metadata(&script_file)
//             .map_err(|e| format!("Failed to get file metadata: {}", e))?
//             .permissions();
//         perms.set_mode(0o755);
//         std::fs::set_permissions(&script_file, perms)
//             .map_err(|e| format!("Failed to set permissions: {}", e))?;
//     }

//     let _status = tokio::process::Command::new(&script_file)
//         .status()
//         .await
//         .map_err(|e| format!("Failed to execute script: {}", e))?;

//     let _ = std::fs::remove_file(&script_file);

//     let output = std::fs::read_to_string(&output_file)
//         .map_err(|e| format!("Failed to read output: {}", e))?;

//     let _ = std::fs::remove_file(&output_file);

//     if output.contains("incorrect password")
//         || output.contains("Sorry, try again")
//         || output.contains("Authentication failure")
//         || output.contains("authentication failure")
//         || output.contains("sudo: no password was provided")
//         || output.contains("sudo: 1 incorrect password attempt")
//     {
//         return Err("Incorrect password provided".to_string());
//     }

//     Ok(output)
// }

// #[tauri::command]
// pub fn get_current_dir() -> Result<String, String> {
//     std::env::current_dir()
//         .map(|path| path.to_string_lossy().into_owned())
//         .map_err(|e| format!("Failed to get current directory: {}", e))
// }

// #[tauri::command]
// pub async fn list_directory_contents(path: Option<String>) -> Result<Vec<String>, String> {
//     let dir_path = match path {
//         Some(p) if !p.is_empty() => p,
//         _ => ".".to_string(),
//     };

//     let ls_command = if cfg!(target_os = "windows") {
//         format!("dir /b \"{}\"", dir_path)
//     } else {
//         format!("ls -la \"{}\"", dir_path)
//     };

//     let output = if cfg!(target_os = "windows") {
//         Command::new("cmd").args(["/C", &ls_command]).output()
//     } else {
//         Command::new("sh").arg("-c").arg(&ls_command).output()
//     };

//     match output {
//         Ok(out) => {
//             let stdout = String::from_utf8_lossy(&out.stdout);
//             let lines: Vec<&str> = stdout.lines().collect();
//             let mut file_list = Vec::new();

//             // Skip the first line if starts with "total" (ls summary)
//             let start_idx = if lines.get(0).map_or(false, |l| l.starts_with("total ")) {
//                 1
//             } else {
//                 0
//             };

//             for line in lines.iter().skip(start_idx) {
//                 let line_trim = line.trim();
//                 if line_trim.is_empty() {
//                     continue;
//                 }

//                 // Unix-style ls output with permissions
//                 if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
//                     if line_trim.len() < 10 {
//                         continue;
//                     }

//                     let parts: Vec<&str> = line_trim.split_whitespace().collect();
//                     if parts.len() < 9 {
//                         continue;
//                     }

//                     // Join all parts from index 8 to handle filenames with spaces
//                     let filename = parts[8..].join(" ");

//                     // Skip . and .. entries
//                     if filename == "." || filename == ".." {
//                         continue;
//                     }

//                     // Check file type and add appropriate suffix
//                     let file_type = line_trim.chars().next().unwrap_or('?');
//                     if file_type == 'd' {
//                         file_list.push(format!("{}/", filename));
//                     } else if line_trim.contains("x") && file_type == '-' {
//                         file_list.push(format!("{}*", filename));
//                     } else {
//                         file_list.push(filename);
//                     }
//                 } else {
//                     // Windows directory listing (simpler format)
//                     if line_trim != "." && line_trim != ".." {
//                         let path = std::path::Path::new(line_trim);
//                         if path.is_dir() {
//                             file_list.push(format!("{}/", line_trim));
//                         } else if line_trim.ends_with(".exe")
//                             || line_trim.ends_with(".bat")
//                             || line_trim.ends_with(".cmd")
//                         {
//                             file_list.push(format!("{}*", line_trim));
//                         } else {
//                             file_list.push(line_trim.to_string());
//                         }
//                     }
//                 }
//             }

//             Ok(file_list)
//         }
//         Err(e) => Err(format!("Failed to list directory: {}", e)),
//     }
// }

// #[tauri::command]
// pub fn change_directory(path: String) -> Result<String, String> {
//     let expanded_path = if path.starts_with("~") {
//         if let Ok(home) = std::env::var("HOME") {
//             path.replacen("~", &home, 1)
//         } else {
//             path
//         }
//     } else {
//         path
//     };

//     match std::env::set_current_dir(expanded_path) {
//         Ok(_) => {
//             let new_dir = std::env::current_dir()
//                 .map_err(|e| format!("Failed to get current directory: {}", e))?;
//             Ok(new_dir.to_string_lossy().into_owned())
//         },
//         Err(e) => Err(format!("Failed to change directory: {}", e))
//     }
// }
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;

#[tauri::command]
pub async fn run_shell(command: String) -> Result<String, String> {
    let cmd = command.trim();

    if cmd == "exit" {
        return Ok("__EXIT_SHELL__".to_string());
    }

    // Special handling for ls/dir commands with robust parsing
    if cmd == "ls" || cmd.starts_with("ls ") || cmd == "dir" || cmd.starts_with("dir ") {
        return handle_ls_command(command).await;
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

async fn handle_ls_command(original_command: String) -> Result<String, String> {
    let cmd = original_command.trim();
    
    // Extract the path from the ls command
    let target_path = extract_path_from_ls_command(cmd);
    let current_dir = std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let absolute_path = resolve_path(&target_path, &current_dir);

    // Use a more reliable ls format for Unix systems
    let enhanced_command = if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        // Use --time-style=long-iso for consistent parsing and -la for details
        if cmd == "ls" {
            format!("ls -la --time-style=long-iso \"{}\"", absolute_path.display())
        } else if cmd.starts_with("ls ") {
            // Check if user already specified format options
            if !cmd.contains(" -l") && !cmd.contains(" -a") && !cmd.contains("--time-style") {
                format!("{} -la --time-style=long-iso", cmd)
            } else if !cmd.contains("--time-style") {
                format!("{} --time-style=long-iso", cmd)
            } else {
                cmd.to_string()
            }
        } else {
            cmd.to_string()
        }
    } else {
        // Windows - use dir command with specific formatting
        format!("dir /a \"{}\"", absolute_path.display())
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

            // Process the output with robust parsing
            Ok(crate::utils::format_directory_listing_robust(&stdout, &absolute_path))
        }
        Err(e) => Err(format!("Failed to run command: {}", e)),
    }
}

fn extract_path_from_ls_command(cmd: &str) -> String {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    
    // Look for the last argument that doesn't start with '-'
    for part in parts.iter().rev() {
        if !part.starts_with('-') && *part != "ls" && *part != "dir" {
            return part.to_string();
        }
    }
    
    // Default to current directory
    ".".to_string()
}

fn resolve_path(path: &str, current_dir: &Path) -> PathBuf {
    let path_buf = PathBuf::from(path);
    
    if path_buf.is_absolute() {
        path_buf
    } else {
        current_dir.join(path_buf)
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
        Some(p) if !p.is_empty() => {
            let expanded = crate::utils::expand_home_path(&p)
                .map_err(|e| format!("Failed to expand path: {}", e))?;
            PathBuf::from(expanded)
        },
        _ => std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?,
    };

    // Use direct file system reading instead of shell commands for reliability
    match fs::read_dir(&dir_path) {
        Ok(entries) => {
            let mut file_list = Vec::new();
            
            for entry in entries {
                match entry {
                    Ok(dir_entry) => {
                        let file_name = dir_entry.file_name().to_string_lossy().into_owned();
                        
                        // Skip hidden files starting with . (make this configurable if needed)
                        if file_name.starts_with('.') && file_name != "." && file_name != ".." {
                            continue;
                        }
                        
                        let file_type = get_file_type_robust(&dir_entry.path());
                        
                        match file_type {
                            FileType::Directory => file_list.push(format!("{}/", file_name)),
                            FileType::Executable => file_list.push(format!("{}*", file_name)),
                            FileType::Symlink => file_list.push(format!("{}@", file_name)),
                            FileType::Regular => file_list.push(file_name),
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading directory entry: {}", e);
                        continue;
                    }
                }
            }
            
            // Sort the results for consistent output
            file_list.sort();
            Ok(file_list)
        }
        Err(e) => Err(format!("Failed to read directory '{}': {}", dir_path.display(), e)),
    }
}

#[tauri::command]
pub fn change_directory(path: String) -> Result<String, String> {
    let expanded_path = crate::utils::expand_home_path(&path)
        .map_err(|e| format!("Failed to expand path: {}", e))?;

    match std::env::set_current_dir(&expanded_path) {
        Ok(_) => {
            let new_dir = std::env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {}", e))?;
            Ok(new_dir.to_string_lossy().into_owned())
        },
        Err(e) => Err(format!("Failed to change directory to '{}': {}", expanded_path, e))
    }
}

#[derive(Debug)]
enum FileType {
    Directory,
    Symlink,
    Executable,
    Regular,
}

fn get_file_type_robust(path: &Path) -> FileType {
    // Use std::fs to get accurate file information
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                FileType::Symlink
            } else if metadata.is_dir() {
                FileType::Directory
            } else if is_executable(&metadata, path) {
                FileType::Executable
            } else {
                FileType::Regular
            }
        }
        Err(_) => {
            // Fallback to basic checks if metadata fails
            if path.is_dir() {
                FileType::Directory
            } else {
                FileType::Regular
            }
        }
    }
}

#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata, _path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    mode & 0o111 != 0 // Check if any execute bit is set
}

#[cfg(windows)]
fn is_executable(_metadata: &fs::Metadata, path: &Path) -> bool {
    // On Windows, check file extension
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "exe" | "bat" | "cmd" | "com" | "ps1")
    } else {
        false
    }
}

#[cfg(not(any(unix, windows)))]
fn is_executable(_metadata: &fs::Metadata, _path: &Path) -> bool {
    false
}