use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployeeProfile {
    pub id: i64,
    pub name: String,
    pub ni: String,
    pub position: String,
    pub work_unit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfficeLocation {
    pub id: i64,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: f64,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttendanceRecord {
    pub id: i64,
    pub date: String,
    pub clock_in_time: Option<String>,
    pub clock_in_lat: Option<f64>,
    pub clock_in_lng: Option<f64>,
    pub clock_in_photo: Option<String>,
    pub clock_in_flag: Option<String>,
    pub clock_out_time: Option<String>,
    pub clock_out_lat: Option<f64>,
    pub clock_out_lng: Option<f64>,
    pub clock_out_photo: Option<String>,
    pub clock_out_flag: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: i64,
    pub attendance_id: i64,
    pub date: String,
    pub time: String,
    pub task_name: String,
    pub output: String,
    pub notes: Option<String>,
    pub photo_path: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}
