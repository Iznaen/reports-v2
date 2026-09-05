use tauri::{State, command};
use std::sync::Mutex;
use rusqlite::{Connection, OptionalExtension};
use chrono::{Local, NaiveTime, Duration};
use crate::models::{TaskRecord};

// Helper to get the "shift date" string (YYYY-MM-DD).
pub fn get_current_shift_date() -> String {
    let now = Local::now();
    let cutoff = NaiveTime::from_hms_opt(2, 0, 0).unwrap();
    let mut date = now.date_naive();
    if now.time() < cutoff {
        date -= Duration::days(1);
    }
    date.format("%Y-%m-%d").to_string()
}

#[command]
pub async fn get_today_tasks(db: State<'_, Mutex<Connection>>) -> Result<Vec<TaskRecord>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let shift_date = get_current_shift_date();
    
    let mut stmt = conn.prepare(
        "SELECT id, attendance_id, date, time, task_name, output, notes, photo_path, latitude, longitude 
         FROM task_records 
         WHERE date = ?1 
         ORDER BY time ASC"
    ).map_err(|e| e.to_string())?;
    
    let tasks = stmt.query_map([&shift_date], |row| {
        Ok(TaskRecord {
            id: row.get(0)?,
            attendance_id: row.get(1)?,
            date: row.get(2)?,
            time: row.get(3)?,
            task_name: row.get(4)?,
            output: row.get(5)?,
            notes: row.get(6)?,
            photo_path: row.get(7)?,
            latitude: row.get(8)?,
            longitude: row.get(9)?,
        })
    }).map_err(|e| e.to_string())?
      .filter_map(Result::ok)
      .collect();
      
    Ok(tasks)
}

#[command]
pub async fn add_task(
    task_name: String, 
    output: String, 
    notes: Option<String>, 
    photo_path: Option<String>, 
    latitude: Option<f64>, 
    longitude: Option<f64>, 
    db: State<'_, Mutex<Connection>>
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let shift_date = get_current_shift_date();
    
    // Check if the user has clocked in today
    let attendance_id: Option<i64> = conn.query_row(
        "SELECT id FROM attendance_records WHERE date = ?1",
        [&shift_date],
        |row| row.get(0)
    ).optional().map_err(|e| format!("Database error: {}", e))?;
    
    let att_id = match attendance_id {
        Some(id) => id,
        None => return Err("Anda harus Absen Masuk terlebih dahulu sebelum mencatat kegiatan.".to_string()),
    };
    
    let now = Local::now();
    let time_str = now.format("%H:%M:%S").to_string();
    
    conn.execute(
        "INSERT INTO task_records (attendance_id, date, time, task_name, output, notes, photo_path, latitude, longitude) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        (
            att_id, 
            &shift_date, 
            &time_str, 
            &task_name, 
            &output, 
            &notes, 
            &photo_path, 
            &latitude, 
            &longitude
        )
    ).map_err(|e| format!("Gagal menyimpan kegiatan: {}", e))?;
    
    Ok(())
}

#[command]
pub async fn delete_task(id: i64, db: State<'_, Mutex<Connection>>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    
    let affected = conn.execute(
        "DELETE FROM task_records WHERE id = ?1",
        [id]
    ).map_err(|e| format!("Gagal menghapus kegiatan: {}", e))?;
    
    if affected == 0 {
        return Err("Kegiatan tidak ditemukan.".to_string());
    }
    
    Ok(())
}
