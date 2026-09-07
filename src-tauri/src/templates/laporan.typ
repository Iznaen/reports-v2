// =====================================================================
// Template Laporan Kinerja dan Presensi Bulanan - Desain Modern & Profesional
// Data diinjeksi via sys.inputs dari Rust backend
// =====================================================================

#import sys: inputs

// --- Pengaturan Halaman ---
#set page(
  paper: "a4",
  margin: (top: 2cm, bottom: 2cm, left: 2.5cm, right: 2.5cm),
  fill: rgb("#ffffff"), // latar halaman putih murni
  footer: context [
    #set text(font: ("Inter", "Helvetica", "Arial"), size: 8pt, fill: rgb("#94a3b8"))
    #align(center)[
      #counter(page).display()
    ]
  ]
)
#set text(font: ("Inter", "Helvetica", "Arial", "sans-serif"), size: 10.5pt, lang: "id")
#set par(justify: false)

// --- Palet Warna Modern ---
#let primary       = rgb("#1e3a8a")   // biru tua
#let primary-light = rgb("#3b82f6")   // biru cerah
#let primary-bg    = rgb("#eff6ff")   // latar biru sangat terang
#let accent        = rgb("#0ea5e9")   // aksen biru muda
#let surface       = rgb("#ffffff")   // putih
#let text-dark     = rgb("#0f172a")   // gelap
#let text-muted    = rgb("#64748b")   // abu-abu
#let border-light  = rgb("#e2e8f0")   // garis tipis
#let row-even      = rgb("#f8fafc")   // baris genap
#let row-odd       = rgb("#ffffff")   // baris ganjil

// --- Helper: Kartu (tanpa shadow untuk kompatibilitas) ---
#let card(body, inset: 12pt, padding: 0pt) = {
  block(
    width: 100%,
    fill: surface,
    stroke: 0.5pt + border-light,
    radius: 6pt,
    inset: inset,
  )[#body]
}

// --- Helper: Section Title Modern ---
#let section_title(content) = {
  v(0.8cm)
  block(width: 100%)[
    #text(weight: "bold", size: 12pt, fill: primary)[#content]
    #line(length: 100%, stroke: 2pt + primary-light)
    #v(0.2cm)
  ]
}

// --- Helper: Profile Table (kartu) ---
#let profile_table(name, ni, position, unit) = {
  card[
    #grid(
      columns: (2.8cm, 1fr), 
      row-gutter: 0.2cm,
      column-gutter: 0.2cm,
      text(fill: text-muted, weight: "regular", size: 9.5pt)[Nama],
      [: #text(weight: "bold", size: 10.5pt)[#name]],
      text(fill: text-muted, weight: "regular", size: 9.5pt)[NI / NIP],
      [: #text(weight: "bold", size: 10.5pt)[#ni]],
      text(fill: text-muted, weight: "regular", size: 9.5pt)[Jabatan],
      [: #text(weight: "bold", size: 10.5pt)[#position]],
      text(fill: text-muted, weight: "regular", size: 9.5pt)[Unit Kerja],
      [: #text(weight: "bold", size: 10.5pt)[#unit]],
    )
  ]
  v(0.5cm)
}

// --- Helper: Tanda Tangan dengan gaya modern ---
#let signature_block(date, name, ni, sig_bytes) = {
  v(0.5cm)
  align(right)[
    #block(width: 6cm)[
      #align(center)[
        #text(size: 9pt, fill: text-muted)[#date]
        #v(0.15cm)
        #if sig_bytes != none {
          image(sig_bytes, width: 4.5cm, height: 2.2cm, fit: "contain")
        } else {
          rect(width: 4.5cm, height: 2.2cm, fill: rgb("#f1f5f9"), stroke: 0.5pt + border-light, radius: 4pt)[
            #align(center + horizon)[
              #text(size: 8pt, fill: text-muted, style: "italic")[Belum tersedia]
            ]
          ]
        }
        #v(0.1cm)
        #underline(stroke: 1pt + primary-light, offset: 2pt)[
          #text(weight: "bold", size: 10pt, fill: primary)[#name]
        ]
        #linebreak()
        #text(size: 9pt, fill: text-muted)[NI. #ni]
      ]
    ]
  ]
}

// =====================================================================
// AMBIL DATA DARI BACKEND
// =====================================================================

#let name       = inputs.name
#let ni         = inputs.ni
#let position   = inputs.position
#let unit       = inputs.unit
#let period     = inputs.period
#let att_rows   = inputs.att_rows
#let task_rows  = inputs.task_rows
#let photos     = inputs.photos
#let date_str   = inputs.date_str
#let sig_bytes  = if "sig_bytes" in inputs and inputs.sig_bytes != none { inputs.sig_bytes } else { none }

// =====================================================================
// HEADER DOKUMEN (modern, dengan latar biru)
// =====================================================================

#block(
  width: 100%,
  fill: primary,
  inset: (x: 20pt, y: 12pt),
  radius: 6pt,
)[
  #align(center)[
    #text(weight: "bold", size: 16pt, fill: white)[LAPORAN KINERJA & PRESENSI BULANAN]
    #linebreak()
    #text(size: 11pt, fill: rgb("#bfdbfe"))[Periode #period]
  ]
]
#v(0.2cm)

// =====================================================================
// BAGIAN A: REKAPITULASI PRESENSI
// =====================================================================

#profile_table(name, ni, position, unit)

#section_title[A. Rekapitulasi Presensi]

// Tabel presensi dengan gaya modern
#card(inset: 0pt)[
  #table(
    columns: (0pt, 0.7cm, 1.8cm, 2.5cm, 2cm, 2cm, 2.5cm, 4.5cm),
    stroke: (col, row) => if col == 0 { none } else { 0.5pt + border-light },
    fill: (col, row) => {
      if row == 0 { return primary }
      if row <= att_rows.len() {
        let stat = att_rows.at(row - 1).status
        if stat == "Hadir" { return rgb("#e6f7e6") }
        if stat == "Alpha" { return rgb("#ffe6e6") }
        if stat == "Parsial" { return rgb("#fff8e6") }
        if stat.starts-with("Libur") { return rgb("#f0e6ff") }
        if stat.starts-with("Cuti") { return rgb("#e0f2fe") }
      }
      return if calc.even(row) { row-even } else { row-odd }
    },
    inset: (col, row) => if col == 0 { 0pt } else { (x: 0.3cm, y: 0.3cm) },
    align: (col, row) => if col == 1 or col == 4 or col == 5 { center } else { left },
    table.header(
      [], // dummy col
      text(fill: white, weight: "bold", size: 9.5pt)[No],
      text(fill: white, weight: "bold", size: 9.5pt)[Hari],
      text(fill: white, weight: "bold", size: 9.5pt)[Tanggal],
      text(fill: white, weight: "bold", size: 9.5pt)[Jam Masuk],
      text(fill: white, weight: "bold", size: 9.5pt)[Jam Pulang],
      text(fill: white, weight: "bold", size: 9.5pt)[Durasi (Jam:Menit)],
      text(fill: white, weight: "bold", size: 9.5pt)[Keterangan],
    ),
    ..att_rows.enumerate().map(pair => {
      let (i, r) = pair
      let keep_rows = 3
      let keep_start = calc.max(0, att_rows.len() - keep_rows)
      let is_grouped = (i >= keep_start)
      
      let dummy = if i == keep_start {
        let span_count = att_rows.len() - keep_start + 1
        (table.cell(rowspan: span_count, breakable: false)[],)
      } else if is_grouped {
        ()
      } else {
        ([],)
      }

      dummy + (
        text(size: 9.5pt)[#r.no],
        text(size: 9.5pt)[#r.day],
        text(size: 9.5pt)[#r.date],
        text(size: 9.5pt)[#if r.in_time == "—" { [—] } else { r.in_time }],
        text(size: 9.5pt)[#if r.out_time == "—" { [—] } else { r.out_time }],
        text(size: 9.5pt)[#r.work_hours],
        text(size: 9.5pt)[#r.status],
      )
    }).flatten(),
    // --- PENTING ---
    // TTD sengaja dimasukkan ke dalam baris terakhir tabel (di-wrap dengan table.cell) 
    // bersamaan dengan dummy column hack di atasnya,
    // ini bertujuan untuk mengikat (keep-with-next) TTD agar tidak tercetak sendirian 
    // di halaman kosong (mencegah orphan signature) ketika baris data penuh.
    // Jangan di-ekstrak keluar dari tabel!
    table.cell(colspan: 7, stroke: none, inset: 0pt, fill: rgb("#ffffff"))[
      #signature_block(date_str, name, ni, sig_bytes)
    ]
  )
]

// =====================================================================
// BAGIAN B: LOGBOOK KEGIATAN HARIAN
// =====================================================================

#pagebreak()

#block(
  width: 100%,
  fill: primary,
  inset: (x: 20pt, y: 10pt),
  radius: 6pt,
)[
  #align(center)[
    #text(weight: "bold", size: 14pt, fill: white)[LAPORAN KINERJA & PRESENSI BULANAN]
    #linebreak()
    #text(size: 10pt, fill: rgb("#bfdbfe"))[Periode #period]
  ]
]
#v(0.2cm)

#profile_table(name, ni, position, unit)

#section_title[B. Logbook Kegiatan Harian]

#card(inset: 0pt)[
  #table(
    columns: (0pt, 0.7cm, 2.2cm, 1.8cm, 2.8fr, 2.2cm, 2.5cm),
    stroke: (col, row) => if col == 0 { none } else { 0.5pt + border-light },
    fill: (col, row) => if row == 0 { primary } else if calc.even(row) { row-even } else { row-odd },
    inset: (col, row) => if col == 0 { 0pt } else { (x: 0.3cm, y: 0.3cm) },
    align: (col, row) => if col == 1 or col == 3 { center } else { left },
    table.header(
      [], // dummy col
      text(fill: white, weight: "bold", size: 9.5pt)[No],
      text(fill: white, weight: "bold", size: 9.5pt)[Tanggal],
      text(fill: white, weight: "bold", size: 9.5pt)[Jam],
      text(fill: white, weight: "bold", size: 9.5pt)[Uraian Tugas],
      text(fill: white, weight: "bold", size: 9.5pt)[Output],
      text(fill: white, weight: "bold", size: 9.5pt)[Keterangan],
    ),
    ..task_rows.enumerate().map(pair => {
      let (i, r) = pair
      let keep_rows = 3
      let keep_start = calc.max(0, task_rows.len() - keep_rows)
      let is_grouped = (i >= keep_start)
      
      let dummy = if i == keep_start {
        let span_count = task_rows.len() - keep_start + 1
        (table.cell(rowspan: span_count, breakable: false)[],)
      } else if is_grouped {
        ()
      } else {
        ([],)
      }

      dummy + (
        text(size: 9.5pt)[#r.no],
        text(size: 9.5pt)[#r.date],
        text(size: 9.5pt)[#r.time],
        text(size: 9.5pt)[#r.task],
        text(size: 9.5pt)[#r.output],
        text(size: 9.5pt)[#r.notes],
      )
    }).flatten(),
    // --- PENTING ---
    // Sama seperti tabel presensi, TTD ini diikat ke dalam tabel
    // untuk mencegah orphan signature di halaman baru. Jangan diekstrak!
    table.cell(colspan: 6, stroke: none, inset: 0pt, fill: rgb("#ffffff"))[
      #signature_block(date_str, name, ni, sig_bytes)
    ]
  )
]

// =====================================================================
// BAGIAN C: LAMPIRAN FOTO DOKUMENTASI (dengan gaya modern)
// =====================================================================

#if photos.len() > 0 [
  #pagebreak()

  #block(
    width: 100%,
    fill: primary,
    inset: (x: 20pt, y: 10pt),
    radius: 6pt,
  )[
    #align(center)[
      #text(weight: "bold", size: 14pt, fill: white)[LAPORAN KINERJA & PRESENSI BULANAN]
      #linebreak()
      #text(size: 10pt, fill: rgb("#bfdbfe"))[Periode #period – Lampiran Dokumentasi]
    ]
  ]
  #v(0.2cm)

  #profile_table(name, ni, position, unit)

  #section_title[C. Lampiran Foto Dokumentasi]

  // Layout 2 kolom dengan kartu foto
  #let photo_pairs = {
    let pairs = ()
    let i = 0
    while i < photos.len() {
      if i + 1 < photos.len() {
        pairs = pairs + ((photos.at(i), photos.at(i + 1)),)
      } else {
        pairs = pairs + ((photos.at(i), none),)
      }
      i = i + 2
    }
    pairs
  }

  #for pair in photo_pairs.enumerate() {
    let (i, pair) = pair
    let is_last = (i == photo_pairs.len() - 1)
    
    let content = {
      grid(
        columns: (1fr, 1fr),
        gutter: 0.5cm,
        {
          let p = pair.at(0)
          card[
            #image(p.bytes, width: 100%, fit: "cover", alt: "Dokumentasi")
            #block(width: 100%, inset: 0.3cm, fill: primary-bg)[
              #text(size: 8.5pt, weight: "bold", fill: primary)[#p.photo_type – #p.date_str]
              #linebreak()
              #text(size: 8pt, fill: text-muted)[#p.caption]
            ]
          ]
        },
        {
          if pair.at(1) != none {
            let p = pair.at(1)
            card[
              #image(p.bytes, width: 100%, fit: "cover", alt: "Dokumentasi")
              #block(width: 100%, inset: 0.3cm, fill: primary-bg)[
                #text(size: 8.5pt, weight: "bold", fill: primary)[#p.photo_type – #p.date_str]
                #linebreak()
                #text(size: 8pt, fill: text-muted)[#p.caption]
              ]
            ]
          }
        }
      )
      v(0.5cm)
    }

    if is_last {
      block(breakable: false)[
        #content
        #signature_block(date_str, name, ni, sig_bytes)
      ]
    } else {
      content
    }
  }
]
