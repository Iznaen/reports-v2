use tauri::{State, command, AppHandle, Manager};
use std::sync::Mutex;
use rusqlite::{Connection, OptionalExtension};
use chrono::{Datelike, NaiveDate, Weekday};
use std::collections::HashMap;
use std::fs;
use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::models::{EmployeeProfile, TaskRecord};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyAttendance {
    pub no: u32,
    pub day_name: String,
    pub date_str: String,
    pub clock_in_time: Option<String>,
    pub clock_out_time: Option<String>,
    pub work_hours: String,
    pub status: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoItem {
    pub photo_type: String, // "Masuk", "Pulang", "Kegiatan"
    pub date_str: String,
    pub time_str: String,
    pub base64_data: String,
    pub caption: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyReportData {
    pub profile: Option<EmployeeProfile>,
    pub signature_base64: Option<String>,
    pub period_name: String, 
    pub attendance_days: Vec<DailyAttendance>,
    pub tasks: Vec<TaskRecord>,
    pub photos: Vec<PhotoItem>,
}

fn indo_weekday(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "Senin",
        Weekday::Tue => "Selasa",
        Weekday::Wed => "Rabu",
        Weekday::Thu => "Kamis",
        Weekday::Fri => "Jumat",
        Weekday::Sat => "Sabtu",
        Weekday::Sun => "Minggu",
    }
}

fn indo_month(month: u32) -> &'static str {
    match month {
        1 => "Januari", 2 => "Februari", 3 => "Maret", 4 => "April",
        5 => "Mei", 6 => "Juni", 7 => "Juli", 8 => "Agustus",
        9 => "September", 10 => "Oktober", 11 => "November", 12 => "Desember",
        _ => "",
    }
}



fn format_time(t: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(t) {
        dt.format("%H:%M").to_string()
    } else {
        t.chars().take(5).collect()
    }
}

fn get_days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    let d1 = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let d2 = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
    (d2 - d1).num_days() as u32
}

#[command]
pub async fn get_monthly_report_data(
    year: i32, 
    month: u32,
    app_handle: AppHandle,
    db: State<'_, Mutex<Connection>>
) -> Result<MonthlyReportData, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    
    // 1. Get Profile
    let profile: Option<EmployeeProfile> = conn.query_row(
        "SELECT id, name, ni, position, work_unit FROM employee_profile LIMIT 1",
        [],
        |row| Ok(EmployeeProfile {
            id: row.get(0)?,
            name: row.get(1)?,
            ni: row.get(2)?,
            position: row.get(3)?,
            work_unit: row.get(4)?,
        })
    ).optional().map_err(|e| e.to_string())?;

    // 2. Get Signature
    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let sig_path = app_data_dir.join("signatures").join("signature.png");
    let signature_base64 = if sig_path.exists() {
        if let Ok(bytes) = fs::read(&sig_path) {
            Some(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
        } else { None }
    } else { None };

    // 3. Fetch all attendance for the month
    let month_prefix = format!("{:04}-{:02}-%", year, month);
    let mut att_stmt = conn.prepare(
        "SELECT date, clock_in_time, clock_in_photo, 
                clock_out_time, clock_out_photo
         FROM attendance_records 
         WHERE date LIKE ?1"
    ).map_err(|e| e.to_string())?;

    struct RawAtt {
        date: String, in_time: Option<String>, in_photo: Option<String>,
        out_time: Option<String>, out_photo: Option<String>,
    }

    let raw_attendances = att_stmt.query_map([&month_prefix], |row| {
        Ok(RawAtt {
            date: row.get(0)?, in_time: row.get(1)?, in_photo: row.get(2)?,
            out_time: row.get(3)?, out_photo: row.get(4)?
        })
    }).map_err(|e| e.to_string())?.filter_map(Result::ok).collect::<Vec<_>>();

    let mut att_map = HashMap::new();
    for a in raw_attendances { att_map.insert(a.date.clone(), a); }

    // 4. Fetch all tasks for the month
    let mut task_stmt = conn.prepare(
        "SELECT id, attendance_id, date, time, task_name, output, notes, photo_path, latitude, longitude 
         FROM task_records 
         WHERE date LIKE ?1 
         ORDER BY date ASC, time ASC"
    ).map_err(|e| e.to_string())?;

    let tasks: Vec<TaskRecord> = task_stmt.query_map([&month_prefix], |row| {
        Ok(TaskRecord {
            id: row.get(0)?, attendance_id: row.get(1)?, date: row.get(2)?, time: row.get(3)?,
            task_name: row.get(4)?, output: row.get(5)?, notes: row.get(6)?,
            photo_path: row.get(7)?, latitude: row.get(8)?, longitude: row.get(9)?
        })
    }).map_err(|e| e.to_string())?.filter_map(Result::ok).collect();

    // 4b. Fetch holidays and leaves for the month
    let mut holiday_stmt = conn.prepare(
        "SELECT date, type, description FROM holidays_leaves WHERE date LIKE ?1"
    ).map_err(|e| e.to_string())?;

    let holidays: std::collections::HashMap<String, String> = holiday_stmt.query_map([&month_prefix], |row| {
        let date: String = row.get(0)?;
        let h_type: String = row.get(1)?;
        let desc: String = row.get(2)?;
        Ok((date, format!("{} - {}", h_type, desc)))
    }).map_err(|e| e.to_string())?.filter_map(Result::ok).collect();

    // 5. Build the Calendar Data (Days 1..N) and Extract Photos
    let num_days = get_days_in_month(year, month);
    let mut attendance_days = Vec::new();
    let mut photos = Vec::new();

    for day in 1..=num_days {
        let date_obj = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let date_key = format!("{:04}-{:02}-{:02}", year, month, day);
        let date_str_indo = format!("{} {} {}", day, indo_month(month), year);
        
        let wd = date_obj.weekday();
        let mut row_status = if let Some(hol) = holidays.get(&date_key) {
            hol.clone()
        } else if wd == Weekday::Sat || wd == Weekday::Sun {
            "Libur Akhir Pekan".to_string()
        } else {
            "Alpha".to_string()
        };
        let mut in_t = None; let mut out_t = None;
        let mut work_hours = "—".to_string();

        if let Some(att) = att_map.get(&date_key) {
            if att.in_time.is_some() && att.out_time.is_some() {
                row_status = "Hadir".to_string();
            } else if att.in_time.is_some() || att.out_time.is_some() {
                row_status = "Parsial".to_string();
            }
            
            in_t = att.in_time.as_ref().map(|t| format_time(t));
            out_t = att.out_time.as_ref().map(|t| format_time(t));
            
            if let (Some(in_str), Some(out_str)) = (&att.in_time, &att.out_time) {
                if let (Ok(in_dt), Ok(out_dt)) = (chrono::DateTime::parse_from_rfc3339(in_str), chrono::DateTime::parse_from_rfc3339(out_str)) {
                    let diff = out_dt.signed_duration_since(in_dt);
                    let hours = diff.num_hours();
                    let minutes = diff.num_minutes() % 60;
                    work_hours = format!("{:02}:{:02}", hours, minutes);
                }
            }

            // Extract Photos
            let full_date_str = format!("{} {} {}", day, indo_month(month), year);

            if let (Some(b64), Some(time)) = (&att.in_photo, &att.in_time) {
                let short_time = format_time(time);
                photos.push(PhotoItem {
                    photo_type: "Masuk".to_string(),
                    date_str: full_date_str.clone(),
                    time_str: short_time.clone(),
                    base64_data: b64.clone(),
                    caption: format!("Presensi Masuk – {}", short_time),
                });
            }
            if let (Some(b64), Some(time)) = (&att.out_photo, &att.out_time) {
                let short_time = format_time(time);
                photos.push(PhotoItem {
                    photo_type: "Pulang".to_string(),
                    date_str: full_date_str.clone(),
                    time_str: short_time.clone(),
                    base64_data: b64.clone(),
                    caption: format!("Presensi Pulang – {}", short_time),
                });
            }
        } else {
            // TODO: In the future, check a Holiday/Leave table here to override "Alpha" -> "Cuti/Sakit"
        }

        attendance_days.push(DailyAttendance {
            no: day,
            day_name: indo_weekday(date_obj.weekday()).to_string(),
            date_str: date_str_indo.clone(),
            clock_in_time: in_t,
            clock_out_time: out_t,
            work_hours: work_hours,
            status: row_status,
        });
    }

    // 6. Extract Task Photos
    for t in &tasks {
        if let Some(b64) = &t.photo_path {
            let short_time = t.time.chars().take(5).collect::<String>();
            let (y, m, d) = (
                t.date[0..4].parse::<i32>().unwrap_or(year),
                t.date[5..7].parse::<u32>().unwrap_or(month),
                t.date[8..10].parse::<u32>().unwrap_or(1)
            );
            let full_date_str = format!("{} {} {}", d, indo_month(m), y);

            photos.push(PhotoItem {
                photo_type: "Kegiatan".to_string(),
                date_str: full_date_str,
                time_str: short_time.clone(),
                base64_data: b64.clone(),
                caption: format!("{} – {}", t.task_name.chars().take(30).collect::<String>(), short_time),
            });
        }
    }

    Ok(MonthlyReportData {
        profile,
        signature_base64,
        period_name: format!("{} {}", indo_month(month), year),
        attendance_days,
        tasks,
        photos,
    })
}


#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthSummary {
    pub year: i32,
    pub month: u32,
    pub period_name: String,
    pub total_hadir: u32,
    pub total_parsial: u32,
    pub total_alpha: u32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardData {
    pub profile: Option<EmployeeProfile>,
    pub months: Vec<MonthSummary>,
}

#[command]
pub async fn get_report_dashboard_data(
    db: State<'_, Mutex<Connection>>
) -> Result<DashboardData, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    
    let profile: Option<EmployeeProfile> = conn.query_row(
        "SELECT id, name, ni, position, work_unit FROM employee_profile LIMIT 1",
        [],
        |row| Ok(EmployeeProfile {
            id: row.get(0)?,
            name: row.get(1)?,
            ni: row.get(2)?,
            position: row.get(3)?,
            work_unit: row.get(4)?,
        })
    ).optional().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT strftime('%Y-%m', date) as ym FROM attendance_records GROUP BY ym ORDER BY ym DESC LIMIT 12").unwrap();
    let yms: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap().filter_map(Result::ok).collect();

    let mut months_summary = Vec::new();

    for ym in yms {
        let parts: Vec<&str> = ym.split('-').collect();
        if parts.len() != 2 { continue; }
        let year: i32 = parts[0].parse().unwrap_or(0);
        let month: u32 = parts[1].parse().unwrap_or(0);

        let month_prefix = format!("{}-%", ym);
        let mut att_stmt = conn.prepare(
            "SELECT date, clock_in_time, clock_out_time FROM attendance_records WHERE date LIKE ?1"
        ).unwrap();
        
        struct BasicAtt { date: String, in_time: Option<String>, out_time: Option<String> }
        let atts: Vec<BasicAtt> = att_stmt.query_map([&month_prefix], |row| {
            Ok(BasicAtt { date: row.get(0)?, in_time: row.get(1)?, out_time: row.get(2)? })
        }).unwrap().filter_map(Result::ok).collect();

        let mut att_map = HashMap::new();
        for a in atts { att_map.insert(a.date.clone(), a); }

        let mut total_hadir = 0;
        let mut total_parsial = 0;
        let mut total_alpha = 0;

        let num_days = get_days_in_month(year, month);
        for day in 1..=num_days {
            let date_obj = NaiveDate::from_ymd_opt(year, month, day).unwrap();
            let date_key = format!("{:04}-{:02}-{:02}", year, month, day);

            if let Some(att) = att_map.get(&date_key) {
                if att.in_time.is_some() && att.out_time.is_some() {
                    total_hadir += 1;
                } else {
                    total_parsial += 1;
                }
            } else {
                let wd = date_obj.weekday();
                if wd != Weekday::Sat && wd != Weekday::Sun {
                    // It's a weekday and no record exists -> Alpha
                    total_alpha += 1;
                }
            }
        }

        months_summary.push(MonthSummary {
            year,
            month,
            period_name: format!("{} {}", indo_month(month), year),
            total_hadir,
            total_parsial,
            total_alpha
        });
    }

    Ok(DashboardData {
        profile,
        months: months_summary,
    })
}
