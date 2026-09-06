use tauri::{State, command};
use std::sync::Mutex;
use rusqlite::{Connection, OptionalExtension};
use chrono::{Local, NaiveTime, Duration, DateTime};
use crate::models::AttendanceRecord;

// Helper to get the "shift date" string (YYYY-MM-DD).
// If it's before 02:00 AM, it counts as the previous day.
fn get_current_shift_date() -> String {
    let now = Local::now();
    let cutoff = NaiveTime::from_hms_opt(2, 0, 0).unwrap();
    let mut date = now.date_naive();
    if now.time() < cutoff {
        date -= Duration::days(1);
    }
    date.format("%Y-%m-%d").to_string()
}

#[command]
pub async fn get_today_attendance(db: State<'_, Mutex<Connection>>) -> Result<Option<AttendanceRecord>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let shift_date = get_current_shift_date();
    
    let mut stmt = conn.prepare("SELECT id, date, clock_in_time, clock_in_lat, clock_in_lng, clock_in_photo, clock_in_flag, clock_out_time, clock_out_lat, clock_out_lng, clock_out_photo, clock_out_flag, status FROM attendance_records WHERE date = ?1").map_err(|e| e.to_string())?;
    
    let record = stmt.query_row([&shift_date], |row| {
        Ok(AttendanceRecord {
            id: row.get(0)?,
            date: row.get(1)?,
            clock_in_time: row.get(2)?,
            clock_in_lat: row.get(3)?,
            clock_in_lng: row.get(4)?,
            clock_in_photo: row.get(5)?,
            clock_in_flag: row.get(6)?,
            clock_out_time: row.get(7)?,
            clock_out_lat: row.get(8)?,
            clock_out_lng: row.get(9)?,
            clock_out_photo: row.get(10)?,
            clock_out_flag: row.get(11)?,
            status: row.get(12)?,
        })
    }).optional().map_err(|e| e.to_string())?;
    
    Ok(record)
}

#[command]
pub async fn clock_in(photo: String, lat: f64, lng: f64, db: State<'_, Mutex<Connection>>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let shift_date = get_current_shift_date();
    let now_str = Local::now().to_rfc3339();
    
    conn.execute(
        "INSERT INTO attendance_records (date, clock_in_time, clock_in_photo, clock_in_lat, clock_in_lng, status) VALUES (?1, ?2, ?3, ?4, ?5, 'Masuk')",
        (&shift_date, &now_str, &photo, &lat, &lng)
    ).map_err(|e| format!("Failed to clock in: {}", e))?;
    
    Ok(())
}

#[command]
pub async fn clock_out(photo: String, lat: f64, lng: f64, db: State<'_, Mutex<Connection>>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let shift_date = get_current_shift_date();
    let now = Local::now();
    let now_str = now.to_rfc3339();
    
    // Validate 8 hours
    let clock_in_time_str: Option<String> = conn.query_row(
        "SELECT clock_in_time FROM attendance_records WHERE date = ?1",
        [&shift_date],
        |row| row.get(0)
    ).optional().map_err(|e| format!("Error checking record: {}", e))?
     .flatten();
     
    if let Some(time_str) = clock_in_time_str {
        if let Ok(in_time) = DateTime::parse_from_rfc3339(&time_str) {
            let duration = now.signed_duration_since(in_time.with_timezone(&Local));
            
            // Read minimum work duration in seconds from app_settings, default to 8 hours (28800 seconds)
            let min_seconds: i64 = conn.query_row(
                "SELECT value FROM app_settings WHERE key = 'min_work_seconds'",
                [],
                |row| row.get::<_, String>(0),
            ).optional().unwrap_or(None)
             .and_then(|v| v.parse::<i64>().ok())
             .unwrap_or(28800);
            
            if duration.num_seconds() < min_seconds {
                let remaining_seconds = min_seconds - duration.num_seconds();
                let remaining_hours = remaining_seconds / 3600;
                let remaining_mins = (remaining_seconds % 3600) / 60;
                let remaining_secs = remaining_seconds % 60;
                
                return Err(format!(
                    "Belum mencapai durasi kerja minimal. Tersisa {} jam {} menit {} detik.",
                    remaining_hours, remaining_mins, remaining_secs
                ));
            }
        }
    } else {
        return Err("Belum melakukan absen masuk hari ini.".to_string());
    }

    conn.execute(
        "UPDATE attendance_records SET clock_out_time = ?1, clock_out_photo = ?2, clock_out_lat = ?3, clock_out_lng = ?4, status = 'Selesai' WHERE date = ?5",
        (&now_str, &photo, &lat, &lng, &shift_date)
    ).map_err(|e| format!("Failed to clock out: {}", e))?;
    
    Ok(())
}
