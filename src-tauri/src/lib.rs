pub mod models;
pub mod db;
pub mod commands;

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let conn = db::init_db(app.handle())
                .expect("Failed to initialize database");
            
            app.manage(Mutex::new(conn));
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::profile::save_profile,
            commands::profile::get_profile,
            commands::profile::save_signature,
            commands::profile::get_signature,
            commands::locations::get_locations,
            commands::locations::add_location,
            commands::locations::update_location_name,
            commands::locations::delete_location,
            commands::attendance::get_today_attendance,
            commands::attendance::clock_in,
            commands::attendance::clock_out,
            commands::tasks::get_today_tasks,
            commands::tasks::add_task,
            commands::tasks::delete_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
