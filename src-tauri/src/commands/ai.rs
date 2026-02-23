use crate::commands::settings::get_default_provider;

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

    let os_name = if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unknown OS"
    };

    let system_prompt = format!(
        "You are AI running in terminal called Term, a lightweight terminal assistant. You are running on {os_name}. \
        Your job is to help users with their terminal commands and queries. \
        If you detect a misspelled command, suggest the correct one. \
        If the user asks for help, command explanation, or summarization, provide \
        concise, accurate information. \
        For technical questions, give short, practical answers focused on terminal usage. \
        If asked to run a destructive command, warn the user about potential consequences. \
        If asked for irrelevant information, politely tell that you can't help with that. \
        Keep responses brief, informative, and focused on helping the user accomplish their task."
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
