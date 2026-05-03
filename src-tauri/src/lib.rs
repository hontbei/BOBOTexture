mod config;
pub mod error;
mod ipc;
mod log;
pub mod tools;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_state = config::AppState::bootstrap(app.handle())?;
            app.manage(app_state);

            #[cfg(debug_assertions)]
            if let Some(main_window) = app.get_webview_window("main") {
                main_window.open_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::system::bootstrap,
            ipc::system::save_settings,
            ipc::system::export_logs,
            ipc::system::collect_files,
            ipc::system::open_in_explorer,
            ipc::system::read_text_file,
            ipc::system::write_text_file,
            ipc::system::path_exists,
            ipc::atlaspro::scan_atlaspro_inputs,
            ipc::atlaspro::execute_atlaspro,
        ])
        .run(tauri::generate_context!())
        .expect("error while running bobotexture-v2")
}
