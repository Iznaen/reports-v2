use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{to_value, from_value};
use wasm_bindgen::prelude::*;
use crate::components::signature_pad::SignaturePad;
use crate::components::map::MapPicker;

// You can edit this password later!
const ADMIN_PASSWORD: &str = "admin123";

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct EmployeeProfile {
    id: i64,
    name: String,
    ni: String,
    position: String,
    work_unit: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct OfficeLocation {
    id: i64,
    name: String,
    latitude: f64,
    longitude: f64,
    radius_meters: f64,
    is_active: bool,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn Settings() -> impl IntoView {
    // Profile State
    let (profile, set_profile) = signal(EmployeeProfile::default());
    let (locations, set_locations) = signal(Vec::<OfficeLocation>::new());
    
    // Modals State
    let (show_password_modal, set_show_password_modal) = signal(false);
    let (password_input, set_password_input) = signal(String::new());
    let (password_error, set_password_error) = signal(false);
    
    let (show_map, set_show_map) = signal(false);
    let (show_sig_modal, set_show_sig_modal) = signal(false);
    let (show_success_dialog, set_show_success_dialog) = signal(false);
    
    // Initial load - Fetch profile and locations
    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(p_res) = invoke("get_profile", JsValue::NULL).await.dyn_into::<JsValue>() {
            if let Ok(Some(p)) = from_value::<Option<EmployeeProfile>>(p_res) {
                set_profile.set(p);
            }
        }
        
        if let Ok(l_res) = invoke("get_locations", JsValue::NULL).await.dyn_into::<JsValue>() {
            if let Ok(l) = from_value::<Vec<OfficeLocation>>(l_res) {
                set_locations.set(l);
            }
        }
    });

    let save_profile = move |_| {
        let p = profile.get();
        wasm_bindgen_futures::spawn_local(async move {
            #[derive(Serialize)]
            struct Args { profile: EmployeeProfile }
            let args = to_value(&Args { profile: p }).unwrap();
            invoke("save_profile", args).await;
            set_show_success_dialog.set(true);
        });
    };

    let verify_password = move |_| {
        if password_input.get() == ADMIN_PASSWORD {
            set_show_password_modal.set(false);
            set_password_error.set(false);
            set_password_input.set(String::new());
            set_show_map.set(true);
        } else {
            set_password_error.set(true);
        }
    };

    view! {
        <div style="padding: 1rem; position: relative;">
            <h2>"Pengaturan (Settings)"</h2>
            
            <section style="margin-bottom: 2rem;">
                <h3>"Profil Pegawai"</h3>
                <div style="display: flex; flex-direction: column; gap: 0.5rem; max-width: 300px;">
                    <input type="text" placeholder="Nama Lengkap" 
                        prop:value=move || profile.get().name 
                        on:input=move |ev| {
                            let mut p = profile.get();
                            p.name = event_target_value(&ev);
                            set_profile.set(p);
                        } />
                    <input type="text" placeholder="Nomor Induk (NI/NIP)" 
                        prop:value=move || profile.get().ni 
                        on:input=move |ev| {
                            let mut p = profile.get();
                            p.ni = event_target_value(&ev);
                            set_profile.set(p);
                        } />
                    <input type="text" placeholder="Jabatan" 
                        prop:value=move || profile.get().position 
                        on:input=move |ev| {
                            let mut p = profile.get();
                            p.position = event_target_value(&ev);
                            set_profile.set(p);
                        } />
                    <input type="text" placeholder="Unit Kerja" 
                        prop:value=move || profile.get().work_unit 
                        on:input=move |ev| {
                            let mut p = profile.get();
                            p.work_unit = event_target_value(&ev);
                            set_profile.set(p);
                        } />
                    <button on:click=save_profile>"Simpan Profil"</button>
                </div>
            </section>

            <section style="margin-bottom: 2rem;">
                <h3>"Lokasi Kantor (Geofence)"</h3>
                <ul>
                    {move || locations.get().into_iter().map(|loc| {
                        view! {
                            <li>
                                <strong>{loc.name}</strong> 
                                {format!(" (Lat: {}, Lng: {}) - Radius: {}m", loc.latitude, loc.longitude, loc.radius_meters)}
                            </li>
                        }
                    }).collect_view()}
                </ul>
                <button on:click=move |_| set_show_password_modal.set(true)>
                    "Tambah Lokasi"
                </button>
            </section>

            <section style="margin-bottom: 2rem;">
                <h3>"Tanda Tangan Digital"</h3>
                <button on:click=move |_| set_show_sig_modal.set(true)>"Buat TTD"</button>
            </section>

            // --- Signature Modal ---
            <div style=move || if show_sig_modal.get() { "display: block; border: 1px solid black; padding: 1rem; margin-top: 1rem;" } else { "display: none;" }>
                <h4>"Gambar Tanda Tangan"</h4>
                <SignaturePad />
                <button on:click=move |_| set_show_sig_modal.set(false) style="margin-top: 0.5rem;">"Tutup / Selesai"</button>
            </div>

            // --- Password Modal ---
            <div style=move || if show_password_modal.get() { "display: block; border: 1px solid black; padding: 1rem; margin-top: 1rem;" } else { "display: none;" }>
                <h4>"Otorisasi Admin"</h4>
                <p>"Masukkan password untuk menambahkan lokasi kantor baru."</p>
                <input type="password" 
                    placeholder="Password Admin"
                    prop:value=move || password_input.get()
                    on:input=move |ev| set_password_input.set(event_target_value(&ev)) />
                <button on:click=verify_password>"Verifikasi"</button>
                <button on:click=move |_| {
                    set_show_password_modal.set(false);
                    set_password_error.set(false);
                }>"Batal"</button>
                <p style="color: red; display: flex;">
                    {move || if password_error.get() { "Password salah!" } else { "" }}
                </p>
            </div>

            // --- Map UI (Task 2.2 Leaflet) ---
            <div style=move || if show_map.get() { "display: block; border: 1px solid blue; padding: 1rem; margin-top: 1rem;" } else { "display: none;" }>
                <h4>"Pilih Lokasi Kantor"</h4>
                <MapPicker />
                <button on:click=move |_| set_show_map.set(false) style="margin-top: 0.5rem;">"Batal / Tutup Peta"</button>
            </div>

            // --- Success Dialog (Bug 2 Fix) ---
            <div style=move || if show_success_dialog.get() { "display: flex; align-items: center; justify-content: center; position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.5); z-index: 1000;" } else { "display: none;" }>
                <div style="background: white; padding: 2rem; width: 80%; max-width: 300px; border-radius: 8px; text-align: center; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">
                    <h4 style="margin-top: 0; color: #1a3a5c; font-size: 1.25rem;">"Berhasil"</h4>
                    <p style="color: #333; margin-bottom: 1.5rem;">"Profil berhasil disimpan!"</p>
                    <button 
                        on:click=move |_| set_show_success_dialog.set(false)
                        style="background: #1a3a5c; color: white; border: none; padding: 0.5rem 1.5rem; border-radius: 4px; cursor: pointer; font-weight: bold;"
                    >
                        "OK"
                    </button>
                </div>
            </div>
        </div>
    }
}
