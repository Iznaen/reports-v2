pub mod models;
pub mod db;
pub mod commands;

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
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
            commands::tasks::delete_task,
            commands::report::get_monthly_report_data,
            commands::report::get_report_dashboard_data,
            commands::gen_pdf::generate_report_preview,
            commands::gen_pdf::export_report_pdf,
            commands::holidays::get_holidays,
            commands::holidays::add_holiday,
            commands::holidays::delete_holiday,
            commands::dev::get_setting,
            commands::dev::set_setting,
            commands::dev::dev_get_attendance_by_month,
            commands::dev::dev_upsert_attendance,
            commands::dev::dev_delete_attendance,
            commands::dev::dev_get_tasks_by_month,
            commands::dev::dev_upsert_task,
            commands::dev::dev_delete_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

