// pub fn format_directory_listing(output: &str) -> String {
//     let lines: Vec<&str> = output.lines().collect();
//     let mut formatted_output = String::new();

//     for line in lines {
//         if line.trim().is_empty() {
//             formatted_output.push_str(line);
//             formatted_output.push('\n');
//             continue;
//         }

//         if line.starts_with("total ") || line.contains("Directory of") {
//             formatted_output.push_str(line);
//             formatted_output.push('\n');
//             continue;
//         }

//         // Special handling for Unix-style ls output
//         if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
//             let first_char = line.chars().next().unwrap_or(' ');

//             if first_char == 'd' {
//                 formatted_output.push_str(&format!("{{DIR}}{}{{/DIR}}", line));
//                 formatted_output.push('\n');
//                 continue;
//             } else if first_char == 'l' {
//                 formatted_output.push_str(&format!("{{LINK}}{}{{/LINK}}", line));
//                 formatted_output.push('\n');
//                 continue;
//             } else if first_char == '-' || first_char.is_alphanumeric() {
//                 formatted_output.push_str(&format!("{{FILE}}{}{{/FILE}}", line));
//                 formatted_output.push('\n');
//                 continue;
//             }
//         }

//         // Windows DIR command handling or fallback
//         let tokens: Vec<&str> = line.split_whitespace().collect();
//         if !tokens.is_empty() {
//             let name = tokens.last().unwrap_or(&"");

//             if line.contains("<DIR>") || name.ends_with("/") || name.ends_with("\\") {
//                 formatted_output.push_str(&format!("{{DIR}}{}{{/DIR}}", line));
//             } else {
//                 formatted_output.push_str(&format!("{{FILE}}{}{{/FILE}}", line));
//             }
//             formatted_output.push('\n');
//         } else {
//             formatted_output.push_str(line);
//             formatted_output.push('\n');
//         }
//     }

//     formatted_output
// }
use std::path::Path;
use std::fs;

pub fn format_directory_listing(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut formatted_output = String::new();

    for line in lines {
        if line.trim().is_empty() {
            formatted_output.push_str(line);
            formatted_output.push('\n');
            continue;
        }

        if line.starts_with("total ") || line.contains("Directory of") {
            formatted_output.push_str(line);
            formatted_output.push('\n');
            continue;
        }

        // Handle the new robust formatting tags
        if line.contains("{DIR}") || line.contains("{FILE}") || line.contains("{LINK}") || line.contains("{EXEC}") {
            let formatted_line = format_tagged_line(line);
            formatted_output.push_str(&formatted_line);
            formatted_output.push('\n');
            continue;
        }

        // Legacy handling for backwards compatibility
        if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
            let first_char = line.chars().next().unwrap_or(' ');

            if first_char == 'd' {
                formatted_output.push_str(&format!("{{DIR}}{}{{/DIR}}", line));
                formatted_output.push('\n');
                continue;
            } else if first_char == 'l' {
                formatted_output.push_str(&format!("{{LINK}}{}{{/LINK}}", line));
                formatted_output.push('\n');
                continue;
            } else if first_char == '-' || first_char.is_alphanumeric() {
                formatted_output.push_str(&format!("{{FILE}}{}{{/FILE}}", line));
                formatted_output.push('\n');
                continue;
            }
        }

        // Windows DIR command handling or fallback
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if !tokens.is_empty() {
            let name = tokens.last().unwrap_or(&"");

            if line.contains("<DIR>") || name.ends_with("/") || name.ends_with("\\") {
                formatted_output.push_str(&format!("{{DIR}}{}{{/DIR}}", line));
            } else {
                formatted_output.push_str(&format!("{{FILE}}{}{{/FILE}}", line));
            }
            formatted_output.push('\n');
        } else {
            formatted_output.push_str(line);
            formatted_output.push('\n');
        }
    }

    formatted_output
}

pub fn format_directory_listing_robust(output: &str, dir_path: &Path) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut formatted_output = String::new();

    for line in lines {
        let line_trim = line.trim();
        if line_trim.is_empty() {
            formatted_output.push_str(line);
            formatted_output.push('\n');
            continue;
        }

        // Skip total line and directory headers
        if line_trim.starts_with("total ") || line_trim.contains("Directory of") {
            formatted_output.push_str(line);
            formatted_output.push('\n');
            continue;
        }

        // Robust Unix-style ls parsing with --time-style=long-iso
        if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
            if let Some(formatted_line) = parse_unix_ls_line_robust(line_trim, dir_path) {
                formatted_output.push_str(&formatted_line);
                formatted_output.push('\n');
            } else {
                // Fallback for lines that don't match expected format
                formatted_output.push_str(line);
                formatted_output.push('\n');
            }
        } else {
            // Windows parsing
            if let Some(formatted_line) = parse_windows_dir_line_robust(line_trim, dir_path) {
                formatted_output.push_str(&formatted_line);
                formatted_output.push('\n');
            } else {
                formatted_output.push_str(line);
                formatted_output.push('\n');
            }
        }
    }

    formatted_output
}

fn parse_unix_ls_line_robust(line: &str, dir_path: &Path) -> Option<String> {
    // Expected format with --time-style=long-iso:
    // drwxr-xr-x 2 user group 4096 2023-12-01 10:30 filename
    
    // Skip . and .. entries
    if line.ends_with(" .") || line.ends_with(" ..") {
        return None;
    }

    // Check if it's a proper ls line (starts with permissions)
    if line.len() < 10 || !line.chars().next().map_or(false, |c| "dl-".contains(c)) {
        return None;
    }

    // More robust parsing: find the filename by looking for the last space after date/time
    // Format: permissions links owner group size date time filename
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 8 {
        return None;
    }

    // Find filename by reconstructing from the end
    // Date format: YYYY-MM-DD, Time format: HH:MM
    let mut filename_start_idx = None;
    
    // Look for the time pattern (HH:MM) and take everything after it as filename
    for (i, part) in parts.iter().enumerate() {
        if part.len() == 5 && part.contains(':') && part.matches(':').count() == 1 {
            // Validate it's actually a time (digits on both sides of colon)
            let time_parts: Vec<&str> = part.split(':').collect();
            if time_parts.len() == 2 {
                if time_parts[0].parse::<u32>().is_ok() && time_parts[1].parse::<u32>().is_ok() {
                    filename_start_idx = Some(i + 1);
                    break;
                }
            }
        }
    }

    let filename = if let Some(start_idx) = filename_start_idx {
        if start_idx < parts.len() {
            parts[start_idx..].join(" ")
        } else {
            return None;
        }
    } else {
        // Fallback: assume last part is filename
        parts.last()?.to_string()
    };

    // Skip hidden files starting with . (except if we want to show them)
    if filename.starts_with('.') && filename != "." && filename != ".." {
        // You might want to make this configurable
    }

    // Use actual file system check instead of relying on ls output parsing
    let file_path = dir_path.join(&filename);
    let file_type = get_file_type_robust(&file_path);

    match file_type {
        FileType::Directory => Some(format!("{{DIR}}{}{{/DIR}}", line)),
        FileType::Symlink => Some(format!("{{LINK}}{}{{/LINK}}", line)),
        FileType::Executable => Some(format!("{{EXEC}}{}{{/EXEC}}", line)),
        FileType::Regular => Some(format!("{{FILE}}{}{{/FILE}}", line)),
    }
}

fn parse_windows_dir_line_robust(line: &str, dir_path: &Path) -> Option<String> {
    // Windows dir output format varies, but generally:
    // MM/DD/YYYY  HH:MM AM/PM    <DIR>          dirname
    // MM/DD/YYYY  HH:MM AM/PM         1,234     filename.ext

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    // Extract filename (last part typically)
    let filename = parts.last()?.to_string();
    
    // Skip . and .. entries
    if filename == "." || filename == ".." {
        return None;
    }

    // Use file system check for accurate type detection
    let file_path = dir_path.join(&filename);
    let file_type = get_file_type_robust(&file_path);

    match file_type {
        FileType::Directory => Some(format!("{{DIR}}{}{{/DIR}}", line)),
        FileType::Symlink => Some(format!("{{LINK}}{}{{/LINK}}", line)),
        FileType::Executable => Some(format!("{{EXEC}}{}{{/EXEC}}", line)),
        FileType::Regular => Some(format!("{{FILE}}{}{{/FILE}}", line)),
    }
}

fn format_tagged_line(line: &str) -> String {
    if line.contains("{DIR}") {
        let clean_line = line.replace("{DIR}", "").replace("{/DIR}", "");
        format!("📁 {}", clean_line.trim())
    } else if line.contains("{LINK}") {
        let clean_line = line.replace("{LINK}", "").replace("{/LINK}", "");
        format!("🔗 {}", clean_line.trim())
    } else if line.contains("{EXEC}") {
        let clean_line = line.replace("{EXEC}", "").replace("{/EXEC}", "");
        format!("⚙️ {}", clean_line.trim())
    } else if line.contains("{FILE}") {
        let clean_line = line.replace("{FILE}", "").replace("{/FILE}", "");
        
        // Add file type icons based on extension
        let icon = get_file_icon(&clean_line);
        format!("{} {}", icon, clean_line.trim())
    } else {
        line.to_string()
    }
}

fn get_file_icon(filename: &str) -> &'static str {
    let lower_name = filename.to_lowercase();
    
    if lower_name.ends_with(".jpg") || lower_name.ends_with(".jpeg") || 
       lower_name.ends_with(".png") || lower_name.ends_with(".gif") || 
       lower_name.ends_with(".bmp") || lower_name.ends_with(".svg") || 
       lower_name.ends_with(".webp") {
        "🖼️"
    } else if lower_name.ends_with(".pdf") || lower_name.ends_with(".doc") || 
              lower_name.ends_with(".docx") || lower_name.ends_with(".txt") || 
              lower_name.ends_with(".md") || lower_name.ends_with(".rtf") {
        "📝"
    } else if lower_name.ends_with(".zip") || lower_name.ends_with(".tar") || 
              lower_name.ends_with(".gz") || lower_name.ends_with(".rar") || 
              lower_name.ends_with(".7z") || lower_name.ends_with(".bz2") {
        "📦"
    } else if lower_name.ends_with(".mp3") || lower_name.ends_with(".wav") || 
              lower_name.ends_with(".flac") || lower_name.ends_with(".ogg") || 
              lower_name.ends_with(".m4a") {
        "🎵"
    } else if lower_name.ends_with(".mp4") || lower_name.ends_with(".avi") || 
              lower_name.ends_with(".mkv") || lower_name.ends_with(".mov") || 
              lower_name.ends_with(".wmv") || lower_name.ends_with(".webm") {
        "🎬"
    } else if lower_name.ends_with(".js") || lower_name.ends_with(".ts") || 
              lower_name.ends_with(".jsx") || lower_name.ends_with(".tsx") || 
              lower_name.ends_with(".html") || lower_name.ends_with(".css") || 
              lower_name.ends_with(".json") || lower_name.ends_with(".xml") {
        "💻"
    } else if lower_name.ends_with(".rs") || lower_name.ends_with(".py") || 
              lower_name.ends_with(".java") || lower_name.ends_with(".cpp") || 
              lower_name.ends_with(".c") || lower_name.ends_with(".h") {
        "⚡"
    } else {
        "📄"
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

// Additional utility functions for robust path handling
pub fn normalize_path(path: &str) -> String {
    // Handle different path separators and clean up the path
    let normalized = if cfg!(target_os = "windows") {
        path.replace('/', "\\")
    } else {
        path.replace('\\', "/")
    };
    
    // Remove redundant separators
    let separator = if cfg!(target_os = "windows") { "\\" } else { "/" };
    let double_sep = format!("{}{}", separator, separator);
    
    normalized.replace(&double_sep, separator)
}

pub fn expand_home_path(path: &str) -> Result<String, String> {
    if path.starts_with("~") {
        if let Ok(home) = std::env::var("HOME") {
            Ok(path.replacen("~", &home, 1))
        } else if cfg!(target_os = "windows") {
            if let Ok(home) = std::env::var("USERPROFILE") {
                Ok(path.replacen("~", &home, 1))
            } else {
                Err("Cannot determine home directory".to_string())
            }
        } else {
            Err("Cannot determine home directory".to_string())
        }
    } else {
        Ok(path.to_string())
    }
}

pub fn get_file_size_human_readable(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: u64 = 1024;

    if size == 0 {
        return "0 B".to_string();
    }

    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size_f /= THRESHOLD as f64;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}

pub fn format_permissions(mode: u32) -> String {
    #[cfg(unix)]
    {
        let mut perms = String::with_capacity(10);
        
        // File type
        perms.push(match mode & 0o170000 {
            0o040000 => 'd', // directory
            0o120000 => 'l', // symlink
            0o100000 => '-', // regular file
            _ => '?',
        });

        // Owner permissions
        perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });

        // Group permissions
        perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });

        // Other permissions
        perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });

        perms
    }
    
    #[cfg(not(unix))]
    {
        // Simplified for non-Unix systems
        "rwxrwxrwx".to_string()
    }