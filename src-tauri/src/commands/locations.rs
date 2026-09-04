use tauri::{State, command};
use std::sync::Mutex;
use rusqlite::Connection;
use crate::models::OfficeLocation;

#[command]
pub async fn get_locations(db: State<'_, Mutex<Connection>>) -> Result<Vec<OfficeLocation>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, name, latitude, longitude, radius_meters, is_active FROM office_locations")
        .map_err(|e| e.to_string())?;
        
    let location_iter = stmt.query_map([], |row| {
        Ok(OfficeLocation {
            id: row.get(0)?,
            name: row.get(1)?,
            latitude: row.get(2)?,
            longitude: row.get(3)?,
            radius_meters: row.get(4)?,
            is_active: row.get::<_, i32>(5)? == 1,
        })
    }).map_err(|e| e.to_string())?;

    let mut locations = Vec::new();
    for loc in location_iter {
        locations.push(loc.map_err(|e| e.to_string())?);
    }
    
    Ok(locations)
}

#[command]
pub async fn add_location(location: OfficeLocation, db: State<'_, Mutex<Connection>>) -> Result<i64, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO office_locations (name, latitude, longitude, radius_meters, is_active)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (&location.name, &location.latitude, &location.longitude, &location.radius_meters, if location.is_active { 1 } else { 0 })
    ).map_err(|e| format!("Failed to insert location: {}", e))?;
    
    Ok(conn.last_insert_rowid())
}

#[command]
pub async fn update_location_name(id: i64, new_name: String, db: State<'_, Mutex<Connection>>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE office_locations SET name = ?1 WHERE id = ?2",
        (&new_name, &id)
    ).map_err(|e| format!("Failed to update location name: {}", e))?;
    
    Ok(())
}
