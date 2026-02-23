#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod commands;
pub mod utils;

fn main() {
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::shell::run_shell,
            commands::shell::run_sudo_command,
            commands::shell::get_current_dir,
            commands::shell::list_directory_contents,
            commands::shell::change_directory,
            commands::ai::ask_llm,
            commands::api_key::save_api_key,
            commands::api_key::get_api_key,
            commands::api_key::validate_api_key,
            commands::api_key::delete_api_key,
            commands::settings::save_ai_settings,
            commands::settings::get_ai_settings,
            commands::settings::get_default_provider,
            commands::settings::validate_provider_api_key,
            commands::settings::fetch_available_models
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
