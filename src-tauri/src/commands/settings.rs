use dirs;
use reqwest;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: String,
    pub model: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSettings {
    pub providers: HashMap<String, ProviderConfig>,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

fn get_settings_path(_app_handle: &AppHandle) -> PathBuf {
    let dir = dirs::config_dir()
        .expect("Failed to get config directory")
        .join("term");

    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create term directory");
    }

    dir.join("ai_settings.json")
}

#[tauri::command]
pub async fn save_ai_settings(
    app_handle: AppHandle,
    provider: String,
    api_key: String,
    model: String,
    is_default: bool,
) -> Result<(), String> {
    let path = get_settings_path(&app_handle);
    
    // Load existing settings
    let mut settings = if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        AiSettings::default()
    };

    // If this provider is being set as default, unset all others
    if is_default {
        for config in settings.providers.values_mut() {
            config.is_default = false;
        }
    }

    // Update or insert the provider config
    settings.providers.insert(
        provider.clone(),
        ProviderConfig {
            api_key,
            model,
            is_default,
        },
    );

    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_ai_settings(app_handle: AppHandle) -> Result<AiSettings, String> {
    let path = get_settings_path(&app_handle);

    if !path.exists() {
        return Ok(AiSettings::default());
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_default_provider(app_handle: AppHandle) -> Result<Option<(String, ProviderConfig)>, String> {
    let settings = get_ai_settings(app_handle).await?;
    
    for (provider, config) in settings.providers.iter() {
        if config.is_default {
            return Ok(Some((provider.clone(), config.clone())));
        }
    }
    
    Ok(None)
}

#[tauri::command]
pub async fn validate_provider_api_key(
    provider: String,
    api_key: String,
) -> Result<bool, String> {
    if api_key.is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    let client = reqwest::Client::new();

    match provider.as_str() {
        "openai" => {
            if !api_key.starts_with("sk-") {
                return Err("OpenAI API key must start with 'sk-'".to_string());
            }

            let res = client
                .get("https://api.openai.com/v1/models")
                .bearer_auth(&api_key)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if res.status().is_success() {
                Ok(true)
            } else {
                Err(format!("Invalid OpenAI API key: {}", res.status()))
            }
        }
        "anthropic" => {
            let res = client
                .get("https://api.anthropic.com/v1/models")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if res.status().is_success() {
                Ok(true)
            } else {
                Err(format!("Invalid Anthropic API key: {}", res.status()))
            }
        }
        "gemini" => {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                api_key
            );
            let res = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if res.status().is_success() {
                Ok(true)
            } else {
                Err(format!("Invalid Gemini API key: {}", res.status()))
            }
        }
        "groq" => {
            let res = client
                .get("https://api.groq.com/openai/v1/models")
                .bearer_auth(&api_key)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if res.status().is_success() {
                Ok(true)
            } else {
                Err(format!("Invalid Groq API key: {}", res.status()))
            }
        }
        "openrouter" => {
            let res = client
                .get("https://openrouter.ai/api/v1/models")
                .bearer_auth(&api_key)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if res.status().is_success() {
                Ok(true)
            } else {
                Err(format!("Invalid OpenRouter API key: {}", res.status()))
            }
        }
        _ => Err(format!("Unsupported provider: {}", provider)),
    }
}

#[tauri::command]
pub async fn fetch_available_models(
    provider: String,
    api_key: String,
) -> Result<Vec<ModelInfo>, String> {
    let client = reqwest::Client::new();

    match provider.as_str() {
        "openai" => {
            let res = client
                .get("https://api.openai.com/v1/models")
                .bearer_auth(&api_key)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
            let models: Vec<ModelInfo> = json["data"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|m| {
                    let id = m["id"].as_str()?;
                    // Filter for chat models
                    if id.contains("gpt") {
                        Some(ModelInfo {
                            id: id.to_string(),
                            name: id.to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            Ok(models)
        }
        "anthropic" => {
            // Anthropic doesn't have a models endpoint, return predefined list
            Ok(vec![
                ModelInfo {
                    id: "claude-3-5-sonnet-20241022".to_string(),
                    name: "Claude 3.5 Sonnet".to_string(),
                },
                ModelInfo {
                    id: "claude-3-5-haiku-20241022".to_string(),
                    name: "Claude 3.5 Haiku".to_string(),
                },
                ModelInfo {
                    id: "claude-3-opus-20240229".to_string(),
                    name: "Claude 3 Opus".to_string(),
                },
            ])
        }
        "gemini" => {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                api_key
            );
            let res = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
            let models: Vec<ModelInfo> = json["models"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|m| {
                    let name = m["name"].as_str()?;
                    // Extract model ID from name (e.g., "models/gemini-pro" -> "gemini-pro")
                    let id = name.strip_prefix("models/")?;
                    // Filter for generative models
                    if id.contains("gemini") && m["supportedGenerationMethods"]
                        .as_array()?
                        .iter()
                        .any(|method| method.as_str() == Some("generateContent"))
                    {
                        Some(ModelInfo {
                            id: id.to_string(),
                            name: id.to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            Ok(models)
        }
        "groq" => {
            let res = client
                .get("https://api.groq.com/openai/v1/models")
                .bearer_auth(&api_key)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
            let models: Vec<ModelInfo> = json["data"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|m| {
                    let id = m["id"].as_str()?;
                    Some(ModelInfo {
                        id: id.to_string(),
                        name: id.to_string(),
                    })
                })
                .collect();

            Ok(models)
        }
        "openrouter" => {
            let res = client
                .get("https://openrouter.ai/api/v1/models")
                .bearer_auth(&api_key)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
            let models: Vec<ModelInfo> = json["data"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|m| {
                    let id = m["id"].as_str()?;
                    let name = m["name"].as_str().unwrap_or(id);
                    Some(ModelInfo {
                        id: id.to_string(),
                        name: name.to_string(),
                    })
                })
                .collect();

            Ok(models)
        }
        _ => Err(format!("Unsupported provider: {}", provider)),
    }
}
