use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::from_value;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct EmployeeProfile {
    pub name: String,
    pub ni: String,
    pub position: String,
    pub work_unit: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MonthSummary {
    pub year: i32,
    pub month: u32,
    pub period_name: String,
    pub total_hadir: u32,
    pub total_parsial: u32,
    pub total_alpha: u32,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DashboardData {
    pub profile: Option<EmployeeProfile>,
    pub months: Vec<MonthSummary>,
}

#[component]
pub fn Report() -> impl IntoView {
    let (data, set_data) = signal::<Option<DashboardData>>(None);
    let (loading, set_loading) = signal(true);
    let (error_msg, set_error_msg) = signal(String::new());

    Effect::new(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match invoke("get_report_dashboard_data", JsValue::NULL).await.dyn_into::<JsValue>() {
                Ok(res) => {
                    if let Ok(dashboard) = from_value::<DashboardData>(res) {
                        set_data.set(Some(dashboard));
                    } else {
                        set_error_msg.set("Gagal membaca format data dashboard.".to_string());
                    }
                }
                Err(e) => {
                    set_error_msg.set(e.as_string().unwrap_or("Error mengambil laporan.".to_string()));
                }
            }
            set_loading.set(false);
        });
    });

    view! {
        <div style="background-color: #f8fafc; min-height: 100%; display: flex; flex-direction: column;">
            
            // Header
            <div style="background: linear-gradient(135deg, #1e3a8a 0%, #3b82f6 100%); padding: 30px 20px 40px 20px; border-bottom-left-radius: 24px; border-bottom-right-radius: 24px; color: white; box-shadow: 0 4px 15px rgba(0,0,0,0.1);">
                <h1 style="margin: 0; font-size: 24px; font-weight: 700; text-align: center;">"Laporan Bulanan"</h1>
                <p style="margin: 8px 0 0 0; text-align: center; font-size: 14px; opacity: 0.9;">"Rekapitulasi Kinerja dan Presensi"</p>
            </div>

            <div style="padding: 0 20px; margin-top: -20px; flex: 1;">
                
                {move || {
                    if loading.get() {
                        return view! { <div style="text-align: center; padding: 40px; color: #64748b;">"Memuat data..."</div> }.into_any();
                    }
                    if !error_msg.get().is_empty() {
                        return view! { <div style="color: #ef4444; text-align: center; padding: 20px;">{error_msg.get()}</div> }.into_any();
                    }
                    
                    if let Some(dashboard) = data.get() {
                        let prof_name = dashboard.profile.as_ref().map(|p| p.name.clone()).unwrap_or("Tidak Ada".to_string());
                        let prof_unit = dashboard.profile.as_ref().map(|p| p.position.clone()).unwrap_or("-".to_string());
                        
                        view! {
                            <div style="display: flex; flex-direction: column; gap: 20px; padding-bottom: 40px;">
                                
                                // Profile Summary Card
                                <div style="background: white; border-radius: 16px; padding: 20px; box-shadow: 0 10px 25px rgba(0,0,0,0.05); display: flex; align-items: center; gap: 16px;">
                                    <div style="width: 50px; height: 50px; border-radius: 25px; background: #e0e7ff; display: flex; align-items: center; justify-content: center; color: #4338ca; font-size: 20px;">
                                        <i class="fas fa-user-tie"></i>
                                    </div>
                                    <div>
                                        <div style="font-weight: 700; color: #1e293b; font-size: 16px;">{prof_name.clone()}</div>
                                        <div style="color: #64748b; font-size: 13px; margin-top: 4px;">{prof_unit.clone()}</div>
                                    </div>
                                </div>
                                
                                <h3 style="margin: 10px 0 0 0; font-size: 16px; color: #334155; font-weight: 700;">"Riwayat Laporan"</h3>
                                
                                // List of Monthly Cards
                                {
                                    if dashboard.months.is_empty() {
                                        view! {
                                            <div style="text-align: center; padding: 30px; color: #94a3b8; background: white; border-radius: 12px;">
                                                "Belum ada data laporan."
                                            </div>
                                        }.into_any()
                                    } else {
                                        dashboard.months.into_iter().map(|m| {
                                            let p_name = m.period_name.clone();
                                            let target_url = format!("/report_print?year={}&month={}", m.year, m.month);
                                            
                                            view! {
                                                <div style="background: white; border-radius: 16px; padding: 20px; box-shadow: 0 4px 15px rgba(0,0,0,0.03); border: 1px solid #f1f5f9;">
                                                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; border-bottom: 1px solid #f1f5f9; padding-bottom: 12px;">
                                                        <div style="font-weight: 700; color: #0f172a; font-size: 17px;">
                                                            <i class="fas fa-calendar-alt" style="color: #3b82f6; margin-right: 8px;"></i>
                                                            {p_name.clone()}
                                                        </div>
                                                    </div>
                                                    
                                                    <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; margin-bottom: 20px;">
                                                        <div style="background: #ecfdf5; padding: 10px; border-radius: 10px; text-align: center;">
                                                            <div style="color: #10b981; font-size: 18px; font-weight: 800;">{m.total_hadir}</div>
                                                            <div style="color: #047857; font-size: 11px; font-weight: 600; margin-top: 4px;">"HADIR"</div>
                                                        </div>
                                                        <div style="background: #fffbeb; padding: 10px; border-radius: 10px; text-align: center;">
                                                            <div style="color: #f59e0b; font-size: 18px; font-weight: 800;">{m.total_parsial}</div>
                                                            <div style="color: #b45309; font-size: 11px; font-weight: 600; margin-top: 4px;">"PARSIAL"</div>
                                                        </div>
                                                        <div style="background: #fef2f2; padding: 10px; border-radius: 10px; text-align: center;">
                                                            <div style="color: #ef4444; font-size: 18px; font-weight: 800;">{m.total_alpha}</div>
                                                            <div style="color: #b91c1c; font-size: 11px; font-weight: 600; margin-top: 4px;">"ALPHA"</div>
                                                        </div>
                                                    </div>
                                                    
                                                    <a href=target_url style="display: block; text-decoration: none;">
                                                        <button style="width: 100%; background: #eff6ff; color: #2563eb; border: 1px solid #bfdbfe; padding: 12px; border-radius: 10px; font-weight: 700; font-size: 14px; cursor: pointer; display: flex; justify-content: center; align-items: center; gap: 8px;">
                                                            <i class="fas fa-file-pdf"></i> "Lihat Laporan"
                                                        </button>
                                                    </a>
                                                </div>
                                            }
                                        }).collect_view().into_any()
                                    }
                                }
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
