use tauri::{State, command};
use std::sync::Mutex;
use rusqlite::{Connection, OptionalExtension};
use crate::models::EmployeeProfile;

#[command]
pub async fn save_profile(profile: EmployeeProfile, db: State<'_, Mutex<Connection>>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    
    // We only store one profile, assume id=1
    conn.execute(
        "INSERT INTO employee_profile (id, name, ni, position, work_unit) 
         VALUES (1, ?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET 
         name=excluded.name, ni=excluded.ni, position=excluded.position, work_unit=excluded.work_unit",
        (&profile.name, &profile.ni, &profile.position, &profile.work_unit)
    ).map_err(|e| format!("Failed to save profile: {}", e))?;
    
    Ok(())
}

#[command]
pub async fn get_profile(db: State<'_, Mutex<Connection>>) -> Result<Option<EmployeeProfile>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, name, ni, position, work_unit FROM employee_profile WHERE id=1")
        .map_err(|e| e.to_string())?;
    
    let profile = stmt.query_row([], |row| {
        Ok(EmployeeProfile {
            id: row.get(0)?,
            name: row.get(1)?,
            ni: row.get(2)?,
            position: row.get(3)?,
            work_unit: row.get(4)?,
        })
    }).optional().map_err(|e| format!("Failed to fetch profile: {}", e))?;

    Ok(profile)
}

#[command]
pub async fn save_signature(app: tauri::AppHandle, base64_data: String) -> Result<(), String> {
    use tauri::Manager;
    use std::fs;
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    // Strip the "data:image/png;base64," prefix if it exists
    let b64 = if let Some(stripped) = base64_data.strip_prefix("data:image/png;base64,") {
        stripped
    } else {
        &base64_data
    };

    let decoded = STANDARD.decode(b64).map_err(|e| format!("Invalid base64: {}", e))?;

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let sig_dir = app_data_dir.join("signatures");
    
    if !sig_dir.exists() {
        fs::create_dir_all(&sig_dir).map_err(|e| e.to_string())?;
    }

    let sig_file = sig_dir.join("signature.png");
    fs::write(&sig_file, decoded).map_err(|e| format!("Failed to write signature file: {}", e))?;

    Ok(())
}

#[command]
pub async fn get_signature(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri::Manager;
    use std::fs;
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let sig_file = app_data_dir.join("signatures").join("signature.png");
    
    if sig_file.exists() {
        let bytes = fs::read(&sig_file).map_err(|e| format!("Failed to read signature: {}", e))?;
        let b64 = STANDARD.encode(&bytes);
        Ok(Some(format!("data:image/png;base64,{}", b64)))
    } else {
        Ok(None)
    }
}
