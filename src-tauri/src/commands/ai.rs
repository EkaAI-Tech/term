use crate::commands::settings::get_default_provider;
use std::fs;

fn get_os_name() -> String {
    #[cfg(target_os = "windows")]
    {
        return "Windows".to_string();
    }

    #[cfg(target_os = "macos")]
    {
        return "macOS".to_string();
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = fs::read_to_string("/etc/os-release") {
            for line in contents.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    return line
                        .trim_start_matches("PRETTY_NAME=")
                        .trim_matches('"')
                        .to_string();
                }
            }
        }
        return "Linux (Unknown Distro)".to_string();
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "Unknown OS".to_string()
    }
}

#[tauri::command]
pub async fn ask_llm(prompt: String, app_handle: tauri::AppHandle) -> Result<String, String> {
    let default_provider = get_default_provider(app_handle).await?;

    let (provider, config) = match default_provider {
        Some((p, c)) => (p, c),
        None => return Err("No default AI provider configured. Please configure your AI provider in settings.".to_string()),
    };

    if config.api_key.is_empty() {
        return Err("API key not configured. Please configure your AI provider in settings.".to_string());
    }

    let os_name = get_os_name();

    let system_prompt = format!(
        "You are Term, a lightweight AI terminal assistant running on {}.
        Your purpose is to help users with terminal-related tasks, commands, and shell workflows.
        Rules:
        - Provide concise, practical answers focused strictly on terminal usage.
        - When suggesting commands, include comments INSIDE the code block (using # for shell comments) to explain their purpose.
        - Use single backticks for inline commands and triple backticks for multi-line commands or scripts.
        - Comments must be on the same line as the command or within the same code block, prefixed with # (e.g., `command # explanation`).
        - Do not add unnecessary blank lines or verbose explanations.
        - If a command appears misspelled, suggest the correct command.
        - If a command is potentially destructive (data loss, system modification, privilege escalation, disk overwrite, recursive deletion, formatting, etc.), place the warning OUTSIDE code block as regular text, then show the command in a separate code block.
        - Never fabricate command output.
        - If unsure about a command or flag, say so instead of guessing.
        - If the request is unrelated to terminal usage, respond briefly that this assistant only handles terminal-related queries.
        - Do not use decorative bullets, extra newlines, emojis or decorative formatting.
        - Keep responses compact and terminal-appropriate.",
        os_name
    );

    match provider.as_str() {
        "openai" => call_openai(&config, &system_prompt, &prompt).await,
        "anthropic" => call_anthropic(&config, &system_prompt, &prompt).await,
        "gemini" => call_gemini(&config, &system_prompt, &prompt).await,
        "groq" => call_groq(&config, &system_prompt, &prompt).await,
        "openrouter" => call_openrouter(&config, &system_prompt, &prompt).await,
        _ => Err(format!("Unsupported provider: {}", provider)),
    }
}

async fn call_openai(
    config: &crate::commands::settings::ProviderConfig,
    system_prompt: &str,
    prompt: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&config.api_key)
        .json(&serde_json::json!({
            "model": config.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": prompt }
            ]
        }))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No response")
        .to_string())
}

async fn call_anthropic(
    config: &crate::commands::settings::ProviderConfig,
    system_prompt: &str,
    prompt: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &config.api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": config.model,
            "max_tokens": 1024,
            "system": system_prompt,
            "messages": [
                { "role": "user", "content": prompt }
            ]
        }))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(json["content"][0]["text"]
        .as_str()
        .unwrap_or("No response")
        .to_string())
}

async fn call_gemini(
    config: &crate::commands::settings::ProviderConfig,
    system_prompt: &str,
    prompt: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        config.model, config.api_key
    );

    let combined_prompt = format!("{}\n\nUser: {}", system_prompt, prompt);

    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": combined_prompt
                }]
            }]
        }))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("No response")
        .to_string())
}

async fn call_groq(
    config: &crate::commands::settings::ProviderConfig,
    system_prompt: &str,
    prompt: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .bearer_auth(&config.api_key)
        .json(&serde_json::json!({
            "model": config.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": prompt }
            ]
        }))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No response")
        .to_string())
}

async fn call_openrouter(
    config: &crate::commands::settings::ProviderConfig,
    system_prompt: &str,
    prompt: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(&config.api_key)
        .json(&serde_json::json!({
            "model": config.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": prompt }
            ]
        }))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No response")
        .to_string())
}
