use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::from_value;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

// ============================================================
// DATA TYPES — harus cocok dengan MonthlyReportData dari backend
// ============================================================

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
    pub clock_out_time: Option<String>,
    pub work_hours: String,
    pub status: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct TaskRecord {
    pub id: Option<i64>,
    pub attendance_id: Option<i64>,
    pub date: String,
    pub time: String,
    pub task_name: String,
    pub output: String,
    pub notes: Option<String>,
    pub photo_path: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
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

// ============================================================
// PAYLOAD TYPES — dikirim ke gen_pdf commands di backend
// ============================================================

#[derive(serde::Serialize, Clone)]
pub struct PdfAttRow {
    pub no: u32,
    pub day: String,
    pub date: String,
    pub in_time: String,
    pub out_time: String,
    pub work_hours: String,
    pub status: String,
}

#[derive(serde::Serialize, Clone)]
pub struct PdfTaskRow {
    pub no: u32,
    pub date: String,
    pub time: String,
    pub task: String,
    pub output: String,
    pub notes: String,
}

#[derive(serde::Serialize, Clone)]
pub struct PdfPhoto {
    pub photo_type: String,
    pub date_str: String,
    pub caption: String,
    pub base64_data: String,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PdfReportInput {
    pub name: String,
    pub ni: String,
    pub position: String,
    pub unit: String,
    pub period: String,
    pub date_str: String,
    pub signature_base64: Option<String>,
    pub att_rows: Vec<PdfAttRow>,
    pub task_rows: Vec<PdfTaskRow>,
    pub photos: Vec<PdfPhoto>,
}

// ============================================================
// SVG PREVIEW RESULT
// ============================================================

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SvgPreviewResult {
    pub pages: Vec<String>,
    pub page_count: usize,
}

// ============================================================
// HELPER: Convert MonthlyReportData → PdfReportInput
// ============================================================

fn to_pdf_input(rpt: &MonthlyReportData, date_str: &str) -> PdfReportInput {
    let name = rpt.profile.as_ref().map(|p| p.name.clone()).unwrap_or_default();
    let ni = rpt.profile.as_ref().map(|p| p.ni.clone()).unwrap_or_default();
    let position = rpt.profile.as_ref().map(|p| p.position.clone()).unwrap_or_default();
    let unit = rpt.profile.as_ref().map(|p| p.work_unit.clone()).unwrap_or_default();

    let att_rows = rpt.attendance_days.iter().map(|d| PdfAttRow {
        no: d.no,
        day: d.day_name.clone(),
        date: d.date_str.clone(),
        in_time: d.clock_in_time.clone().unwrap_or_else(|| "—".to_string()),
        out_time: d.clock_out_time.clone().unwrap_or_else(|| "—".to_string()),
        work_hours: d.work_hours.clone(),
        status: d.status.clone(),
    }).collect();

    let task_rows = rpt.tasks.iter().enumerate().map(|(i, t)| PdfTaskRow {
        no: (i + 1) as u32,
        date: t.date.clone(),
        time: t.time.chars().take(5).collect(),
        task: t.task_name.clone(),
        output: t.output.clone(),
        notes: t.notes.clone().unwrap_or_else(|| "—".to_string()),
    }).collect();

    let photos = rpt.photos.iter().map(|p| PdfPhoto {
        photo_type: p.photo_type.clone(),
        date_str: p.date_str.clone(),
        caption: p.caption.clone(),
        base64_data: p.base64_data.clone(),
    }).collect();

    PdfReportInput {
        name,
        ni,
        position,
        unit,
        period: rpt.period_name.clone(),
        date_str: date_str.to_string(),
        signature_base64: rpt.signature_base64.clone(),
        att_rows,
        task_rows,
        photos,
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

// ============================================================
// MAIN COMPONENT
// ============================================================

#[component]
pub fn ReportPrint() -> impl IntoView {
    let query = use_query_map().get_untracked();
    let year = query.get("year").and_then(|y| y.parse::<i32>().ok()).unwrap_or(2026);
    let month = query.get("month").and_then(|m| m.parse::<u32>().ok()).unwrap_or(9);

    let (data, set_data) = signal::<Option<MonthlyReportData>>(None);
    let (loading, set_loading) = signal(true);
    let (error_msg, set_error_msg) = signal(String::new());

    // State preview SVG
    let (preview_pages, set_preview_pages) = signal::<Vec<String>>(vec![]);
    let (preview_loading, set_preview_loading) = signal(false);
    let (preview_error, set_preview_error) = signal(String::new());

    // State export PDF
    let (exporting, set_exporting) = signal(false);
    let (export_status, set_export_status) = signal(String::new());

    // Tanggal pengesahan (editable)
    let (report_date, set_report_date) = signal(format!("Kendari, 30 {} {}", indo_month(month), year));

    // Fetch data laporan saat load
    Effect::new(move |_| {
        spawn_local(async move {
            set_loading.set(true);

            #[derive(serde::Serialize)]
            struct Args { year: i32, month: u32 }
            let args = serde_wasm_bindgen::to_value(&Args { year, month }).unwrap();

            match invoke("get_monthly_report_data", args).await {
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

    // ---- ACTION: Generate SVG Preview ----
    let do_preview = move |_| {
        let rpt = data.get_untracked();
        let date = report_date.get_untracked();
        if let Some(rpt) = rpt {
            set_preview_loading.set(true);
            set_preview_error.set(String::new());
            set_preview_pages.set(vec![]);
            let pdf_input = to_pdf_input(&rpt, &date);
            spawn_local(async move {
                #[derive(serde::Serialize)]
                struct Args { input: PdfReportInput }
                let args = serde_wasm_bindgen::to_value(&Args { input: pdf_input }).unwrap();
                match invoke("generate_report_preview", args).await {
                    Ok(res) => {
                        if let Ok(result) = from_value::<SvgPreviewResult>(res) {
                            set_preview_pages.set(result.pages);
                        } else {
                            set_preview_error.set("Gagal membaca hasil preview.".to_string());
                        }
                    }
                    Err(e) => {
                        set_preview_error.set(
                            e.as_string().unwrap_or("Error membuat preview.".to_string())
                        );
                    }
                }
                set_preview_loading.set(false);
            });
        }
    };

    // ---- ACTION: Export PDF ----
    let do_export = move |_| {
        let rpt = data.get_untracked();
        let date = report_date.get_untracked();
        if let Some(rpt) = rpt {
            set_exporting.set(true);
            set_export_status.set(String::new());
            let pdf_input = to_pdf_input(&rpt, &date);
            spawn_local(async move {
                #[derive(serde::Serialize)]
                struct Args { input: PdfReportInput }
                let args = serde_wasm_bindgen::to_value(&Args { input: pdf_input }).unwrap();
                match invoke("export_report_pdf", args).await {
                    Ok(res) => {
                        let msg = res.as_string().unwrap_or_default();
                        if msg == "cancelled" {
                            set_export_status.set("Export dibatalkan.".to_string());
                        } else {
                            set_export_status.set(msg);
                        }
                    }
                    Err(e) => {
                        let err = e.as_string().unwrap_or_default();
                        if err.contains("cancelled") {
                            set_export_status.set("Export dibatalkan.".to_string());
                        } else {
                            set_export_status.set(format!("Error: {}", err));
                        }
                    }
                }
                set_exporting.set(false);
            });
        }
    };

    view! {
        <div style="background-color: #e2e8f0; min-height: 100vh; display: flex; flex-direction: column; align-items: center; font-family: 'Inter', sans-serif;">
            <style>
                "
                .svg-wrapper svg {
                    width: 100% !important;
                    height: auto !important;
                    display: block;
                }
                "
            </style>

            // ========== STICKY TOOLBAR ==========
            <div style="
                position: sticky; top: 0; z-index: 9999;
                background: white; width: 100%;
                box-shadow: 0 4px 15px rgba(0,0,0,0.1);
                padding: 16px 20px; box-sizing: border-box;
                display: flex; flex-direction: column; gap: 12px;
            ">
                // Baris atas: Kembali + tombol-tombol
                <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 10px;">
                    <a href="/report" style="text-decoration: none; color: #64748b; display: flex; align-items: center; gap: 8px; font-weight: 700; font-size: 15px;">
                        <i class="fas fa-arrow-left"></i> "Kembali"
                    </a>
                    <div style="display: flex; gap: 10px; align-items: center;">
                        // Tombol Preview
                        <button
                            on:click=do_preview
                            disabled=move || preview_loading.get() || loading.get()
                            style="background: #0f172a; color: white; border: none; padding: 10px 18px; border-radius: 8px; font-weight: 700; font-size: 13px; cursor: pointer; display: flex; align-items: center; gap: 8px;"
                        >
                            <i class="fas fa-eye"></i>
                            {move || if preview_loading.get() { "Membuat Preview..." } else { "Preview PDF" }}
                        </button>
                        // Tombol Export
                        <button
                            on:click=do_export
                            disabled=move || exporting.get() || loading.get()
                            style="background: #1e3a8a; color: white; border: none; padding: 10px 18px; border-radius: 8px; font-weight: 700; font-size: 13px; cursor: pointer; display: flex; align-items: center; gap: 8px;"
                        >
                            <i class="fas fa-file-pdf"></i>
                            {move || if exporting.get() { "Menyimpan..." } else { "Export PDF" }}
                        </button>
                    </div>
                </div>

                // Baris tanggal pengesahan
                <div style="display: flex; align-items: center; gap: 10px; background: #f8fafc; padding: 10px 15px; border-radius: 8px; border: 1px solid #e2e8f0;">
                    <i class="fas fa-calendar-alt" style="color: #64748b;"></i>
                    <span style="font-weight: 600; font-size: 13px; color: #475569;">"Tgl Pengesahan:"</span>
                    <input
                        type="text"
                        placeholder="Contoh: Kendari, 30 September 2026"
                        prop:value=move || report_date.get()
                        on:input=move |ev| set_report_date.set(event_target_value(&ev))
                        style="flex: 1; padding: 6px 12px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px;"
                    />
                </div>

                // Status export / error preview
                {move || {
                    let status = export_status.get();
                    let prev_err = preview_error.get();
                    if !status.is_empty() {
                        let color = if status.contains("Error") || status.contains("Gagal") { "#ef4444" } else { "#16a34a" };
                        view! {
                            <div style=format!("color: {}; font-size: 13px; font-weight: 600; padding: 6px 12px; background: #f8fafc; border-radius: 6px;", color)>
                                {status}
                            </div>
                        }.into_any()
                    } else if !prev_err.is_empty() {
                        view! {
                            <div style="color: #ef4444; font-size: 13px; font-weight: 600; padding: 6px 12px; background: #fef2f2; border-radius: 6px;">
                                {prev_err}
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>

            // ========== CONTENT AREA ==========
            <div style="width: 100%; max-width: 900px; padding: 24px; box-sizing: border-box;">

                {move || {
                    if loading.get() {
                        return view! {
                            <div style="text-align: center; padding: 60px; color: #64748b; font-size: 16px;">
                                <i class="fas fa-spinner fa-spin" style="margin-right: 10px;"></i>
                                "Memuat data laporan..."
                            </div>
                        }.into_any();
                    }
                    if !error_msg.get().is_empty() {
                        return view! {
                            <div style="text-align: center; padding: 60px; color: #ef4444;">
                                <i class="fas fa-exclamation-triangle" style="margin-right: 10px;"></i>
                                {error_msg.get()}
                            </div>
                        }.into_any();
                    }

                    let pages = preview_pages.get();

                    if preview_loading.get() {
                        return view! {
                            <div style="text-align: center; padding: 60px; color: #64748b; font-size: 16px;">
                                <i class="fas fa-spinner fa-spin" style="margin-right: 10px;"></i>
                                "Merender preview PDF... (mungkin perlu beberapa detik)"
                            </div>
                        }.into_any();
                    }

                    if pages.is_empty() {
                        // Belum ada preview — tampilkan placeholder
                        return view! {
                            <div style="
                                text-align: center; padding: 80px 40px;
                                background: white; border-radius: 16px;
                                box-shadow: 0 4px 20px rgba(0,0,0,0.06);
                                color: #64748b;
                            ">
                                <i class="fas fa-file-pdf" style="font-size: 48px; color: #cbd5e1; display: block; margin-bottom: 20px;"></i>
                                <p style="font-size: 16px; font-weight: 600; margin: 0 0 8px;">"Preview belum dibuat"</p>
                                <p style="font-size: 14px; margin: 0 0 24px;">"Klik tombol \"Preview PDF\" di atas untuk melihat hasil laporan sebelum mengexport."</p>
                                <p style="font-size: 12px; color: #94a3b8; margin: 0;">"Proses ini mungkin membutuhkan beberapa detik karena laporan di-render sepenuhnya di backend."</p>
                            </div>
                        }.into_any();
                    }

                    // Tampilkan SVG pages
                    view! {
                        <div style="display: flex; flex-direction: column; gap: 24px; align-items: center;">
                            {pages.into_iter().enumerate().map(|(i, svg)| {
                                view! {
                                    <div style="width: 100%; position: relative;">
                                        <div style="
                                            position: absolute; top: -12px; left: 12px;
                                            background: #1e3a8a; color: white;
                                            font-size: 11px; font-weight: 700; padding: 2px 10px;
                                            border-radius: 4px; z-index: 1;
                                        ">
                                            {format!("Halaman {}", i + 1)}
                                        </div>
                                        // Render SVG inline — 100% identik dengan output PDF
                                        <div
                                            class="svg-wrapper"
                                            style="
                                                background: white;
                                                box-shadow: 0 10px 30px rgba(0,0,0,0.15);
                                                border-radius: 4px;
                                                overflow: hidden;
                                                width: 100%;
                                            "
                                            inner_html=svg
                                        ></div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
