use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::from_value;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = window)]
    fn print();
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
pub struct DailyAttendance {
    pub no: u32,
    pub day_name: String,
    pub date_str: String,
    pub clock_in_time: Option<String>,
    pub clock_in_location: String,
    pub clock_out_time: Option<String>,
    pub clock_out_location: String,
    pub status: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct TaskRecord {
    pub date: String,
    pub time: String,
    pub task_name: String,
    pub output: String,
    pub notes: Option<String>,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PhotoItem {
    pub photo_type: String,
    pub date_str: String,
    pub time_str: String,
    pub base64_data: String,
    pub caption: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyReportData {
    pub profile: Option<EmployeeProfile>,
    pub signature_base64: Option<String>,
    pub period_name: String,
    pub attendance_days: Vec<DailyAttendance>,
    pub tasks: Vec<TaskRecord>,
    pub photos: Vec<PhotoItem>,
}

fn indo_month(month: u32) -> &'static str {
    match month {
        1 => "Januari", 2 => "Februari", 3 => "Maret", 4 => "April",
        5 => "Mei", 6 => "Juni", 7 => "Juli", 8 => "Agustus",
        9 => "September", 10 => "Oktober", 11 => "November", 12 => "Desember",
        _ => "",
    }
}

#[component]
pub fn ReportPrint() -> impl IntoView {
    let query = use_query_map().get_untracked();
    let year = query.get("year").and_then(|y| y.parse::<i32>().ok()).unwrap_or(2026);
    let month = query.get("month").and_then(|m| m.parse::<u32>().ok()).unwrap_or(9);

    let (data, set_data) = signal::<Option<MonthlyReportData>>(None);
    let (loading, set_loading) = signal(true);
    let (error_msg, set_error_msg) = signal(String::new());
    
    // Editable Report Date (defaults to end of the month roughly)
    let (report_date, set_report_date) = signal(format!("Kendari, 30 {} {}", indo_month(month), year));

    Effect::new(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            
            #[derive(serde::Serialize)]
            struct Args { year: i32, month: u32 }
            let args = serde_wasm_bindgen::to_value(&Args { year, month }).unwrap();

            match invoke("get_monthly_report_data", args).await.dyn_into::<JsValue>() {
                Ok(res) => {
                    if let Ok(report) = from_value::<MonthlyReportData>(res) {
                        set_data.set(Some(report));
                    } else {
                        set_error_msg.set("Gagal membaca format data laporan.".to_string());
                    }
                }
                Err(e) => {
                    set_error_msg.set(e.as_string().unwrap_or("Error mengambil laporan.".to_string()));
                }
            }
            set_loading.set(false);
        });
    });

    let do_print = move |_| {
        print();
    };

    view! {
        <div style="background-color: #e2e8f0; min-height: 100vh; display: flex; flex-direction: column; align-items: center;">
            <style>
                "
                /* Sticky Toolbar */
                .sticky-toolbar {
                    position: sticky;
                    top: 0;
                    z-index: 9999;
                    background: white;
                    width: 100%;
                    box-shadow: 0 4px 15px rgba(0,0,0,0.1);
                    padding: 16px 20px;
                    display: flex;
                    flex-direction: column;
                    gap: 12px;
                    box-sizing: border-box;
                }
                .toolbar-top {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                }
                .toolbar-bottom {
                    display: flex;
                    align-items: center;
                    gap: 10px;
                    background: #f8fafc;
                    padding: 10px 15px;
                    border-radius: 8px;
                    border: 1px solid #e2e8f0;
                }
                .toolbar-bottom input {
                    flex: 1;
                    padding: 6px 12px;
                    border: 1px solid #cbd5e1;
                    border-radius: 6px;
                    font-size: 14px;
                }
                
                /* Page Container for Mobile Preview */
                .report-preview-container {
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    gap: 24px;
                    padding: 24px;
                    width: 100%;
                    box-sizing: border-box;
                }

                .page {
                    background: white;
                    width: 100%;
                    max-width: 210mm; 
                    min-height: 297mm;
                    padding: 40px;
                    box-shadow: 0 10px 30px rgba(0,0,0,0.15);
                    box-sizing: border-box;
                    font-family: 'Inter', 'Helvetica Neue', Helvetica, Arial, sans-serif;
                    color: #1e293b;
                    overflow: hidden;
                    position: relative;
                }

                /* Modern Header */
                .doc-header {
                    text-align: center;
                    border-bottom: 3px solid #1e3a8a;
                    padding-bottom: 15px;
                    margin-bottom: 25px;
                }
                .doc-header h1 {
                    margin: 0;
                    font-size: 16pt;
                    font-weight: 800;
                    color: #0f172a;
                    text-transform: uppercase;
                    letter-spacing: 1px;
                }
                .doc-header .subtitle {
                    font-size: 11pt;
                    color: #475569;
                    margin-top: 5px;
                    font-weight: 500;
                }
                
                /* Profile Grid */
                .profile-header {
                    display: grid;
                    grid-template-columns: 1fr 1fr;
                    gap: 15px;
                    background: #f8fafc;
                    padding: 15px 20px;
                    border-radius: 8px;
                    border-left: 4px solid #3b82f6;
                    margin-bottom: 25px;
                    font-size: 10pt;
                }
                .profile-row { display: flex; gap: 8px; margin-bottom: 6px; }
                .profile-label { font-weight: 700; color: #475569; width: 80px; }
                .profile-val { font-weight: 600; color: #0f172a; }
                
                /* Section Title */
                h2.section-title {
                    font-size: 12pt;
                    color: #1e3a8a;
                    margin-bottom: 15px;
                    display: flex;
                    align-items: center;
                    gap: 8px;
                }
                
                /* Modern Table */
                .table-wrap {
                    width: 100%;
                    margin-bottom: 30px;
                }
                table {
                    width: 100%;
                    border-collapse: collapse;
                    font-size: 9pt;
                }
                th {
                    background-color: #1e3a8a;
                    color: white;
                    font-weight: 600;
                    text-align: left;
                    padding: 10px 8px;
                    border: 1px solid #1e3a8a;
                }
                td {
                    border: 1px solid #cbd5e1;
                    padding: 8px;
                    vertical-align: middle;
                    color: #334155;
                }
                tr:nth-child(even) td { background-color: #f8fafc; }
                .text-center { text-align: center; }
                
                /* Signature Modern Layout */
                .signature-area {
                    margin-top: 40px;
                    float: right;
                    width: 250px;
                    text-align: center;
                    font-size: 10pt;
                    color: #0f172a;
                }
                .sig-date { margin-bottom: 8px; }
                .sig-box {
                    height: 80px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    margin: 5px 0;
                }
                .sig-box img { max-height: 100%; max-width: 100%; }
                .sig-name {
                    font-weight: 800;
                    text-decoration: underline;
                    margin-bottom: 4px;
                }
                .sig-ni {
                    font-weight: 600;
                    color: #475569;
                }

                .page-footer {
                    position: absolute;
                    bottom: 20px;
                    left: 40px;
                    right: 40px;
                    text-align: right;
                    font-size: 8pt;
                    color: #94a3b8;
                    border-top: 1px solid #e2e8f0;
                    padding-top: 10px;
                }
                
                /* Photo Grid */
                .photo-grid {
                    display: grid;
                    grid-template-columns: repeat(2, 1fr);
                    gap: 15px;
                    margin-bottom: 30px;
                }
                .photo-item {
                    border: 1px solid #e2e8f0;
                    border-radius: 8px;
                    padding: 10px;
                    text-align: center;
                    background: white;
                    box-shadow: 0 2px 5px rgba(0,0,0,0.02);
                }
                .photo-item img {
                    width: 100%;
                    height: 160px;
                    object-fit: cover;
                    border-radius: 4px;
                }
                .photo-item .caption {
                    font-size: 9pt;
                    margin-top: 10px;
                    font-weight: 600;
                    color: #475569;
                }

                /* --- Strict Print Rules --- */
                @media print {
                  body, html { background: white !important; margin: 0; padding: 0; height: auto !important; }
                  .no-print { display: none !important; }
                  #app-nav-bar { display: none !important; }
                  
                  /* FIX FLEXBOX PAGINATION BUG: Browsers cannot paginate inside flex containers */
                  div[style*=\"display: flex\"] { display: block !important; }
                  .report-preview-container { 
                      display: block !important; 
                      padding: 0 !important; 
                      margin: 0 !important; 
                  }
                  
                  .page { 
                    box-shadow: none !important; 
                    margin: 0 !important; 
                    padding: 0 !important; 
                    page-break-after: always;
                    page-break-inside: auto;
                    width: 100% !important;
                    max-width: 100% !important;
                    position: static !important;
                    height: auto !important;
                    min-height: 0 !important;
                  }

                  .page-footer {
                    position: static !important;
                    margin-top: 20px !important;
                  }
                  
                  @page {
                    size: A4 portrait;
                    margin: 15mm;
                  }
                  
                  table { table-layout: auto !important; width: 100% !important; font-size: 8pt !important; page-break-inside: auto; }
                  tr { page-break-inside: avoid; page-break-after: auto; }
                  thead { display: table-header-group; }
                  tfoot { display: table-footer-group; }
                  th, td { padding: 4px 4px !important; }
                }
                "
            </style>

            <div class="no-print sticky-toolbar">
                <div class="toolbar-top">
                    <a href="/report" style="text-decoration: none; color: #64748b; display: flex; align-items: center; gap: 8px; font-weight: 700; font-size: 15px;">
                        <i class="fas fa-arrow-left"></i> "Kembali"
                    </a>
                    <button 
                        on:click=do_print
                        style="background: #1e3a8a; color: white; border: none; padding: 10px 20px; border-radius: 8px; font-weight: 700; font-size: 14px; cursor: pointer; box-shadow: 0 4px 6px rgba(30,58,138,0.2);"
                    >
                        <i class="fas fa-print" style="margin-right: 8px;"></i> "Cetak PDF"
                    </button>
                </div>
                <div class="toolbar-bottom">
                    <i class="fas fa-calendar-alt" style="color: #64748b;"></i>
                    <span style="font-weight: 600; font-size: 13px; color: #475569;">"Tgl Pengesahan:"</span>
                    <input 
                        type="text" 
                        placeholder="Contoh: Kendari, 30 September 2026"
                        prop:value=move || report_date.get() 
                        on:input=move |ev| set_report_date.set(event_target_value(&ev)) 
                    />
                </div>
            </div>
            
            {move || {
                if loading.get() {
                    return view! { <div style="text-align: center; padding: 40px;">"Memuat laporan..."</div> }.into_any();
                }
                if !error_msg.get().is_empty() {
                    return view! { <div style="color: red; text-align: center;">{error_msg.get()}</div> }.into_any();
                }
                
                if let Some(rpt) = data.get() {
                    let prof_name = rpt.profile.as_ref().map(|p| p.name.clone()).unwrap_or("Tidak Ada".to_string());
                    let prof_ni = rpt.profile.as_ref().map(|p| p.ni.clone()).unwrap_or("-".to_string());
                    let prof_pos = rpt.profile.as_ref().map(|p| p.position.clone()).unwrap_or("-".to_string());
                    let prof_unit = rpt.profile.as_ref().map(|p| p.work_unit.clone()).unwrap_or("-".to_string());
                    let p_name = rpt.period_name.clone();
                    let sig_img = rpt.signature_base64.clone();
                    let active_date = report_date.get();
                    
                    view! {
                        <div class="report-preview-container">
                            // ================= HALAMAN 1: PRESENSI =================
                            <div class="page">
                                <div class="doc-header">
                                    <h1>"LAPORAN KINERJA DAN PRESENSI BULANAN"</h1>
                                    <div class="subtitle">"Periode "{p_name.clone()}</div>
                                </div>
                                
                                <div class="profile-header">
                                    <div>
                                        <div class="profile-row"><span class="profile-label">"Nama"</span><span class="profile-val">": " {prof_name.clone()}</span></div>
                                        <div class="profile-row"><span class="profile-label">"NI / NIP"</span><span class="profile-val">": " {prof_ni.clone()}</span></div>
                                    </div>
                                    <div>
                                        <div class="profile-row"><span class="profile-label">"Jabatan"</span><span class="profile-val">": " {prof_pos.clone()}</span></div>
                                        <div class="profile-row"><span class="profile-label">"Unit Kerja"</span><span class="profile-val">": " {prof_unit.clone()}</span></div>
                                    </div>
                                </div>
                                
                                <h2 class="section-title">"A. Rekapitulasi Presensi"</h2>
                                <div class="table-wrap">
                                    <table>
                                        <thead>
                                            <tr>
                                                <th style="width:5%;" class="text-center">"No"</th>
                                                <th style="width:10%;">"Hari"</th>
                                                <th style="width:15%;">"Tanggal"</th>
                                                <th style="width:10%;" class="text-center">"Jam Masuk"</th>
                                                <th style="width:15%;">"Lokasi Masuk"</th>
                                                <th style="width:10%;" class="text-center">"Jam Pulang"</th>
                                                <th style="width:15%;">"Lokasi Pulang"</th>
                                                <th style="width:20%;">"Keterangan"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {rpt.attendance_days.into_iter().map(|d| {
                                                view! {
                                                    <tr>
                                                        <td class="text-center">{d.no}</td>
                                                        <td>{d.day_name}</td>
                                                        <td>{d.date_str}</td>
                                                        <td class="text-center">{d.clock_in_time.unwrap_or("—".to_string())}</td>
                                                        <td>{d.clock_in_location}</td>
                                                        <td class="text-center">{d.clock_out_time.unwrap_or("—".to_string())}</td>
                                                        <td>{d.clock_out_location}</td>
                                                        <td>{d.status}</td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                                
                                <div class="signature-area">
                                    <div class="sig-date">{active_date.clone()}</div>
                                    <div class="sig-box">
                                        {if let Some(ref b64) = sig_img {
                                            view! { <img src=b64.clone() alt="Tanda Tangan" /> }.into_any()
                                        } else {
                                            view! { <div style="color: #94a3b8; font-style: italic;">"(Belum ada ttd)"</div> }.into_any()
                                        }}
                                    </div>
                                    <div class="sig-name">{prof_name.clone()}</div>
                                    <div class="sig-ni">"NI. "{prof_ni.clone()}</div>
                                </div>
                                <div class="page-footer">{format!("Dicetak dari Sistem Laporan Kinerja | Halaman 1")}</div>
                            </div>

                            // ================= HALAMAN 2: LOGBOOK =================
                            <div class="page">
                                <div class="doc-header">
                                    <h1>"LAPORAN KINERJA DAN PRESENSI BULANAN"</h1>
                                    <div class="subtitle">"Periode "{p_name.clone()}</div>
                                </div>

                                <div class="profile-header">
                                    <div>
                                        <div class="profile-row"><span class="profile-label">"Nama"</span><span class="profile-val">": " {prof_name.clone()}</span></div>
                                        <div class="profile-row"><span class="profile-label">"NI / NIP"</span><span class="profile-val">": " {prof_ni.clone()}</span></div>
                                    </div>
                                    <div>
                                        <div class="profile-row"><span class="profile-label">"Jabatan"</span><span class="profile-val">": " {prof_pos.clone()}</span></div>
                                        <div class="profile-row"><span class="profile-label">"Unit Kerja"</span><span class="profile-val">": " {prof_unit.clone()}</span></div>
                                    </div>
                                </div>
                                
                                <h2 class="section-title">"B. Logbook Kegiatan Harian"</h2>
                                <div class="table-wrap">
                                    <table>
                                        <thead>
                                            <tr>
                                                <th style="width:5%;" class="text-center">"No"</th>
                                                <th style="width:15%;">"Tanggal"</th>
                                                <th style="width:10%;" class="text-center">"Jam"</th>
                                                <th style="width:30%;">"Uraian Tugas"</th>
                                                <th style="width:25%;">"Output"</th>
                                                <th style="width:15%;">"Keterangan"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {
                                                let mut i = 1;
                                                rpt.tasks.into_iter().map(|t| {
                                                    let cur_i = i;
                                                    i += 1;
                                                    view! {
                                                        <tr>
                                                            <td class="text-center">{cur_i}</td>
                                                            <td>{t.date}</td>
                                                            <td class="text-center">{t.time}</td>
                                                            <td>{t.task_name}</td>
                                                            <td>{t.output}</td>
                                                            <td>{t.notes.unwrap_or("—".to_string())}</td>
                                                        </tr>
                                                    }
                                                }).collect_view()
                                            }
                                        </tbody>
                                    </table>
                                </div>
                                
                                <div class="signature-area">
                                    <div class="sig-date">{active_date.clone()}</div>
                                    <div class="sig-box">
                                        {if let Some(ref b64) = sig_img {
                                            view! { <img src=b64.clone() alt="Tanda Tangan" /> }.into_any()
                                        } else {
                                            view! { <div style="color: #94a3b8; font-style: italic;">"(Belum ada ttd)"</div> }.into_any()
                                        }}
                                    </div>
                                    <div class="sig-name">{prof_name.clone()}</div>
                                    <div class="sig-ni">"NI. "{prof_ni.clone()}</div>
                                </div>
                                <div class="page-footer">{format!("Dicetak dari Sistem Laporan Kinerja | Halaman 2")}</div>
                            </div>
                            
                            // ================= HALAMAN 3: LAMPIRAN FOTO =================
                            <div class="page">
                                <div class="doc-header">
                                    <h1>"LAPORAN KINERJA DAN PRESENSI BULANAN"</h1>
                                    <div class="subtitle">"Periode "{p_name.clone()}" – Lampiran Dokumentasi"</div>
                                </div>

                                <div class="profile-header">
                                    <div>
                                        <div class="profile-row"><span class="profile-label">"Nama"</span><span class="profile-val">": " {prof_name.clone()}</span></div>
                                        <div class="profile-row"><span class="profile-label">"NI / NIP"</span><span class="profile-val">": " {prof_ni.clone()}</span></div>
                                    </div>
                                    <div>
                                        <div class="profile-row"><span class="profile-label">"Jabatan"</span><span class="profile-val">": " {prof_pos.clone()}</span></div>
                                        <div class="profile-row"><span class="profile-label">"Unit Kerja"</span><span class="profile-val">": " {prof_unit.clone()}</span></div>
                                    </div>
                                </div>
                                
                                <h2 class="section-title">"C. Lampiran Foto Dokumentasi"</h2>
                                <div class="photo-grid">
                                    {rpt.photos.into_iter().map(|p| {
                                        view! {
                                            <div class="photo-item">
                                                <img src=p.base64_data alt=p.photo_type />
                                                <div class="caption">{p.caption}</div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                                
                                <div class="signature-area">
                                    <div class="sig-date">{active_date.clone()}</div>
                                    <div class="sig-box">
                                        {if let Some(ref b64) = sig_img {
                                            view! { <img src=b64.clone() alt="Tanda Tangan" /> }.into_any()
                                        } else {
                                            view! { <div style="color: #94a3b8; font-style: italic;">"(Belum ada ttd)"</div> }.into_any()
                                        }}
                                    </div>
                                    <div class="sig-name">{prof_name.clone()}</div>
                                    <div class="sig-ni">"NI. "{prof_ni.clone()}</div>
                                </div>
                                <div class="page-footer">{format!("Dicetak dari Sistem Laporan Kinerja | Halaman 3")}</div>
                            </div>

                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}
