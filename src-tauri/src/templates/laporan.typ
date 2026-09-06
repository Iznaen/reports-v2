// =====================================================================
// Template Laporan Kinerja dan Presensi Bulanan
// Data diinjeksi via sys.inputs dari Rust backend
// =====================================================================

#import sys: inputs

// --- Halaman Setup ---
#set page(
  paper: "a4",
  margin: (top: 1.5cm, bottom: 2cm, left: 2.5cm, right: 2cm),
)
#set text(font: "Times New Roman", size: 10pt, lang: "id")
#set par(justify: false)

// --- Style Helpers ---
#let header_fill = rgb("#2563eb")
#let header_text_color = white
#let row_alt_fill = rgb("#f8fafc")
#let border_color = rgb("#cbd5e1")

#let section_title(content) = [
  #v(0.4cm)
  #text(weight: "bold", size: 10.5pt)[#content]
  #v(0.15cm)
]

#let profile_table(name, ni, position, unit) = {
  grid(
    columns: (1fr, 1fr),
    gutter: 0.3cm,
    [
      #grid(columns: (3cm, auto), gutter: 0.2cm,
        [*Nama*], [: #name],
        [*NI / NIP*], [: #ni],
      )
    ],
    [
      #grid(columns: (3cm, auto), gutter: 0.2cm,
        [*Jabatan*], [: #position],
        [*Unit Kerja*], [: #unit],
      )
    ],
  )
  v(0.3cm)
}

#let signature_block(date, name, ni, sig_bytes) = {
  v(0.5cm)
  align(right)[
    #block(width: 5.5cm)[
      #align(center)[
        #text(size: 9pt)[#date]
        #v(0.1cm)
        #if sig_bytes != none {
          image(sig_bytes, width: 4cm, height: 2cm, fit: "contain")
        } else {
          rect(width: 4cm, height: 2cm, stroke: 0.5pt + border_color)[
            #align(center + horizon)[
              #text(size: 8pt, fill: rgb("#94a3b8"), style: "italic")[(Belum ada ttd)]
            ]
          ]
        }
        #v(0.1cm)
        #underline(text(weight: "bold", size: 9pt)[#name])
        #linebreak()
        #text(size: 9pt)[NI. #ni]
      ]
    ]
  ]
}

// =====================================================================
// AMBIL DATA
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
// BAGIAN A: REKAPITULASI PRESENSI
// =====================================================================

// --- Header Dokumen ---
#align(center)[
  #text(weight: "bold", size: 12pt)[LAPORAN KINERJA DAN PRESENSI BULANAN]
  #linebreak()
  #text(size: 10pt)[Periode #period]
]
#v(0.5cm)
#line(length: 100%, stroke: 1pt + black)
#v(0.3cm)

#profile_table(name, ni, position, unit)

#section_title[A. Rekapitulasi Presensi]

#table(
  columns: (0pt, 0.7cm, 1.4cm, 2cm, 1.8cm, 3cm, 1.8cm, 3cm, 2.5cm),
  stroke: (col, row) => if col == 0 { none } else { 0.5pt + border_color },
  fill: (col, row) => if row == 0 { header_fill } else if calc.odd(row) { row_alt_fill } else { white },
  inset: (col, row) => if col == 0 { 0pt } else { (x: 0.3cm, y: 0.25cm) },
  align: (col, row) => if col == 1 or col == 4 or col == 6 { center } else { left },
  table.header(
    [], // dummy col
    text(fill: white, weight: "bold", size: 9pt)[No],
    text(fill: white, weight: "bold", size: 9pt)[Hari],
    text(fill: white, weight: "bold", size: 9pt)[Tanggal],
    text(fill: white, weight: "bold", size: 9pt)[Jam Masuk],
    text(fill: white, weight: "bold", size: 9pt)[Lokasi Masuk],
    text(fill: white, weight: "bold", size: 9pt)[Jam Pulang],
    text(fill: white, weight: "bold", size: 9pt)[Lokasi Pulang],
    text(fill: white, weight: "bold", size: 9pt)[Keterangan],
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
      text(size: 9pt)[#r.no],
      text(size: 9pt)[#r.day],
      text(size: 9pt)[#r.date],
      text(size: 9pt)[#r.in_time],
      text(size: 9pt)[#r.in_loc],
      text(size: 9pt)[#r.out_time],
      text(size: 9pt)[#r.out_loc],
      text(size: 9pt)[#r.status],
    )
  }).flatten(),
  table.cell(colspan: 8, stroke: none, inset: 0pt)[
    #signature_block(date_str, name, ni, sig_bytes)
  ]
)

// =====================================================================
// BAGIAN B: LOGBOOK KEGIATAN HARIAN
// =====================================================================

#pagebreak()

#align(center)[
  #text(weight: "bold", size: 12pt)[LAPORAN KINERJA DAN PRESENSI BULANAN]
  #linebreak()
  #text(size: 10pt)[Periode #period]
]
#v(0.5cm)
#line(length: 100%, stroke: 1pt + black)
#v(0.3cm)

#profile_table(name, ni, position, unit)

#section_title[B. Logbook Kegiatan Harian]

#table(
  columns: (0pt, 0.7cm, 2.2cm, 1.5cm, 1fr, 3cm, 2.5cm),
  stroke: (col, row) => if col == 0 { none } else { 0.5pt + border_color },
  fill: (col, row) => if row == 0 { header_fill } else if calc.odd(row) { row_alt_fill } else { white },
  inset: (col, row) => if col == 0 { 0pt } else { (x: 0.3cm, y: 0.25cm) },
  align: (col, row) => if col == 1 or col == 3 { center } else { left },
  table.header(
    [], // dummy col
    text(fill: white, weight: "bold", size: 9pt)[No],
    text(fill: white, weight: "bold", size: 9pt)[Tanggal],
    text(fill: white, weight: "bold", size: 9pt)[Jam],
    text(fill: white, weight: "bold", size: 9pt)[Uraian Tugas],
    text(fill: white, weight: "bold", size: 9pt)[Output],
    text(fill: white, weight: "bold", size: 9pt)[Keterangan],
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
      text(size: 9pt)[#r.no],
      text(size: 9pt)[#r.date],
      text(size: 9pt)[#r.time],
      text(size: 9pt)[#r.task],
      text(size: 9pt)[#r.output],
      text(size: 9pt)[#r.notes],
    )
  }).flatten(),
  table.cell(colspan: 6, stroke: none, inset: 0pt)[
    #signature_block(date_str, name, ni, sig_bytes)
  ]
)

// =====================================================================
// BAGIAN C: LAMPIRAN FOTO DOKUMENTASI
// =====================================================================

#if photos.len() > 0 [
  #pagebreak()

  #align(center)[
    #text(weight: "bold", size: 12pt)[LAPORAN KINERJA DAN PRESENSI BULANAN]
    #linebreak()
    #text(size: 10pt)[Periode #period – Lampiran Dokumentasi]
  ]
  #v(0.5cm)
  #line(length: 100%, stroke: 1pt + black)
  #v(0.3cm)

  #profile_table(name, ni, position, unit)

  #section_title[C. Lampiran Foto Dokumentasi]

  // Layout 2 kolom untuk foto
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
        gutter: 0.4cm,
        {
          let p = pair.at(0)
          block(stroke: 0.5pt + border_color, radius: 3pt, clip: true, width: 100%)[
            #image(p.bytes, width: 100%, fit: "cover")
            #block(width: 100%, inset: 0.25cm, fill: rgb("#f1f5f9"))[
              #text(size: 8pt, weight: "bold")[#p.photo_type – #p.date_str]
              #linebreak()
              #text(size: 7.5pt)[#p.caption]
            ]
          ]
        },
        {
          if pair.at(1) != none {
            let p = pair.at(1)
            block(stroke: 0.5pt + border_color, radius: 3pt, clip: true, width: 100%)[
              #image(p.bytes, width: 100%, fit: "cover")
              #block(width: 100%, inset: 0.25cm, fill: rgb("#f1f5f9"))[
                #text(size: 8pt, weight: "bold")[#p.photo_type – #p.date_str]
                #linebreak()
                #text(size: 7.5pt)[#p.caption]
              ]
            ]
          }
        }
      )
      v(0.3cm)
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
