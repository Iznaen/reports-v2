use tauri::{State, command};
use std::sync::Mutex;
use rusqlite::{Connection, OptionalExtension};
use crate::models::{AttendanceRecord, TaskRecord};

// ── App Settings ─────────────────────────────────────────────────────────────

#[command]
pub async fn get_setting(key: String, db: State<'_, Mutex<Connection>>) -> Result<Option<String>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let val: Option<String> = conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        [&key],
        |row| row.get(0),
    ).optional().map_err(|e| e.to_string())?;
    Ok(val)
}

#[command]
pub async fn set_setting(key: String, value: String, db: State<'_, Mutex<Connection>>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        (&key, &value),
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// ── Attendance CRUD ───────────────────────────────────────────────────────────

/// Get attendance records for a specific year-month (YYYY-MM pattern).
#[command]
pub async fn dev_get_attendance_by_month(
    year: i32,
    month: u32,
    db: State<'_, Mutex<Connection>>,
) -> Result<Vec<AttendanceRecord>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let pattern = format!("{:04}-{:02}-%", year, month);
    let mut stmt = conn.prepare(
        "SELECT id, date, clock_in_time, clock_in_lat, clock_in_lng, clock_in_photo,
                clock_in_flag, clock_out_time, clock_out_lat, clock_out_lng,
                clock_out_photo, clock_out_flag, status
         FROM attendance_records
         WHERE date LIKE ?1
         ORDER BY date ASC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([&pattern], |row| {
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
    }).map_err(|e| e.to_string())?
      .filter_map(Result::ok)
      .collect();
    Ok(rows)
}

/// Create or update an attendance record. Returns the record id.
#[command]
pub async fn dev_upsert_attendance(
    id: Option<i64>,
    date: String,
    clock_in_time: Option<String>,
    clock_out_time: Option<String>,
    clock_in_photo: Option<String>,
    clock_out_photo: Option<String>,
    status: String,
    db: State<'_, Mutex<Connection>>,
) -> Result<i64, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    if let Some(existing_id) = id {
        conn.execute(
            "UPDATE attendance_records
             SET clock_in_time = ?1, clock_out_time = ?2, status = ?3,
                 clock_in_photo = COALESCE(?4, clock_in_photo),
                 clock_out_photo = COALESCE(?5, clock_out_photo)
             WHERE id = ?6",
            (&clock_in_time, &clock_out_time, &status, &clock_in_photo, &clock_out_photo, existing_id),
        ).map_err(|e| format!("Gagal update absensi: {}", e))?;
        Ok(existing_id)
    } else {
        conn.execute(
            "INSERT OR REPLACE INTO attendance_records
             (date, clock_in_time, clock_out_time, clock_in_photo, clock_out_photo, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&date, &clock_in_time, &clock_out_time, &clock_in_photo, &clock_out_photo, &status),
        ).map_err(|e| format!("Gagal buat absensi: {}", e))?;
        Ok(conn.last_insert_rowid())
    }
}

/// Delete an attendance record (and its tasks via CASCADE) by id.
#[command]
pub async fn dev_delete_attendance(id: i64, db: State<'_, Mutex<Connection>>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM attendance_records WHERE id = ?1", [id])
        .map_err(|e| format!("Gagal menghapus absensi: {}", e))?;
    Ok(())
}

// ── Task CRUD ─────────────────────────────────────────────────────────────────

/// Get task records for a specific year-month.
#[command]
pub async fn dev_get_tasks_by_month(
    year: i32,
    month: u32,
    db: State<'_, Mutex<Connection>>,
) -> Result<Vec<TaskRecord>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let pattern = format!("{:04}-{:02}-%", year, month);
    let mut stmt = conn.prepare(
        "SELECT id, attendance_id, date, time, task_name, output, notes, photo_path, latitude, longitude
         FROM task_records
         WHERE date LIKE ?1
         ORDER BY date ASC, time ASC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([&pattern], |row| {
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
    Ok(rows)
}

/// Create or update a task record.
/// When creating (id = None), auto-creates or re-uses the attendance record for the given date.
#[command]
pub async fn dev_upsert_task(
    id: Option<i64>,
    date: String,
    time: String,
    task_name: String,
    output: String,
    notes: Option<String>,
    photo_path: Option<String>,
    db: State<'_, Mutex<Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    if let Some(existing_id) = id {
        conn.execute(
            "UPDATE task_records
             SET date = ?1, time = ?2, task_name = ?3, output = ?4, notes = ?5,
                 photo_path = COALESCE(?6, photo_path)
             WHERE id = ?7",
            (&date, &time, &task_name, &output, &notes, &photo_path, existing_id),
        ).map_err(|e| format!("Gagal update kegiatan: {}", e))?;
    } else {
        // Get or create attendance record for the given date
        let att_id: Option<i64> = conn.query_row(
            "SELECT id FROM attendance_records WHERE date = ?1",
            [&date],
            |row| row.get(0),
        ).optional().map_err(|e| e.to_string())?;

        let att_id = match att_id {
            Some(id) => id,
            None => {
                conn.execute(
                    "INSERT INTO attendance_records (date, status) VALUES (?1, 'Masuk')",
                    [&date],
                ).map_err(|e| format!("Gagal buat record absensi untuk tanggal ini: {}", e))?;
                conn.last_insert_rowid()
            }
        };

        conn.execute(
            "INSERT INTO task_records (attendance_id, date, time, task_name, output, notes, photo_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (att_id, &date, &time, &task_name, &output, &notes, &photo_path),
        ).map_err(|e| format!("Gagal buat kegiatan: {}", e))?;
    }
    Ok(())
}

/// Delete a single task by id.
#[command]
pub async fn dev_delete_task(id: i64, db: State<'_, Mutex<Connection>>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM task_records WHERE id = ?1", [id])
        .map_err(|e| format!("Gagal menghapus kegiatan: {}", e))?;
    Ok(())
}
