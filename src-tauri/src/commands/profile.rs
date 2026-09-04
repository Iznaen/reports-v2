use tauri::{State, command};
use std::sync::Mutex;
use rusqlite::{Connection, OptionalExtension};
use crate::models::EmployeeProfile;

#[command]
pub fn save_profile(profile: EmployeeProfile, db: State<'_, Mutex<Connection>>) -> Result<(), String> {
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
pub fn get_profile(db: State<'_, Mutex<Connection>>) -> Result<Option<EmployeeProfile>, String> {
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
