use tauri::{command, AppHandle};
use tauri_plugin_dialog::DialogExt;
use base64::{Engine as _, engine::general_purpose::STANDARD};


use typst_as_lib::TypstEngine;
use typst_library::foundations::{Dict, IntoValue, Value, Bytes, Array};
use typst_layout::PagedDocument;

// ========================================================================
// EMBEDDED ASSETS (kompilasi ke dalam binary)
// ========================================================================

static TEMPLATE: &str = include_str!("../templates/laporan.typ");

// Font Times New Roman (di-embed ke binary agar tidak bergantung sistem)
static FONT_TNR_REGULAR: &[u8]     = include_bytes!("/usr/share/fonts/truetype/msttcorefonts/Times_New_Roman.ttf");
static FONT_TNR_BOLD: &[u8]        = include_bytes!("/usr/share/fonts/truetype/msttcorefonts/Times_New_Roman_Bold.ttf");
static FONT_TNR_ITALIC: &[u8]      = include_bytes!("/usr/share/fonts/truetype/msttcorefonts/Times_New_Roman_Italic.ttf");
static FONT_TNR_BOLD_ITALIC: &[u8] = include_bytes!("/usr/share/fonts/truetype/msttcorefonts/Times_New_Roman_Bold_Italic.ttf");

// ========================================================================
// DATA TYPES — input dari Leptos via JSON
// ========================================================================

#[derive(Debug, serde::Deserialize, Clone)]
pub struct PdfAttRow {
    pub no: u32,
    pub day: String,
    pub date: String,
    pub in_time: String,
    pub in_loc: String,
    pub out_time: String,
    pub out_loc: String,
    pub status: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct PdfTaskRow {
    pub no: u32,
    pub date: String,
    pub time: String,
    pub task: String,
    pub output: String,
    pub notes: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct PdfPhoto {
    pub photo_type: String,
    pub date_str: String,
    pub caption: String,
    pub base64_data: String,
}

#[derive(Debug, serde::Deserialize)]
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

// ========================================================================
// CONVERT: Rust → Typst Dict
// ========================================================================

fn att_row_to_dict(r: &PdfAttRow) -> Value {
    let mut d = Dict::new();
    d.insert("no".into(),       (r.no as i64).into_value());
    d.insert("day".into(),      r.day.clone().into_value());
    d.insert("date".into(),     r.date.clone().into_value());
    d.insert("in_time".into(),  r.in_time.clone().into_value());
    d.insert("in_loc".into(),   r.in_loc.clone().into_value());
    d.insert("out_time".into(), r.out_time.clone().into_value());
    d.insert("out_loc".into(),  r.out_loc.clone().into_value());
    d.insert("status".into(),   r.status.clone().into_value());
    Value::Dict(d)
}

fn task_row_to_dict(r: &PdfTaskRow) -> Value {
    let mut d = Dict::new();
    d.insert("no".into(),     (r.no as i64).into_value());
    d.insert("date".into(),   r.date.clone().into_value());
    d.insert("time".into(),   r.time.clone().into_value());
    d.insert("task".into(),   r.task.clone().into_value());
    d.insert("output".into(), r.output.clone().into_value());
    d.insert("notes".into(),  r.notes.clone().into_value());
    Value::Dict(d)
}

fn photo_to_dict(p: &PdfPhoto) -> Value {
    let mut d = Dict::new();
    d.insert("photo_type".into(), p.photo_type.clone().into_value());
    d.insert("date_str".into(),   p.date_str.clone().into_value());
    d.insert("caption".into(),    p.caption.clone().into_value());
    // Strip data URL prefix if present, then decode to raw bytes for Typst image.decode()
    let raw_b64 = p.base64_data
        .splitn(2, ',')
        .last()
        .unwrap_or(&p.base64_data);
    let bytes = STANDARD.decode(raw_b64).unwrap_or_default();
    d.insert("bytes".into(), Value::Bytes(Bytes::new(bytes)));
    Value::Dict(d)
}

fn build_typst_dict(input: &PdfReportInput) -> Dict {
    let mut root = Dict::new();
    root.insert("name".into(),     input.name.clone().into_value());
    root.insert("ni".into(),       input.ni.clone().into_value());
    root.insert("position".into(), input.position.clone().into_value());
    root.insert("unit".into(),     input.unit.clone().into_value());
    root.insert("period".into(),   input.period.clone().into_value());
    root.insert("date_str".into(), input.date_str.clone().into_value());

    // Signature sebagai raw bytes untuk Typst image.decode()
    let sig_val = match &input.signature_base64 {
        Some(b64) if !b64.is_empty() => {
            let raw = b64.splitn(2, ',').last().unwrap_or(b64);
            let bytes = STANDARD.decode(raw).unwrap_or_default();
            if bytes.is_empty() {
                Value::None
            } else {
                Value::Bytes(Bytes::new(bytes))
            }
        }
        _ => Value::None,
    };
    root.insert("sig_bytes".into(), sig_val);

    // Attendance rows
    let att_arr: Array = input.att_rows.iter().map(att_row_to_dict).collect();
    root.insert("att_rows".into(), Value::Array(att_arr));

    // Task rows
    let task_arr: Array = input.task_rows.iter().map(task_row_to_dict).collect();
    root.insert("task_rows".into(), Value::Array(task_arr));

    // Photos
    let photo_arr: Array = input.photos.iter().map(photo_to_dict).collect();
    root.insert("photos".into(), Value::Array(photo_arr));

    root
}

// ========================================================================
// BUILD TYPST ENGINE
// Pakai .main_file() → menghasilkan TypstEngine<TypstTemplateMainFile>
// compile_with_input pada jenis ini hanya butuh 1 argumen (Dict)
// ========================================================================

fn build_engine() -> TypstEngine<typst_as_lib::TypstTemplateMainFile> {
    TypstEngine::builder()
        .main_file(TEMPLATE)
        .fonts([
            FONT_TNR_REGULAR,
            FONT_TNR_BOLD,
            FONT_TNR_ITALIC,
            FONT_TNR_BOLD_ITALIC,
        ])
        .build()
}

// ========================================================================
// COMMAND 1: Generate SVG pages untuk PREVIEW
// ========================================================================

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SvgPreviewResult {
    pub pages: Vec<String>,
    pub page_count: usize,
}

#[command]
pub fn generate_report_preview(input: PdfReportInput) -> Result<SvgPreviewResult, String> {
    let engine = build_engine();
    let dict = build_typst_dict(&input);

    let doc: PagedDocument = engine
        .compile_with_input(dict)
        .output
        .map_err(|e| format!("Typst compile error: {:?}", e))?;

    let mut svg_pages = Vec::new();
    for page in doc.pages().iter() {
        let svg_str = typst_svg::svg(page, &Default::default());
        svg_pages.push(svg_str);
    }

    let count = svg_pages.len();
    Ok(SvgPreviewResult {
        pages: svg_pages,
        page_count: count,
    })
}

// ========================================================================
// COMMAND 2: Export PDF dengan native save dialog
// ========================================================================

#[command]
pub async fn export_report_pdf(
    input: PdfReportInput,
    app: AppHandle,
) -> Result<String, String> {
    let engine = build_engine();
    let dict = build_typst_dict(&input);

    let doc: PagedDocument = engine
        .compile_with_input(dict)
        .output
        .map_err(|e| format!("Typst compile error: {:?}", e))?;

    let pdf_options = typst_pdf::PdfOptions::default();
    let pdf_bytes = typst_pdf::pdf(&doc, &pdf_options)
        .map_err(|e| format!("PDF render error: {:?}", e))?;

    // Nama file default
    let default_filename = format!("Laporan_{}.pdf", input.period.replace(' ', "_"));

    // Native save dialog dari Tauri
    let save_path = app
        .dialog()
        .file()
        .set_file_name(&default_filename)
        .add_filter("PDF Document", &["pdf"])
        .blocking_save_file();

    match save_path {
        Some(file_path) => {
            use tauri_plugin_fs::{FsExt, OpenOptions};
            use std::io::Write;
            
            let mut opts = OpenOptions::new();
            opts.write(true).create(true).truncate(true);
            
            let mut file = app.fs().open(file_path, opts)
                .map_err(|e| format!("Gagal membuka file: {}", e))?;
                
            file.write_all(&pdf_bytes)
                .map_err(|e| format!("Gagal menyimpan PDF: {}", e))?;
                
            Ok("PDF berhasil disimpan!".to_string())
        }
        None => Err("cancelled".to_string()),
    }
}
