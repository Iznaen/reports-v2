use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::from_value;
use js_sys::Date;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct AttendanceRecord {
    pub id: i64,
    pub date: String,
    pub clock_in_time: Option<String>,
    pub clock_out_time: Option<String>,
    pub status: String,
}

#[component]
pub fn Home() -> impl IntoView {
    let (current_record, set_current_record) = signal::<Option<AttendanceRecord>>(None);
    let (current_time, set_current_time) = signal(Date::now());
    let (loading, set_loading) = signal(true);
    let (error_msg, set_error_msg) = signal(String::new());

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

    // Load initial state
    Effect::new(move |_| {
        fetch_attendance();
    });

    let do_clock_in = move |_| {
        spawn_local(async move {
            if let Err(e) = invoke("clock_in", JsValue::NULL).await.dyn_into::<JsValue>() {
                if let Some(err_str) = e.as_string() {
                    set_error_msg.set(err_str);
                }
            } else {
                fetch_attendance();
            }
        });
    };

    let do_clock_out = move |_| {
        spawn_local(async move {
            if let Err(e) = invoke("clock_out", JsValue::NULL).await.dyn_into::<JsValue>() {
                if let Some(err_str) = e.as_string() {
                    set_error_msg.set(err_str);
                }
            } else {
                fetch_attendance();
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
        </div>
    }
}
