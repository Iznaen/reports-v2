pub mod queries;

use rusqlite::{Connection, Result};
use std::fs;
use tauri::{AppHandle, Manager};

pub fn init_db(app_handle: &AppHandle) -> Result<Connection, String> {
    // Get the private internal storage directory for the app
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let db_dir = app_data_dir.join("database");
    
    // Ensure the database directory exists
    if !db_dir.exists() {
        fs::create_dir_all(&db_dir)
            .map_err(|e| format!("Failed to create database directory: {}", e))?;
    }

    let db_path = db_dir.join("reports.db");
    
    // Open the SQLite connection
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Enable WAL mode for better concurrency and reliability on mobile
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;"
    ).map_err(|e| format!("Failed to set PRAGMA modes: {}", e))?;

    // Create the schema tables
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS employee_profile (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            ni TEXT NOT NULL,
            position TEXT NOT NULL,
            work_unit TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS office_locations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            radius_meters REAL NOT NULL DEFAULT 70.0,
            is_active INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS attendance_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL UNIQUE,
            clock_in_time TEXT,
            clock_in_lat REAL,
            clock_in_lng REAL,
            clock_in_photo TEXT,
            clock_in_flag TEXT,
            clock_out_time TEXT,
            clock_out_lat REAL,
            clock_out_lng REAL,
            clock_out_photo TEXT,
            clock_out_flag TEXT,
            status TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS task_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            attendance_id INTEGER NOT NULL,
            date TEXT NOT NULL,
            time TEXT NOT NULL,
            task_name TEXT NOT NULL,
            output TEXT NOT NULL,
            notes TEXT,
            photo_path TEXT,
            latitude REAL,
            longitude REAL,
            FOREIGN KEY(attendance_id) REFERENCES attendance_records(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS holidays_leaves (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL UNIQUE,
            type TEXT NOT NULL,
            description TEXT NOT NULL
        );
        "
    ).map_err(|e| format!("Failed to create database tables: {}", e))?;

    // Pre-populate with a default hardcoded office location if empty
    // YOU CAN EDIT THESE COORDINATES LATER
    let default_name = "Kantor Pusat";
    let default_lat = -6.200000; // Default Jakarta Latitude
    let default_lng = 106.816666; // Default Jakarta Longitude
    
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM office_locations",
        [],
        |row| row.get(0)
    ).unwrap_or(0);

    if count == 0 {
        conn.execute(
            "INSERT INTO office_locations (name, latitude, longitude, radius_meters, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (default_name, default_lat, default_lng, 70.0, 1)
        ).map_err(|e| format!("Failed to insert default location: {}", e))?;
    }

    Ok(conn)
}
