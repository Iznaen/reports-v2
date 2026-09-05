use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::from_value;
use js_sys::Date;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = window, catch)]
    async fn captureAttendancePhoto(status: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = window, catch)]
    async fn captureTaskPhoto() -> Result<JsValue, JsValue>;
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct AttendanceRecord {
    pub id: i64,
    pub date: String,
    pub clock_in_time: Option<String>,
    pub clock_out_time: Option<String>,
    pub status: String,
}


#[derive(Debug, serde::Deserialize, Clone)]
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

#[component]
pub fn Home() -> impl IntoView {
    let (current_record, set_current_record) = signal::<Option<AttendanceRecord>>(None);
    let (current_time, set_current_time) = signal(Date::now());
    let (loading, set_loading) = signal(true);
    let (error_msg, set_error_msg) = signal(String::new());

    let (tasks, set_tasks) = signal::<Vec<TaskRecord>>(Vec::new());
    let (show_task_modal, set_show_task_modal) = signal(false);
    let (task_name, set_task_name) = signal(String::new());
    let (task_output, set_task_output) = signal(String::new());
    let (task_notes, set_task_notes) = signal(String::new());
    let (task_saving, set_task_saving) = signal(false);

    // Setup 1-second interval timer for UI updates
    Effect::new(move |_| {
        web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                Closure::wrap(Box::new(move || {
                    set_current_time.set(Date::now());
                }) as Box<dyn FnMut()>).into_js_value().unchecked_ref(),
                1000
            )
            .unwrap();
            
        // We can't strictly clean this up in Leptos 0.7 Effect directly without more complex tracking, 
        // but for a top-level component, it's fine.
    });

    let fetch_attendance = move || {
        spawn_local(async move {
            set_loading.set(true);
            if let Ok(res) = invoke("get_today_attendance", JsValue::NULL).await.dyn_into::<JsValue>() {
                if res.is_null() || res.is_undefined() {
                    // Create an empty dummy record so UI knows we loaded but have nothing
                    set_current_record.set(Some(AttendanceRecord {
                        id: 0,
                        date: String::new(),
                        clock_in_time: None,
                        clock_out_time: None,
                        status: "Belum".to_string(),
                    }));
                } else if let Ok(rec) = from_value::<AttendanceRecord>(res) {
                    set_current_record.set(Some(rec));
                }
            }
            set_loading.set(false);
        });
    };

    let fetch_tasks = move || {
        spawn_local(async move {
            if let Ok(res) = invoke("get_today_tasks", JsValue::NULL).await.dyn_into::<JsValue>() {
                if let Ok(list) = from_value::<Vec<TaskRecord>>(res) {
                    set_tasks.set(list);
                }
            }
        });
    };

    // Load initial state
    Effect::new(move |_| {
        fetch_attendance();
        fetch_tasks();
    });


    let do_save_task = move |_| {
        if task_name.get().is_empty() || task_output.get().is_empty() {
            set_error_msg.set("Uraian tugas dan output tidak boleh kosong.".to_string());
            return;
        }
        
        spawn_local(async move {
            set_task_saving.set(true);
            set_error_msg.set(String::new());
            
            match captureTaskPhoto().await {
                Ok(js_obj) => {
                    #[derive(serde::Deserialize)]
                    #[allow(non_snake_case)]
                    struct PhotoResult { base64Data: String, lat: f64, lng: f64 }
                    
                    if let Ok(res) = from_value::<PhotoResult>(js_obj) {
                        #[derive(serde::Serialize)]
                        #[serde(rename_all = "camelCase")]
                        struct Args { 
                            task_name: String, 
                            output: String, 
                            notes: Option<String>,
                            photo_path: Option<String>,
                            latitude: Option<f64>,
                            longitude: Option<f64>
                        }
                        
                        let args = serde_wasm_bindgen::to_value(&Args {
                            task_name: task_name.get(),
                            output: task_output.get(),
                            notes: if task_notes.get().is_empty() { None } else { Some(task_notes.get()) },
                            photo_path: Some(res.base64Data),
                            latitude: Some(res.lat),
                            longitude: Some(res.lng)
                        }).unwrap();
                        
                        if let Err(e) = invoke("add_task", args).await.dyn_into::<JsValue>() {
                            if let Some(err_str) = e.as_string() {
                                set_error_msg.set(err_str);
                            }
                        } else {
                            set_show_task_modal.set(false);
                            set_task_name.set(String::new());
                            set_task_output.set(String::new());
                            set_task_notes.set(String::new());
                            fetch_tasks();
                        }
                    } else {
                        set_error_msg.set("Gagal memproses data foto.".to_string());
                    }
                }
                Err(e) => {
                    set_error_msg.set(e.as_string().unwrap_or("Kamera dibatalkan.".to_string()));
                }
            }
            set_task_saving.set(false);
        });
    };

    let do_delete_task = move |id: i64| {
        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct Args { id: i64 }
            let args = serde_wasm_bindgen::to_value(&Args { id }).unwrap();
            
            if let Err(e) = invoke("delete_task", args).await.dyn_into::<JsValue>() {
                if let Some(err_str) = e.as_string() {
                    set_error_msg.set(err_str);
                }
            } else {
                fetch_tasks();
            }
        });
    };

    let do_clock_in = move |_| {
        spawn_local(async move {
            set_error_msg.set(String::new());
            match captureAttendancePhoto("Absen Masuk").await {
                Ok(js_obj) => {
                    #[derive(serde::Deserialize)]
                    #[allow(non_snake_case)]
                    struct PhotoResult { base64Data: String, lat: f64, lng: f64 }
                    
                    if let Ok(res) = from_value::<PhotoResult>(js_obj) {
                        #[derive(serde::Serialize)]
                        struct Args { photo: String, lat: f64, lng: f64 }
                        let args = serde_wasm_bindgen::to_value(&Args {
                            photo: res.base64Data,
                            lat: res.lat,
                            lng: res.lng
                        }).unwrap();
                        
                        if let Err(e) = invoke("clock_in", args).await.dyn_into::<JsValue>() {
                            if let Some(err_str) = e.as_string() {
                                set_error_msg.set(err_str);
                            }
                        } else {
                            fetch_attendance();
                        }
                    }
                }
                Err(e) => {
                    set_error_msg.set(e.as_string().unwrap_or("Kamera dibatalkan.".to_string()));
                }
            }
        });
    };

    let do_clock_out = move |_| {
        spawn_local(async move {
            set_error_msg.set(String::new());
            match captureAttendancePhoto("Absen Keluar").await {
                Ok(js_obj) => {
                    #[derive(serde::Deserialize)]
                    #[allow(non_snake_case)]
                    struct PhotoResult { base64Data: String, lat: f64, lng: f64 }
                    
                    if let Ok(res) = from_value::<PhotoResult>(js_obj) {
                        #[derive(serde::Serialize)]
                        struct Args { photo: String, lat: f64, lng: f64 }
                        let args = serde_wasm_bindgen::to_value(&Args {
                            photo: res.base64Data,
                            lat: res.lat,
                            lng: res.lng
                        }).unwrap();
                        
                        if let Err(e) = invoke("clock_out", args).await.dyn_into::<JsValue>() {
                            if let Some(err_str) = e.as_string() {
                                set_error_msg.set(err_str);
                            }
                        } else {
                            fetch_attendance();
                        }
                    }
                }
                Err(e) => {
                    set_error_msg.set(e.as_string().unwrap_or("Kamera dibatalkan.".to_string()));
                }
            }
        });
    };

    let elapsed_seconds = move || {
        if let Some(record) = current_record.get() {
            if let Some(in_time_str) = record.clock_in_time {
                if record.clock_out_time.is_some() {
                    // If already clocked out, just show the total fixed duration
                    let out_time = Date::parse(&record.clock_out_time.unwrap());
                    let in_time = Date::parse(&in_time_str);
                    return ((out_time - in_time) / 1000.0) as i64;
                } else {
                    // Realtime tracking
                    let in_time = Date::parse(&in_time_str);
                    let now = current_time.get();
                    let diff_ms = now - in_time;
                    if diff_ms > 0.0 {
                        return (diff_ms / 1000.0) as i64;
                    }
                }
            }
        }
        0
    };

    let elapsed_str = move || {
        let secs = elapsed_seconds();
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    };

    let can_clock_out = move || {
        // [TESTING] Changed to 5 seconds. Revert to 8 * 3600 later!
        elapsed_seconds() >= 5
    };

    view! {
        <div style="background: #f8fafc; min-height: 100%; position: relative;">
            // --- Header ---
            <div style="position: sticky; top: 0; z-index: 50; background: #1a3a5c; padding: 20px; color: #fff; display: flex; align-items: center; justify-content: space-between; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                <h1 style="font-size: 20px; font-weight: 600; margin: 0; display: flex; align-items: center; gap: 8px;">
                    <i class="fas fa-home"></i> "Beranda"
                </h1>
            </div>

            <div style="padding: 20px;">
                <div style="font-size: 18px; font-weight: 700; color: #0f172a; margin-bottom: 16px;">
                    "Status Kehadiran Hari Ini"
                </div>

                <div style="background: white; border-radius: 16px; padding: 20px; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.03); border: 1px solid #edf2f7;">
                    
                    // Error Message
                    {move || {
                        let err = error_msg.get();
                        if !err.is_empty() {
                            view! {
                                <div style="background: #fee2e2; color: #b91c1c; padding: 12px; border-radius: 8px; margin-bottom: 16px; font-size: 14px; text-align: center;">
                                    {err.clone()}
                                    <div 
                                        style="margin-top: 4px; text-decoration: underline; cursor: pointer;"
                                        on:click=move |_| set_error_msg.set(String::new())
                                    >"Tutup"</div>
                                </div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }
                    }}

                    // Absen Masuk Row
                    <div style="display: flex; justify-content: space-between; align-items: center; padding-bottom: 16px; border-bottom: 1px solid #e2e8f0;">
                        <div style="font-weight: 600; color: #1e293b; font-size: 16px;">
                            <i class="fas fa-sign-in-alt" style="margin-right: 8px; color: #3b82f6;"></i>
                            "Absen Masuk"
                        </div>
                        {move || {
                            if loading.get() {
                                view! { <i class="fas fa-spinner fa-spin" style="color: #cbd5e1; font-size: 24px;"></i> }.into_any()
                            } else if let Some(rec) = current_record.get() {
                                if rec.clock_in_time.is_some() {
                                    view! { <i class="fas fa-check-circle" style="color: #10b981; font-size: 28px;"></i> }.into_any()
                                } else {
                                    view! {
                                        <button 
                                            on:click=do_clock_in
                                            style="background: #3b82f6; color: white; border: none; padding: 8px 24px; border-radius: 100px; font-weight: 600; cursor: pointer; box-shadow: 0 4px 6px rgba(59, 130, 246, 0.2);"
                                        >"Mulai"</button>
                                    }.into_any()
                                }
                            } else {
                                view! { <span></span> }.into_any()
                            }
                        }}
                    </div>
                    
                    // Realtime Tracker
                    <div style="text-align: center; padding: 32px 0;">
                        <div style="font-size: 14px; color: #64748b; margin-bottom: 8px; font-weight: 500; text-transform: uppercase; letter-spacing: 1px;">
                            "Waktu Kerja"
                        </div>
                        <div style="font-size: 40px; font-weight: 700; color: #0f172a; font-family: 'Courier New', monospace; letter-spacing: 2px;">
                            {elapsed_str}
                        </div>
                    </div>
                    

                    // Task Log Section
                    {move || {
                        if let Some(rec) = current_record.get() {
                            if rec.clock_in_time.is_some() {
                                view! {
                                    <div style="margin-top: 32px; border-top: 1px solid #e2e8f0; padding-top: 24px;">
                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                                            <div style="font-weight: 700; color: #0f172a; font-size: 16px;">
                                                <i class="fas fa-tasks" style="color: #6366f1; margin-right: 8px;"></i>
                                                "Catat Bukti Kegiatan"
                                            </div>
                                            <button 
                                                on:click=move |_| set_show_task_modal.set(true)
                                                style="background: #eef2ff; color: #4f46e5; border: 1px solid #c7d2fe; padding: 6px 12px; border-radius: 6px; font-weight: 600; font-size: 14px; cursor: pointer;"
                                            >
                                                <i class="fas fa-plus" style="margin-right: 4px;"></i> "Tambah"
                                            </button>
                                        </div>
                                        
                                        // Task List
                                        <div style="display: flex; flex-direction: column; gap: 12px;">
                                            {move || {
                                                let t_list = tasks.get();
                                                if t_list.is_empty() {
                                                    view! {
                                                        <div style="text-align: center; color: #94a3b8; font-size: 14px; padding: 16px; background: #f8fafc; border-radius: 8px; border: 1px dashed #cbd5e1;">
                                                            "Belum ada kegiatan yang dicatat hari ini."
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    t_list.into_iter().map(|t| {
                                                        let tid = t.id;
                                                        view! {
                                                            <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 12px; position: relative;">
                                                                <button 
                                                                    on:click=move |_| do_delete_task(tid)
                                                                    style="position: absolute; top: 12px; right: 12px; background: transparent; border: none; color: #ef4444; cursor: pointer; padding: 4px;"
                                                                >
                                                                    <i class="fas fa-trash-alt"></i>
                                                                </button>
                                                                <div style="font-size: 12px; color: #64748b; font-weight: 600; margin-bottom: 4px;">{t.time}</div>
                                                                <div style="font-weight: 600; color: #1e293b; margin-bottom: 4px;">{t.task_name}</div>
                                                                <div style="font-size: 14px; color: #475569;">
                                                                    <span style="font-weight: 600;">"Output: "</span> {t.output}
                                                                </div>
                                                                {if let Some(n) = t.notes {
                                                                    view! {
                                                                        <div style="font-size: 13px; color: #64748b; margin-top: 4px; font-style: italic;">
                                                                            {n}
                                                                        </div>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <span></span> }.into_any()
                                                                }}
                                                            </div>
                                                        }
                                                    }).collect_view().into_any()
                                                }
                                            }}
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }
                        } else {
                            view! { <span></span> }.into_any()
                        }
                    }}

                    // Absen Keluar Row
                    <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 16px; border-top: 1px solid #e2e8f0;">
                        <div style="font-weight: 600; color: #1e293b; font-size: 16px;">
                            <i class="fas fa-sign-out-alt" style="margin-right: 8px; color: #ef4444;"></i>
                            "Absen Keluar"
                        </div>
                        {move || {
                            if loading.get() {
                                view! { <i class="fas fa-spinner fa-spin" style="color: #cbd5e1; font-size: 24px;"></i> }.into_any()
                            } else if let Some(rec) = current_record.get() {
                                if rec.clock_out_time.is_some() {
                                    view! { <i class="fas fa-check-circle" style="color: #10b981; font-size: 28px;"></i> }.into_any()
                                } else if rec.clock_in_time.is_some() {
                                    if can_clock_out() {
                                        view! {
                                            <button 
                                                on:click=do_clock_out
                                                style="background: #10b981; color: white; border: none; padding: 8px 24px; border-radius: 100px; font-weight: 600; cursor: pointer; box-shadow: 0 4px 6px rgba(16, 185, 129, 0.2);"
                                            >"Selesai"</button>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <button 
                                                style="background: #fef2f2; color: #ef4444; border: 1px solid #f87171; padding: 8px 16px; border-radius: 100px; font-weight: 600; opacity: 0.8; cursor: not-allowed;"
                                            >
                                                <i class="fas fa-lock" style="margin-right: 6px;"></i>"Belum 5 Detik"
                                            </button>
                                        }.into_any()
                                    }
                                } else {
                                    // Not clocked in yet
                                    view! {
                                        <i class="fas fa-times-circle" style="color: #cbd5e1; font-size: 28px;"></i>
                                    }.into_any()
                                }
                            } else {
                                view! { <span></span> }.into_any()
                            }
                        }}
                    </div>
                </div>
            </div>

            // Task Modal
            {move || {
                if show_task_modal.get() {
                    view! {
                        <div style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.5); z-index: 100; display: flex; align-items: center; justify-content: center; padding: 20px; box-sizing: border-box;">
                            <div style="background: white; border-radius: 16px; padding: 24px; width: 100%; max-width: 400px; box-shadow: 0 10px 25px rgba(0,0,0,0.2);">
                                <h3 style="margin-top: 0; margin-bottom: 16px; color: #0f172a; font-size: 18px;">"Tambah Kegiatan Baru"</h3>
                                
                                <div style="margin-bottom: 12px;">
                                    <label style="display: block; font-size: 14px; font-weight: 600; color: #475569; margin-bottom: 4px;">"Uraian Tugas"</label>
                                    <input 
                                        type="text" 
                                        placeholder="Cth: Rapat koordinasi..."
                                        prop:value=task_name
                                        on:input=move |e| set_task_name.set(event_target_value(&e))
                                        style="width: 100%; padding: 10px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px; box-sizing: border-box;"
                                    />
                                </div>
                                
                                <div style="margin-bottom: 12px;">
                                    <label style="display: block; font-size: 14px; font-weight: 600; color: #475569; margin-bottom: 4px;">"Output / Hasil"</label>
                                    <input 
                                        type="text" 
                                        placeholder="Cth: Dokumen kesepakatan"
                                        prop:value=task_output
                                        on:input=move |e| set_task_output.set(event_target_value(&e))
                                        style="width: 100%; padding: 10px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px; box-sizing: border-box;"
                                    />
                                </div>
                                
                                <div style="margin-bottom: 24px;">
                                    <label style="display: block; font-size: 14px; font-weight: 600; color: #475569; margin-bottom: 4px;">"Keterangan (Opsional)"</label>
                                    <textarea 
                                        placeholder="Catatan tambahan..."
                                        prop:value=task_notes
                                        on:input=move |e| set_task_notes.set(event_target_value(&e))
                                        style="width: 100%; padding: 10px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px; box-sizing: border-box; min-height: 80px;"
                                    ></textarea>
                                </div>
                                
                                <div style="display: flex; gap: 12px;">
                                    <button 
                                        on:click=move |_| set_show_task_modal.set(false)
                                        style="flex: 1; padding: 10px; background: #f1f5f9; color: #475569; border: none; border-radius: 8px; font-weight: 600; cursor: pointer;"
                                    >"Batal"</button>
                                    
                                    <button 
                                        on:click=do_save_task
                                        disabled=move || task_saving.get()
                                        style="flex: 1; padding: 10px; background: #4f46e5; color: white; border: none; border-radius: 8px; font-weight: 600; cursor: pointer; display: flex; justify-content: center; align-items: center; gap: 8px;"
                                    >
                                        {move || if task_saving.get() {
                                            view! { <i class="fas fa-spinner fa-spin"></i> }.into_any()
                                        } else {
                                            view! { <><i class="fas fa-camera"></i> "Foto & Simpan"</> }.into_any()
                                        }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

        </div>
    }
}
