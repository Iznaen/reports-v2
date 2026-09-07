use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{to_value, from_value};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Date;
use crate::app::DevMode;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct AttendanceRecord {
    id: i64,
    date: String,
    clock_in_time: Option<String>,
    clock_out_time: Option<String>,
    status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct TaskRecord {
    id: i64,
    attendance_id: i64,
    date: String,
    time: String,
    task_name: String,
    output: String,
    notes: Option<String>,
    photo_path: Option<String>,
}

#[component]
pub fn Dev() -> impl IntoView {
    let dev_mode = use_context::<DevMode>().expect("DevMode context missing");

    let not_dev = move || !dev_mode.0.get();
    let (active_tab, set_active_tab) = signal("attendance".to_string());

    // --- Filter State ---
    let current_date = Date::new_0();
    let (filter_year, set_filter_year) = signal(current_date.get_full_year() as i32);
    let (filter_month, set_filter_month) = signal(current_date.get_month() + 1); // JS months are 0-indexed

    // --- Data Signals ---
    let (attendance_list, set_attendance_list) = signal(Vec::<AttendanceRecord>::new());
    let (tasks_list, set_tasks_list) = signal(Vec::<TaskRecord>::new());
    
    // Config Signals
    let (cfg_hours, set_cfg_hours) = signal(8i64);
    let (cfg_minutes, set_cfg_minutes) = signal(0i64);
    let (cfg_seconds, set_cfg_seconds) = signal(0i64);
    let (save_msg, set_save_msg) = signal(String::new());

    // Modals
    let (photo_preview, set_photo_preview) = signal::<Option<String>>(None);
    
    // Attendance Edit Modal
    let (show_att_modal, set_show_att_modal) = signal(false);
    let (edit_att, set_edit_att) = signal(AttendanceRecord::default());
    let (edit_att_is_new, set_edit_att_is_new) = signal(false);
    let (att_in_str, set_att_in_str) = signal(String::new());
    let (att_out_str, set_att_out_str) = signal(String::new());
    let (att_photo_in, set_att_photo_in) = signal::<Option<String>>(None);   // base64
    let (att_photo_out, set_att_photo_out) = signal::<Option<String>>(None);
    let (att_photo_in_loading, set_att_photo_in_loading) = signal(false);
    let (att_photo_out_loading, set_att_photo_out_loading) = signal(false);

    // Task Edit Modal
    let (show_task_modal, set_show_task_modal) = signal(false);
    let (edit_task, set_edit_task) = signal(TaskRecord::default());
    let (edit_task_is_new, set_edit_task_is_new) = signal(false);
    let (task_time_str, set_task_time_str) = signal(String::new());
    let (task_photo, set_task_photo) = signal::<Option<String>>(None);       // base64
    let (task_photo_loading, set_task_photo_loading) = signal(false);

    // --- Actions ---

    let load_attendance = move || {
        let y = filter_year.get_untracked();
        let m = filter_month.get_untracked();
        wasm_bindgen_futures::spawn_local(async move {
            #[derive(Serialize)]
            struct Args { year: i32, month: u32 }
            let args = to_value(&Args { year: y, month: m }).unwrap();
            if let Ok(res) = invoke("dev_get_attendance_by_month", args).await.dyn_into::<JsValue>() {
                if let Ok(v) = from_value::<Vec<AttendanceRecord>>(res) {
                    set_attendance_list.set(v);
                }
            }
        });
    };

    let load_tasks = move || {
        let y = filter_year.get_untracked();
        let m = filter_month.get_untracked();
        wasm_bindgen_futures::spawn_local(async move {
            #[derive(Serialize)]
            struct Args { year: i32, month: u32 }
            let args = to_value(&Args { year: y, month: m }).unwrap();
            if let Ok(res) = invoke("dev_get_tasks_by_month", args).await.dyn_into::<JsValue>() {
                if let Ok(v) = from_value::<Vec<TaskRecord>>(res) {
                    set_tasks_list.set(v);
                }
            }
        });
    };

    let load_config = move || {
        wasm_bindgen_futures::spawn_local(async move {
            #[derive(Serialize)]
            struct Args { key: String }
            let args = to_value(&Args { key: "min_work_seconds".to_string() }).unwrap();
            if let Ok(res) = invoke("get_setting", args).await.dyn_into::<JsValue>() {
                let total_secs = if let Ok(Some(v)) = from_value::<Option<String>>(res) {
                    v.parse::<i64>().unwrap_or(28800)
                } else {
                    28800
                };
                set_cfg_hours.set(total_secs / 3600);
                set_cfg_minutes.set((total_secs % 3600) / 60);
                set_cfg_seconds.set(total_secs % 60);
            }
        });
    };

    // Load static config
    load_config();

    // Load data and track filter changes
    Effect::new(move |_| {
        filter_year.get();
        filter_month.get();
        load_attendance();
        load_tasks();
    });

    let delete_attendance = move |id: i64| {
        wasm_bindgen_futures::spawn_local(async move {
            #[derive(Serialize)]
            struct Args { id: i64 }
            let args = to_value(&Args { id }).unwrap();
            invoke("dev_delete_attendance", args).await;
            load_attendance();
        });
    };

    let delete_task = move |id: i64| {
        wasm_bindgen_futures::spawn_local(async move {
            #[derive(Serialize)]
            struct Args { id: i64 }
            let args = to_value(&Args { id }).unwrap();
            invoke("dev_delete_task", args).await;
            load_tasks();
        });
    };

    let save_min_hours = move |_| {
        let total = cfg_hours.get() * 3600 + cfg_minutes.get() * 60 + cfg_seconds.get();
        wasm_bindgen_futures::spawn_local(async move {
            #[derive(Serialize)]
            struct Args { key: String, value: String }
            let args = to_value(&Args { key: "min_work_seconds".to_string(), value: total.to_string() }).unwrap();
            invoke("set_setting", args).await;
            set_save_msg.set("Tersimpan!".to_string());
            let set_save_msg_clone = set_save_msg;
            let promise = js_sys::Promise::new(&mut |resolve, _| {
                let _ = web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 2000);
            });
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            set_save_msg_clone.set(String::new());
        });
    };

    let save_att = move |_| {
        let att = edit_att.get_untracked();
        let in_val = att_in_str.get_untracked();
        let out_val = att_out_str.get_untracked();
        let photo_in = att_photo_in.get_untracked();
        let photo_out = att_photo_out.get_untracked();
        wasm_bindgen_futures::spawn_local(async move {
            #[derive(Serialize)]
            struct Args {
                id: Option<i64>,
                date: String,
                #[serde(rename = "clockInTime")] clock_in_time: Option<String>,
                #[serde(rename = "clockOutTime")] clock_out_time: Option<String>,
                #[serde(rename = "clockInPhoto")] clock_in_photo: Option<String>,
                #[serde(rename = "clockOutPhoto")] clock_out_photo: Option<String>,
                status: String,
            }
            let clock_in = if in_val.len() >= 4 { Some(format!("{}T{}:00+08:00", att.date, in_val)) } else { None };
            let clock_out = if out_val.len() >= 4 { Some(format!("{}T{}:00+08:00", att.date, out_val)) } else { None };
            let args = to_value(&Args {
                id: if edit_att_is_new.get_untracked() { None } else { Some(att.id) },
                date: att.date.clone(),
                clock_in_time: clock_in,
                clock_out_time: clock_out,
                clock_in_photo: photo_in,
                clock_out_photo: photo_out,
                status: att.status.clone(),
            }).unwrap();
            invoke("dev_upsert_attendance", args).await;
            set_show_att_modal.set(false);
            load_attendance();
        });
    };

    let save_task = move |_| {
        let task = edit_task.get_untracked();
        let t_val = task_time_str.get_untracked();
        let photo = task_photo.get_untracked();
        wasm_bindgen_futures::spawn_local(async move {
            #[derive(Serialize)]
            struct Args {
                id: Option<i64>,
                date: String,
                time: String,
                #[serde(rename = "taskName")] task_name: String,
                output: String,
                notes: Option<String>,
                #[serde(rename = "photoPath")] photo_path: Option<String>,
            }
            let args = to_value(&Args {
                id: if edit_task_is_new.get_untracked() { None } else { Some(task.id) },
                date: task.date.clone(),
                time: if t_val.is_empty() { "08:00".to_string() } else { t_val },
                task_name: task.task_name.clone(),
                output: task.output.clone(),
                notes: task.notes.clone().filter(|s| !s.is_empty()),
                photo_path: photo,
            }).unwrap();
            invoke("dev_upsert_task", args).await;
            set_show_task_modal.set(false);
            load_tasks();
        });
    };

    // ── Photo capture helpers ────────────────────────────────────────────────
    // Calls window.captureDevPhoto(isCamera, badge, date, time) → Promise<base64>
    let capture_att_photo = move |is_camera: bool, is_in: bool| {
        let date = edit_att.get_untracked().date;
        let time = if is_in { att_in_str.get_untracked() } else { att_out_str.get_untracked() };
        let badge = if is_in { "Absensi Masuk" } else { "Absensi Pulang" };
        let set_loading = if is_in { set_att_photo_in_loading } else { set_att_photo_out_loading };
        let set_photo = if is_in { set_att_photo_in } else { set_att_photo_out };
        wasm_bindgen_futures::spawn_local(async move {
            set_loading.set(true);
            let func = js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("captureDevPhoto")).unwrap();
            let func = func.dyn_into::<js_sys::Function>().unwrap();
            let args = js_sys::Array::new();
            args.push(&JsValue::from_bool(is_camera));
            args.push(&JsValue::from_str(badge));
            args.push(&JsValue::from_str(&date));
            args.push(&JsValue::from_str(&time));
            let promise = func.apply(&JsValue::NULL, &args).unwrap();
            let promise = js_sys::Promise::from(promise);
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(raw) => {
                    if let Some(s) = raw.as_string() {
                        set_photo.set(Some(s));
                    }
                }
                Err(e) => {
                    web_sys::console::warn_1(&e);
                }
            }
            set_loading.set(false);
        });
    };

    let capture_task_photo_fn = move |is_camera: bool| {
        let date = edit_task.get_untracked().date;
        let time = task_time_str.get_untracked();
        wasm_bindgen_futures::spawn_local(async move {
            set_task_photo_loading.set(true);
            let func = js_sys::Reflect::get(&web_sys::window().unwrap(), &JsValue::from_str("captureDevPhoto")).unwrap();
            let func = func.dyn_into::<js_sys::Function>().unwrap();
            let args = js_sys::Array::new();
            args.push(&JsValue::from_bool(is_camera));
            args.push(&JsValue::from_str("Kegiatan"));
            args.push(&JsValue::from_str(&date));
            args.push(&JsValue::from_str(&time));
            let promise = func.apply(&JsValue::NULL, &args).unwrap();
            let promise = js_sys::Promise::from(promise);
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(raw) => {
                    if let Some(s) = raw.as_string() {
                        set_task_photo.set(Some(s));
                    }
                }
                Err(e) => {
                    web_sys::console::warn_1(&e);
                }
            }
            set_task_photo_loading.set(false);
        });
    };


    view! {
        <div style="background: #0f172a; min-height: 100%; color: white; display: flex; flex-direction: column;">
            {move || if not_dev() {
                view! {
                    <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; flex: 1; gap: 12px; padding: 20px;">
                        <i class="fas fa-lock" style="font-size: 48px; color: #ef4444;"></i>
                        <p style="color: #94a3b8; font-size: 14px;">"Dev Mode tidak aktif."</p>
                        <a href="/settings" style="background: #ef4444; color: white; padding: 10px 20px; border-radius: 8px; text-decoration: none; font-weight: 600;">"Ke Pengaturan"</a>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div style="display: flex; flex-direction: column; flex: 1;">
                        // --- Header ---
                        <div style="background: #ef4444; padding: 20px; display: flex; align-items: center; justify-content: space-between;">
                            <div>
                                <h1 style="font-size: 18px; font-weight: 800; margin: 0; letter-spacing: 1px;">"⚙ DEV MODE"</h1>
                                <p style="margin: 4px 0 0 0; font-size: 11px; opacity: 0.85;">"Cakerja — Internal Dashboard"</p>
                            </div>
                            <button
                                on:click=move |_| dev_mode.0.set(false)
                                style="background: rgba(255,255,255,0.2); color: white; border: none; padding: 8px 14px; border-radius: 8px; font-weight: 600; cursor: pointer; font-size: 13px;"
                            >
                                <i class="fas fa-power-off" style="margin-right: 6px;"></i>"Keluar"
                            </button>
                        </div>

                        // --- Tab Bar ---
                        <div style="display: flex; background: #1e293b; border-bottom: 2px solid #334155;">
                            {[("attendance", "fa-user-clock", "Presensi"), ("tasks", "fa-tasks", "Logbook"), ("config", "fa-sliders-h", "Konfigurasi")].into_iter().map(|(tab, icon, label)| {
                                let tab_str = tab.to_string();
                                let tab_str2 = tab_str.clone();
                                view! {
                                    <button
                                        on:click=move |_| set_active_tab.set(tab_str.clone())
                                        style=move || {
                                            let active = active_tab.get() == tab_str2;
                                            if active {
                                                "flex: 1; padding: 12px 4px; border: none; background: transparent; color: #ef4444; font-size: 12px; font-weight: 700; cursor: pointer; border-bottom: 2px solid #ef4444; display: flex; flex-direction: column; align-items: center; gap: 3px;"
                                            } else {
                                                "flex: 1; padding: 12px 4px; border: none; background: transparent; color: #64748b; font-size: 12px; font-weight: 600; cursor: pointer; border-bottom: 2px solid transparent; display: flex; flex-direction: column; align-items: center; gap: 3px;"
                                            }
                                        }
                                    >
                                        <i class=format!("fas {}", icon) style="font-size: 16px;"></i>
                                        {label}
                                    </button>
                                }
                            }).collect_view()}
                        </div>

                        // --- Filter Bulan/Tahun (Shared for Presensi & Logbook) ---
                        {move || if active_tab.get() != "config" {
                            view! {
                                <div style="padding: 12px 16px; background: #0f172a; border-bottom: 1px solid #1e293b; display: flex; gap: 8px; align-items: center;">
                                    <select
                                        style="background-color: #1e293b; color: white; border: 1px solid #334155; padding: 6px 24px 6px 10px; border-radius: 6px; font-size: 13px; -webkit-appearance: none; appearance: none; background-image: url('data:image/svg+xml;charset=US-ASCII,<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\"><path fill=\"%23ffffff\" d=\"M3 4l3 3 3-3z\"/></svg>'); background-repeat: no-repeat; background-position: right 8px center;"
                                        on:change=move |ev| {
                                            if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                                                set_filter_month.set(v);
                                            }
                                        }
                                    >
                                        {(1..=12).map(|m| {
                                            view! {
                                                <option value=m selected=move || filter_month.get() == m style="background: #1e293b; color: white;">
                                                    {match m {
                                                        1 => "Januari", 2 => "Februari", 3 => "Maret", 4 => "April",
                                                        5 => "Mei", 6 => "Juni", 7 => "Juli", 8 => "Agustus",
                                                        9 => "September", 10 => "Oktober", 11 => "November", 12 => "Desember",
                                                        _ => ""
                                                    }}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </select>
                                    <input
                                        type="number"
                                        style="background: #1e293b; color: white; border: 1px solid #334155; padding: 6px 10px; border-radius: 6px; font-size: 13px; width: 70px;"
                                        prop:value=move || filter_year.get()
                                        on:input=move |ev| {
                                            if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                                set_filter_year.set(v);
                                            }
                                        }
                                    />
                                </div>
                            }.into_any()
                        } else { view! { <div></div> }.into_any() }}

                        // --- Tab Content ---
                        <div style="padding: 16px; flex: 1;">

                            // == Presensi Tab ==
                            {move || if active_tab.get() == "attendance" {
                                let list = attendance_list.get();
                                view! {
                                    <div>
                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                                            <div style="font-size: 12px; color: #64748b;">
                                                {format!("{} rekord", list.len())}
                                            </div>
                                            <button
                                                on:click=move |_| {
                                                    let mut new_att = AttendanceRecord::default();
                                                    new_att.date = format!("{:04}-{:02}-{:02}", filter_year.get(), filter_month.get(), 1); // default
                                                    new_att.status = "Selesai".to_string();
                                                    set_edit_att.set(new_att);
                                                    set_edit_att_is_new.set(true);
                                                    set_att_in_str.set(String::new());
                                                    set_att_out_str.set(String::new());
                                                    set_att_photo_in.set(None);
                                                    set_att_photo_out.set(None);
                                                    set_show_att_modal.set(true);
                                                }
                                                style="background: #3b82f6; color: white; border: none; padding: 6px 12px; border-radius: 6px; font-size: 12px; font-weight: 600; cursor: pointer;"
                                            >
                                                <i class="fas fa-plus"></i> " Tambah"
                                            </button>
                                        </div>
                                        {list.into_iter().map(|att| {
                                            let id = att.id;
                                            let att_clone = att.clone();
                                            let in_t = att.clock_in_time.clone().unwrap_or_else(|| "—".to_string());
                                            let out_t = att.clock_out_time.clone().unwrap_or_else(|| "—".to_string());
                                            let in_short = in_t.chars().take(19).collect::<String>();
                                            let out_short = out_t.chars().take(19).collect::<String>();
                                            view! {
                                                <div style="background: #1e293b; border-radius: 10px; padding: 12px 14px; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: flex-start;">
                                                    <div style="flex: 1;">
                                                        <div style="font-weight: 700; font-size: 14px; color: white; margin-bottom: 4px;">{att.date.clone()}</div>
                                                        <div style="font-size: 11px; color: #94a3b8;">
                                                            "Masuk: " {in_short}
                                                        </div>
                                                        <div style="font-size: 11px; color: #94a3b8;">
                                                            "Pulang: " {out_short}
                                                        </div>
                                                        <div style=format!("font-size: 11px; margin-top: 4px; font-weight: 600; color: {};", 
                                                            if att.status == "Selesai" { "#4ade80" } else { "#facc15" })>
                                                            {att.status.clone()}
                                                        </div>
                                                    </div>
                                                    <div style="display: flex; flex-direction: column; gap: 6px;">
                                                        <button
                                                            on:click=move |_| {
                                                                set_edit_att.set(att_clone.clone());
                                                                set_edit_att_is_new.set(false);
                                                                set_att_in_str.set(att_clone.clock_in_time.clone().unwrap_or_default().chars().skip(11).take(5).collect::<String>());
                                                                set_att_out_str.set(att_clone.clock_out_time.clone().unwrap_or_default().chars().skip(11).take(5).collect::<String>());
                                                                set_att_photo_in.set(None);
                                                                set_att_photo_out.set(None);
                                                                set_show_att_modal.set(true);
                                                            }
                                                            style="background: #334155; color: white; border: none; border-radius: 6px; padding: 6px 10px; cursor: pointer; font-size: 12px;"
                                                        >
                                                            <i class="fas fa-edit"></i>
                                                        </button>
                                                        <button
                                                            on:click=move |_| delete_attendance(id)
                                                            style="background: #7f1d1d; color: #fca5a5; border: none; border-radius: 6px; padding: 6px 10px; cursor: pointer; font-size: 12px;"
                                                        >
                                                            <i class="fas fa-trash"></i>
                                                        </button>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            } else { view! { <div></div> }.into_any() }}

                            // == Logbook Tab ==
                            {move || if active_tab.get() == "tasks" {
                                let list = tasks_list.get();
                                view! {
                                    <div>
                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                                            <div style="font-size: 12px; color: #64748b;">
                                                {format!("{} rekord", list.len())}
                                            </div>
                                            <button
                                                on:click=move |_| {
                                                    let mut new_task = TaskRecord::default();
                                                    new_task.date = format!("{:04}-{:02}-{:02}", filter_year.get(), filter_month.get(), 1);
                                                    new_task.time = "08:00".to_string();
                                                    set_edit_task.set(new_task);
                                                    set_edit_task_is_new.set(true);
                                                    set_task_time_str.set("08:00".to_string());
                                                    set_task_photo.set(None);
                                                    set_show_task_modal.set(true);
                                                }
                                                style="background: #3b82f6; color: white; border: none; padding: 6px 12px; border-radius: 6px; font-size: 12px; font-weight: 600; cursor: pointer;"
                                            >
                                                <i class="fas fa-plus"></i> " Tambah"
                                            </button>
                                        </div>
                                        {list.into_iter().map(|task| {
                                            let id = task.id;
                                            let task_clone = task.clone();
                                            let photo = task.photo_path.clone();
                                            view! {
                                                <div style="background: #1e293b; border-radius: 10px; padding: 12px 14px; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: flex-start; gap: 10px;">
                                                    <div style="flex: 1; min-width: 0;">
                                                        <div style="font-weight: 700; font-size: 13px; color: white; margin-bottom: 2px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">
                                                            {task.task_name.clone()}
                                                        </div>
                                                        <div style="font-size: 11px; color: #94a3b8;">{task.date.clone()} " | " {task.time.chars().take(5).collect::<String>()}</div>
                                                        <div style="font-size: 11px; color: #94a3b8; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">{task.output.clone()}</div>
                                                        {move || if let Some(p) = photo.clone() {
                                                            let p2 = p.clone();
                                                            view! {
                                                                <img
                                                                    src=format!("data:image/jpeg;base64,{}", p)
                                                                    style="width: 48px; height: 48px; object-fit: cover; border-radius: 4px; margin-top: 6px; cursor: pointer;"
                                                                    on:click=move |_| set_photo_preview.set(Some(p2.clone()))
                                                                />
                                                            }.into_any()
                                                        } else { view! { <span></span> }.into_any() }}
                                                    </div>
                                                    <div style="display: flex; flex-direction: column; gap: 6px; flex-shrink: 0;">
                                                        <button
                                                            on:click=move |_| {
                                                                set_edit_task.set(task_clone.clone());
                                                                set_edit_task_is_new.set(false);
                                                                set_task_time_str.set(task_clone.time.chars().take(5).collect::<String>());
                                                                set_task_photo.set(None);
                                                                set_show_task_modal.set(true);
                                                            }
                                                            style="background: #334155; color: white; border: none; border-radius: 6px; padding: 6px 10px; cursor: pointer; font-size: 12px;"
                                                        >
                                                            <i class="fas fa-edit"></i>
                                                        </button>
                                                        <button
                                                            on:click=move |_| delete_task(id)
                                                            style="background: #7f1d1d; color: #fca5a5; border: none; border-radius: 6px; padding: 6px 10px; cursor: pointer; font-size: 12px;"
                                                        >
                                                            <i class="fas fa-trash"></i>
                                                        </button>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            } else { view! { <div></div> }.into_any() }}

                            // == Konfigurasi Tab ==
                            {move || if active_tab.get() == "config" {
                                view! {
                                    <div>
                                        <div style="background: #1e293b; border-radius: 12px; padding: 16px; margin-bottom: 16px;">
                                            <div style="font-weight: 700; font-size: 14px; color: white; margin-bottom: 4px;">
                                                <i class="fas fa-clock" style="margin-right: 8px; color: #ef4444;"></i>
                                                "Durasi Kerja Minimal (Testing)"
                                            </div>
                                            <div style="font-size: 12px; color: #64748b; margin-bottom: 12px;">
                                                "Atur batas minimal sebelum Absen Pulang diizinkan."
                                            </div>
                                            <div style="display: flex; gap: 12px; margin-bottom: 16px;">
                                                <div style="display: flex; flex-direction: column; gap: 4px;">
                                                    <label style="font-size: 11px; color: #94a3b8;">"Jam"</label>
                                                    <input type="number" min="0" style="background: #0f172a; color: white; border: 1px solid #334155; border-radius: 6px; padding: 8px; font-size: 14px; width: 60px;"
                                                        prop:value=move || cfg_hours.get() on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse() { set_cfg_hours.set(v) } } />
                                                </div>
                                                <div style="display: flex; flex-direction: column; gap: 4px;">
                                                    <label style="font-size: 11px; color: #94a3b8;">"Menit"</label>
                                                    <input type="number" min="0" max="59" style="background: #0f172a; color: white; border: 1px solid #334155; border-radius: 6px; padding: 8px; font-size: 14px; width: 60px;"
                                                        prop:value=move || cfg_minutes.get() on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse() { set_cfg_minutes.set(v) } } />
                                                </div>
                                                <div style="display: flex; flex-direction: column; gap: 4px;">
                                                    <label style="font-size: 11px; color: #94a3b8;">"Detik"</label>
                                                    <input type="number" min="0" max="59" style="background: #0f172a; color: white; border: 1px solid #334155; border-radius: 6px; padding: 8px; font-size: 14px; width: 60px;"
                                                        prop:value=move || cfg_seconds.get() on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse() { set_cfg_seconds.set(v) } } />
                                                </div>
                                            </div>
                                            <div style="display: flex; gap: 8px; align-items: center;">
                                                <button on:click=save_min_hours style="background: #ef4444; color: white; border: none; padding: 8px 16px; border-radius: 6px; font-weight: 600; cursor: pointer;">
                                                    "Simpan"
                                                </button>
                                                {move || if !save_msg.get().is_empty() {
                                                    view! { <span style="color: #4ade80; font-size: 13px; font-weight: 600;">{save_msg.get()}</span> }.into_any()
                                                } else { view! { <span></span> }.into_any() }}
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            } else { view! { <div></div> }.into_any() }}

                        </div>

                        // --- Edit Attendance Modal ---
                        {move || if show_att_modal.get() {
                            view! {
                                <div style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.8); z-index: 1000; display: flex; align-items: center; justify-content: center; padding: 20px; box-sizing: border-box;">
                                    <div style="background: #1e293b; padding: 20px; border-radius: 12px; width: 100%; max-width: 400px; display: flex; flex-direction: column; gap: 12px;">
                                        <h3 style="margin: 0; font-size: 16px; color: white;">{if edit_att_is_new.get() { "Tambah Absensi" } else { "Edit Absensi" }}</h3>
                                        
                                        <div style="display: flex; flex-direction: column; gap: 4px;">
                                            <label style="font-size: 11px; color: #94a3b8;">"Tanggal (YYYY-MM-DD)"</label>
                                            <input type="date" style="background: #0f172a; color: white; border: 1px solid #334155; padding: 8px; border-radius: 6px;"
                                                prop:value=move || edit_att.get().date
                                                on:input=move |ev| { 
                                                    let mut t = edit_att.get(); 
                                                    let new_date = event_target_value(&ev);
                                                    t.date = new_date.clone();
                                                    if let Some(in_str) = t.clock_in_time.as_mut() {
                                                        if in_str.len() >= 10 { in_str.replace_range(0..10, &new_date); }
                                                    }
                                                    if let Some(out_str) = t.clock_out_time.as_mut() {
                                                        if out_str.len() >= 10 { out_str.replace_range(0..10, &new_date); }
                                                    }
                                                    set_edit_att.set(t); 
                                                } />
                                        </div>
                                        
                                        <div style="display: flex; gap: 8px;">
                                            <div style="display: flex; flex-direction: column; gap: 4px; flex: 1;">
                                                <label style="font-size: 11px; color: #94a3b8;">"Waktu Masuk (HH:MM)"</label>
                                                <input type="text" placeholder="08:00" maxlength="5" style="background: #0f172a; color: white; border: 1px solid #334155; padding: 8px; border-radius: 6px; font-size: 12px;"
                                                    prop:value=move || att_in_str.get()
                                                    on:input=move |ev| set_att_in_str.set(event_target_value(&ev)) />
                                            </div>
                                            <div style="display: flex; flex-direction: column; gap: 4px; flex: 1;">
                                                <label style="font-size: 11px; color: #94a3b8;">"Waktu Pulang (HH:MM)"</label>
                                                <input type="text" placeholder="17:00" maxlength="5" style="background: #0f172a; color: white; border: 1px solid #334155; padding: 8px; border-radius: 6px; font-size: 12px;"
                                                    prop:value=move || att_out_str.get()
                                                    on:input=move |ev| set_att_out_str.set(event_target_value(&ev)) />
                                            </div>
                                        </div>

                                        <div style="font-size: 11px; color: #3b82f6; font-weight: 600; padding: 4px 0;">
                                            "Durasi Kerja: "
                                            {move || {
                                                let in_val = att_in_str.get();
                                                let out_val = att_out_str.get();
                                                if in_val.len() >= 4 && out_val.len() >= 4 {
                                                    let in_parts: Vec<&str> = in_val.split(':').collect();
                                                    let out_parts: Vec<&str> = out_val.split(':').collect();
                                                    if in_parts.len() == 2 && out_parts.len() == 2 {
                                                        let in_h = in_parts[0].parse::<i32>().unwrap_or(0);
                                                        let in_m = in_parts[1].parse::<i32>().unwrap_or(0);
                                                        let out_h = out_parts[0].parse::<i32>().unwrap_or(0);
                                                        let out_m = out_parts[1].parse::<i32>().unwrap_or(0);
                                                        let diff = (out_h * 60 + out_m) - (in_h * 60 + in_m);
                                                        if diff >= 0 {
                                                            return format!("{} Jam {} Menit", diff / 60, diff % 60);
                                                        }
                                                    }
                                                }
                                                "-".to_string()
                                            }}
                                        </div>

                                        <div style="display: flex; flex-direction: column; gap: 4px;">
                                            <label style="font-size: 11px; color: #94a3b8;">"Status"</label>
                                            <select style="background-color: #0f172a; color: white; border: 1px solid #334155; padding: 8px 24px 8px 8px; border-radius: 6px; -webkit-appearance: none; appearance: none; background-image: url('data:image/svg+xml;charset=US-ASCII,<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 12 12\"><path fill=\"%23ffffff\" d=\"M3 4l3 3 3-3z\"/></svg>'); background-repeat: no-repeat; background-position: right 10px center;"
                                                on:change=move |ev| { let mut t = edit_att.get(); t.status = event_target_value(&ev); set_edit_att.set(t); }>
                                                <option value="Selesai" selected=move || edit_att.get().status == "Selesai" style="background: #0f172a; color: white;">"Selesai"</option>
                                                <option value="Masuk" selected=move || edit_att.get().status == "Masuk" style="background: #0f172a; color: white;">"Masuk"</option>
                                                <option value="Libur" selected=move || edit_att.get().status == "Libur" style="background: #0f172a; color: white;">"Libur"</option>
                                                <option value="Cuti" selected=move || edit_att.get().status == "Cuti" style="background: #0f172a; color: white;">"Cuti"</option>
                                            </select>
                                        </div>

                                        // ── Foto Absensi ───────────────────────────────────────
                                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px;">
                                            // Foto Masuk
                                            <div style="display: flex; flex-direction: column; gap: 6px;">
                                                <div style="font-size: 11px; color: #94a3b8; font-weight: 600;">"Foto Masuk"</div>
                                                {move || if let Some(b64) = att_photo_in.get() {
                                                    view! {
                                                        <div style="position: relative;">
                                                            <img
                                                                src=format!("data:image/jpeg;base64,{}", b64)
                                                                style="width: 100%; height: 80px; object-fit: cover; border-radius: 6px; border: 1px solid #22c55e;"
                                                                on:click=move |_| set_photo_preview.set(Some(att_photo_in.get().unwrap_or_default()))
                                                            />
                                                            <button
                                                                on:click=move |_| set_att_photo_in.set(None)
                                                                style="position: absolute; top: 2px; right: 2px; background: #7f1d1d; color: #fca5a5; border: none; border-radius: 4px; width: 20px; height: 20px; font-size: 10px; cursor: pointer; line-height: 20px;"
                                                            >"✕"</button>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div style="display: flex; flex-direction: column; gap: 4px;">
                                                            <button
                                                                on:click=move |_| capture_att_photo(true, true)
                                                                disabled=move || att_photo_in_loading.get()
                                                                style="background: #1e3a5f; color: #60a5fa; border: 1px solid #1d4ed8; border-radius: 6px; padding: 6px 4px; font-size: 11px; cursor: pointer;"
                                                            >
                                                                {move || if att_photo_in_loading.get() { "⏳" } else { "📷 Ambil" }}
                                                            </button>
                                                            <button
                                                                on:click=move |_| capture_att_photo(false, true)
                                                                disabled=move || att_photo_in_loading.get()
                                                                style="background: #1e293b; color: #94a3b8; border: 1px solid #334155; border-radius: 6px; padding: 6px 4px; font-size: 11px; cursor: pointer;"
                                                            >
                                                                {move || if att_photo_in_loading.get() { "⏳" } else { "📁 Pilih" }}
                                                            </button>
                                                        </div>
                                                    }.into_any()
                                                }}
                                            </div>
                                            // Foto Pulang
                                            <div style="display: flex; flex-direction: column; gap: 6px;">
                                                <div style="font-size: 11px; color: #94a3b8; font-weight: 600;">"Foto Pulang"</div>
                                                {move || if let Some(b64) = att_photo_out.get() {
                                                    view! {
                                                        <div style="position: relative;">
                                                            <img
                                                                src=format!("data:image/jpeg;base64,{}", b64)
                                                                style="width: 100%; height: 80px; object-fit: cover; border-radius: 6px; border: 1px solid #22c55e;"
                                                                on:click=move |_| set_photo_preview.set(Some(att_photo_out.get().unwrap_or_default()))
                                                            />
                                                            <button
                                                                on:click=move |_| set_att_photo_out.set(None)
                                                                style="position: absolute; top: 2px; right: 2px; background: #7f1d1d; color: #fca5a5; border: none; border-radius: 4px; width: 20px; height: 20px; font-size: 10px; cursor: pointer; line-height: 20px;"
                                                            >"✕"</button>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div style="display: flex; flex-direction: column; gap: 4px;">
                                                            <button
                                                                on:click=move |_| capture_att_photo(true, false)
                                                                disabled=move || att_photo_out_loading.get()
                                                                style="background: #1e3a5f; color: #60a5fa; border: 1px solid #1d4ed8; border-radius: 6px; padding: 6px 4px; font-size: 11px; cursor: pointer;"
                                                            >
                                                                {move || if att_photo_out_loading.get() { "⏳" } else { "📷 Ambil" }}
                                                            </button>
                                                            <button
                                                                on:click=move |_| capture_att_photo(false, false)
                                                                disabled=move || att_photo_out_loading.get()
                                                                style="background: #1e293b; color: #94a3b8; border: 1px solid #334155; border-radius: 6px; padding: 6px 4px; font-size: 11px; cursor: pointer;"
                                                            >
                                                                {move || if att_photo_out_loading.get() { "⏳" } else { "📁 Pilih" }}
                                                            </button>
                                                        </div>
                                                    }.into_any()
                                                }}
                                            </div>
                                        </div>

                                        <div style="display: flex; gap: 8px; margin-top: 8px;">
                                            <button on:click=save_att style="flex: 1; background: #3b82f6; color: white; border: none; padding: 10px; border-radius: 6px; font-weight: 600;">"Simpan"</button>
                                            <button on:click=move |_| set_show_att_modal.set(false) style="flex: 1; background: #334155; color: white; border: none; padding: 10px; border-radius: 6px;">"Batal"</button>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        } else { view! { <span></span> }.into_any() }}

                        // --- Edit Task Modal ---
                        {move || if show_task_modal.get() {
                            view! {
                                <div style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.8); z-index: 1000; display: flex; align-items: center; justify-content: center; padding: 20px; box-sizing: border-box;">
                                    <div style="background: #1e293b; padding: 20px; border-radius: 12px; width: 100%; max-width: 400px; display: flex; flex-direction: column; gap: 12px;">
                                        <h3 style="margin: 0; font-size: 16px; color: white;">{if edit_task_is_new.get() { "Tambah Kegiatan" } else { "Edit Kegiatan" }}</h3>
                                        
                                        <div style="display: flex; gap: 8px;">
                                            <div style="display: flex; flex-direction: column; gap: 4px; flex: 2;">
                                                <label style="font-size: 11px; color: #94a3b8;">"Tanggal"</label>
                                                <input type="date" style="background: #0f172a; color: white; border: 1px solid #334155; padding: 8px; border-radius: 6px;"
                                                    prop:value=move || edit_task.get().date
                                                    on:input=move |ev| { let mut t = edit_task.get(); t.date = event_target_value(&ev); set_edit_task.set(t); } />
                                            </div>
                                            <div style="display: flex; flex-direction: column; gap: 4px; flex: 1;">
                                                <label style="font-size: 11px; color: #94a3b8;">"Waktu (HH:MM)"</label>
                                                <input type="text" placeholder="08:00" maxlength="5" style="background: #0f172a; color: white; border: 1px solid #334155; padding: 8px; border-radius: 6px;"
                                                    prop:value=move || task_time_str.get()
                                                    on:input=move |ev| set_task_time_str.set(event_target_value(&ev)) />
                                            </div>
                                        </div>
                                        
                                        <div style="display: flex; flex-direction: column; gap: 4px;">
                                            <label style="font-size: 11px; color: #94a3b8;">"Nama Kegiatan"</label>
                                            <input type="text" style="background: #0f172a; color: white; border: 1px solid #334155; padding: 8px; border-radius: 6px;"
                                                prop:value=move || edit_task.get().task_name
                                                on:input=move |ev| { let mut t = edit_task.get(); t.task_name = event_target_value(&ev); set_edit_task.set(t); } />
                                        </div>

                                        <div style="display: flex; flex-direction: column; gap: 4px;">
                                            <label style="font-size: 11px; color: #94a3b8;">"Output/Keluaran"</label>
                                            <input type="text" style="background: #0f172a; color: white; border: 1px solid #334155; padding: 8px; border-radius: 6px;"
                                                prop:value=move || edit_task.get().output
                                                on:input=move |ev| { let mut t = edit_task.get(); t.output = event_target_value(&ev); set_edit_task.set(t); } />
                                        </div>
                                        
                                        <div style="display: flex; flex-direction: column; gap: 4px;">
                                            <label style="font-size: 11px; color: #94a3b8;">"Catatan (Opsional)"</label>
                                            <input type="text" style="background: #0f172a; color: white; border: 1px solid #334155; padding: 8px; border-radius: 6px;"
                                                prop:value=move || edit_task.get().notes.unwrap_or_default()
                                                on:input=move |ev| { let mut t = edit_task.get(); t.notes = Some(event_target_value(&ev)); set_edit_task.set(t); } />
                                        </div>

                                        // ── Foto Kegiatan ──────────────────────────────────────
                                        <div>
                                            <div style="font-size: 11px; color: #94a3b8; font-weight: 600; margin-bottom: 6px;">"Foto Kegiatan"</div>
                                            {move || if let Some(b64) = task_photo.get() {
                                                view! {
                                                    <div style="position: relative; display: inline-block; width: 100%;">
                                                        <img
                                                            src=format!("data:image/jpeg;base64,{}", b64)
                                                            style="width: 100%; height: 100px; object-fit: cover; border-radius: 6px; border: 1px solid #22c55e;"
                                                            on:click=move |_| set_photo_preview.set(Some(task_photo.get().unwrap_or_default()))
                                                        />
                                                        <button
                                                            on:click=move |_| set_task_photo.set(None)
                                                            style="position: absolute; top: 4px; right: 4px; background: #7f1d1d; color: #fca5a5; border: none; border-radius: 4px; width: 22px; height: 22px; font-size: 11px; cursor: pointer; line-height: 22px;"
                                                        >"✕"</button>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div style="display: flex; gap: 8px;">
                                                        <button
                                                            on:click=move |_| capture_task_photo_fn(true)
                                                            disabled=move || task_photo_loading.get()
                                                            style="flex: 1; background: #1e3a5f; color: #60a5fa; border: 1px solid #1d4ed8; border-radius: 6px; padding: 8px; font-size: 12px; cursor: pointer;"
                                                        >
                                                            {move || if task_photo_loading.get() { "⏳ Memproses..." } else { "📷 Ambil Foto" }}
                                                        </button>
                                                        <button
                                                            on:click=move |_| capture_task_photo_fn(false)
                                                            disabled=move || task_photo_loading.get()
                                                            style="flex: 1; background: #1e293b; color: #94a3b8; border: 1px solid #334155; border-radius: 6px; padding: 8px; font-size: 12px; cursor: pointer;"
                                                        >
                                                            {move || if task_photo_loading.get() { "⏳ Memproses..." } else { "📁 Pilih Foto" }}
                                                        </button>
                                                    </div>
                                                }.into_any()
                                            }}
                                        </div>

                                        <div style="display: flex; gap: 8px; margin-top: 8px;">
                                            <button on:click=save_task style="flex: 1; background: #3b82f6; color: white; border: none; padding: 10px; border-radius: 6px; font-weight: 600;">"Simpan"</button>
                                            <button on:click=move |_| set_show_task_modal.set(false) style="flex: 1; background: #334155; color: white; border: none; padding: 10px; border-radius: 6px;">"Batal"</button>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        } else { view! { <span></span> }.into_any() }}

                        // --- Photo Preview Modal ---
                        {move || if let Some(photo_b64) = photo_preview.get() {
                            view! {
                                <div
                                    on:click=move |_| set_photo_preview.set(None)
                                    style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.9); z-index: 9998; display: flex; align-items: center; justify-content: center;"
                                >
                                    <img
                                        src=format!("data:image/jpeg;base64,{}", photo_b64)
                                        style="max-width: 95%; max-height: 90vh; border-radius: 8px;"
                                    />
                                </div>
                            }.into_any()
                        } else { view! { <div></div> }.into_any() }}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
