use rusqlite::Connection;
use std::sync::{Mutex};
use tauri::State;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HolidayRecord {
    pub id: i64,
    pub date: String,
    pub h_type: String, // "Cuti" or "Libur"
    pub description: String,
}

#[tauri::command]
pub async fn get_holidays(db: State<'_, Mutex<Connection>>) -> Result<Vec<HolidayRecord>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT id, date, type, description FROM holidays_leaves ORDER BY date ASC")
        .map_err(|e| e.to_string())?;
        
    let rows = stmt.query_map([], |row| {
        Ok(HolidayRecord {
            id: row.get(0)?,
            date: row.get(1)?,
            h_type: row.get(2)?,
            description: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut holidays = Vec::new();
    for row in rows {
        holidays.push(row.map_err(|e| e.to_string())?);
    }
    
    Ok(holidays)
}

#[tauri::command]
pub async fn add_holiday(
    date: String,
    h_type: String,
    description: String,
    db: State<'_, Mutex<Connection>>
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    
    // Use REPLACE to update if the date already exists
    conn.execute(
        "INSERT OR REPLACE INTO holidays_leaves (date, type, description) VALUES (?1, ?2, ?3)",
        (&date, &h_type, &description),
    ).map_err(|e| format!("Gagal menyimpan data: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn delete_holiday(id: i64, db: State<'_, Mutex<Connection>>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    
    conn.execute(
        "DELETE FROM holidays_leaves WHERE id = ?1",
        [id],
    ).map_err(|e| format!("Gagal menghapus data: {}", e))?;
    
    Ok(())
}
