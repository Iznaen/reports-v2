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

    #[wasm_bindgen(js_namespace = window)]
    fn resetLeafletMap();
}

#[component]
pub fn Settings() -> impl IntoView {
    // Profile State
    let (profile, set_profile) = signal(EmployeeProfile::default());
    let (locations, set_locations) = signal(Vec::<OfficeLocation>::new());
    let (saved_signature, set_saved_signature) = signal::<Option<String>>(None);
    
    // Location Editing State
    let (editing_loc_id, set_editing_loc_id) = signal::<Option<i64>>(None);
    let (edit_loc_name, set_edit_loc_name) = signal(String::new());
    
    // Modals State
    let (show_password_modal, set_show_password_modal) = signal(false);
    let (password_input, set_password_input) = signal(String::new());
    let (password_error, set_password_error) = signal(false);
    
    let (show_map, set_show_map) = signal(false);
    let (show_sig_modal, set_show_sig_modal) = signal(false);
    let (show_success_dialog, set_show_success_dialog) = signal(false);
    let (show_info_modal, set_show_info_modal) = signal(false);
    
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

        if let Ok(sig_res) = invoke("get_signature", JsValue::NULL).await.dyn_into::<JsValue>() {
            if let Ok(Some(sig)) = from_value::<Option<String>>(sig_res) {
                set_saved_signature.set(Some(sig));
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
            resetLeafletMap();
        } else {
            set_password_error.set(true);
        }
    };

    view! {
        <div style="background: #f8fafc; min-height: 100%; position: relative;">
            // --- Header ---
            <div style="position: sticky; top: 0; z-index: 50; background: #1a3a5c; padding: 20px; color: #fff; display: flex; align-items: center; justify-content: space-between; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                <h1 style="font-size: 20px; font-weight: 600; margin: 0; display: flex; align-items: center; gap: 8px;">
                    <i class="fas fa-cog"></i> "Pengaturan"
                </h1>
                <i 
                    class="fas fa-info-circle" 
                    style="font-size: 20px; color: #94a3b8; cursor: pointer;"
                    on:click=move |_| set_show_info_modal.set(true)
                ></i>
            </div>
            
            <div style="padding: 20px;">
                // --- Profil Pegawai ---
                <section style="margin-bottom: 24px;">
                    <div style="font-size: 16px; font-weight: 600; color: #0f172a; margin-bottom: 12px; display: flex; align-items: center; gap: 8px;">
                        <i class="fas fa-user-cog" style="color: #1a3a5c; font-size: 18px;"></i>
                        "Profil Pegawai"
                    </div>
                    <div style="background: #ffffff; border-radius: 16px; padding: 16px; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.03); border: 1px solid #edf2f7; display: flex; flex-direction: column; gap: 12px;">
                        <div style="display: flex; flex-direction: column; gap: 4px;">
                            <label style="font-size: 12px; color: #64748b; font-weight: 500;">"Nama Lengkap"</label>
                            <input type="text" placeholder="Nama Lengkap" 
                                style="border: 1px solid #e2e8f0; border-radius: 8px; padding: 10px 12px; font-size: 14px; outline: none; background: #fff; color: #1e293b;"
                                prop:value=move || profile.get().name 
                                on:input=move |ev| {
                                    let mut p = profile.get();
                                    p.name = event_target_value(&ev);
                                    set_profile.set(p);
                                } />
                        </div>
                        <div style="display: flex; flex-direction: column; gap: 4px;">
                            <label style="font-size: 12px; color: #64748b; font-weight: 500;">"Nomor Induk (NI/NIP)"</label>
                            <input type="text" placeholder="Nomor Induk" 
                                style="border: 1px solid #e2e8f0; border-radius: 8px; padding: 10px 12px; font-size: 14px; outline: none; background: #fff; color: #1e293b;"
                                prop:value=move || profile.get().ni 
                                on:input=move |ev| {
                                    let mut p = profile.get();
                                    p.ni = event_target_value(&ev);
                                    set_profile.set(p);
                                } />
                        </div>
                        <div style="display: flex; flex-direction: column; gap: 4px;">
                            <label style="font-size: 12px; color: #64748b; font-weight: 500;">"Jabatan"</label>
                            <input type="text" placeholder="Jabatan" 
                                style="border: 1px solid #e2e8f0; border-radius: 8px; padding: 10px 12px; font-size: 14px; outline: none; background: #fff; color: #1e293b;"
                                prop:value=move || profile.get().position 
                                on:input=move |ev| {
                                    let mut p = profile.get();
                                    p.position = event_target_value(&ev);
                                    set_profile.set(p);
                                } />
                        </div>
                        <div style="display: flex; flex-direction: column; gap: 4px;">
                            <label style="font-size: 12px; color: #64748b; font-weight: 500;">"Unit Kerja"</label>
                            <input type="text" placeholder="Unit Kerja" 
                                style="border: 1px solid #e2e8f0; border-radius: 8px; padding: 10px 12px; font-size: 14px; outline: none; background: #fff; color: #1e293b;"
                                prop:value=move || profile.get().work_unit 
                                on:input=move |ev| {
                                    let mut p = profile.get();
                                    p.work_unit = event_target_value(&ev);
                                    set_profile.set(p);
                                } />
                        </div>
                        <button on:click=save_profile style="background: #1a3a5c; color: white; border: none; padding: 12px; border-radius: 8px; font-weight: 600; cursor: pointer; margin-top: 8px;">
                            <i class="fas fa-save" style="margin-right: 8px;"></i>"Simpan Profil"
                        </button>
                    </div>
                </section>

                // --- Lokasi Kantor ---
                <section style="margin-bottom: 24px;">
                    <div style="font-size: 16px; font-weight: 600; color: #0f172a; margin-bottom: 12px; display: flex; align-items: center; gap: 8px;">
                        <i class="fas fa-map-marker-alt" style="color: #1a3a5c; font-size: 18px;"></i>
                        "Lokasi Kantor"
                    </div>
                    <div style="margin-bottom: 12px;">
                        {move || locations.get().into_iter().map(|loc| {
                            let loc_id = loc.id;
                            let loc_name = loc.name.clone();
                            let loc_is_active = loc.is_active;
                            let loc_clone = loc.clone();
                            
                            view! {
                                <div style="background: #fff; border: 1px solid #edf2f7; border-radius: 12px; padding: 12px; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
                                    {move || {
                                        if editing_loc_id.get() == Some(loc_id) {
                                            view! {
                                                <div style="display: flex; flex-direction: column; gap: 8px; flex: 1; margin-right: 12px;">
                                                    <div style="display: flex; gap: 4px;">
                                                        <input 
                                                            type="text" 
                                                            prop:value=edit_loc_name.get()
                                                            on:input=move |e| set_edit_loc_name.set(event_target_value(&e))
                                                            style="flex: 1; padding: 4px 8px; border: 1px solid #cbd5e1; border-radius: 4px; font-size: 14px;"
                                                        />
                                                        <button 
                                                            style="background: #16a34a; color: white; border: none; border-radius: 4px; padding: 4px 10px; cursor: pointer;"
                                                            on:click=move |_| {
                                                                let new_name = edit_loc_name.get();
                                                                let id = loc_id;
                                                                wasm_bindgen_futures::spawn_local(async move {
                                                                    #[derive(serde::Serialize)]
                                                                    #[serde(rename_all = "camelCase")]
                                                                    struct Args { id: i64, new_name: String }
                                                                    let args = to_value(&Args { id, new_name }).unwrap();
                                                                    invoke("update_location_name", args).await;
                                                                    
                                                                    set_editing_loc_id.set(None);
                                                                    if let Ok(l_res) = invoke("get_locations", JsValue::NULL).await.dyn_into::<JsValue>() {
                                                                        if let Ok(l) = from_value::<Vec<OfficeLocation>>(l_res) {
                                                                            set_locations.set(l);
                                                                        }
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            <i class="fas fa-check"></i>
                                                        </button>
                                                        <button 
                                                            style="background: #ef4444; color: white; border: none; border-radius: 4px; padding: 4px 10px; cursor: pointer;"
                                                            on:click=move |_| set_editing_loc_id.set(None)
                                                        >
                                                            <i class="fas fa-times"></i>
                                                        </button>
                                                    </div>
                                                    <div style="font-size: 12px; color: #64748b;">
                                                        {format!("{}, {} • Radius {}m", loc_clone.latitude, loc_clone.longitude, loc_clone.radius_meters)}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            let name_for_edit = loc_name.clone();
                                            view! {
                                                <div style="flex: 1;">
                                                    <div style="font-weight: 600; font-size: 14px; color: #0f172a; margin-bottom: 2px; display: flex; align-items: center; gap: 8px;">
                                                        {loc_name.clone()}
                                                        <i class="fas fa-pen" 
                                                           style="color: #94a3b8; font-size: 12px; cursor: pointer; padding: 2px;"
                                                           on:click=move |_| {
                                                               set_edit_loc_name.set(name_for_edit.clone());
                                                               set_editing_loc_id.set(Some(loc_id));
                                                           }
                                                        ></i>
                                                    </div>
                                                    <div style="font-size: 12px; color: #64748b;">
                                                        {format!("{}, {} • Radius {}m", loc_clone.latitude, loc_clone.longitude, loc_clone.radius_meters)}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                    <div style="display: flex; align-items: center; gap: 8px;">
                                        <span style=move || if loc_is_active { "background: #e0f2fe; color: #0284c7; font-size: 11px; padding: 4px 10px; border-radius: 100px; font-weight: 600;" } else { "background: #f1f5f9; color: #64748b; font-size: 11px; padding: 4px 10px; border-radius: 100px; font-weight: 600;" }>
                                            {if loc_is_active { "Aktif" } else { "Inaktif" }}
                                        </span>
                                        {move || {
                                            if locations.get().len() > 1 {
                                                view! {
                                                    <i 
                                                        class="fas fa-trash-alt" 
                                                        style="color: #ef4444; font-size: 14px; cursor: pointer; padding: 4px;"
                                                        on:click=move |_| {
                                                            let id = loc_id;
                                                            wasm_bindgen_futures::spawn_local(async move {
                                                                #[derive(serde::Serialize)]
                                                                #[serde(rename_all = "camelCase")]
                                                                struct Args { id: i64 }
                                                                let args = to_value(&Args { id }).unwrap();
                                                                invoke("delete_location", args).await;
                                                                
                                                                if let Ok(l_res) = invoke("get_locations", JsValue::NULL).await.dyn_into::<JsValue>() {
                                                                    if let Ok(l) = from_value::<Vec<OfficeLocation>>(l_res) {
                                                                        set_locations.set(l);
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    ></i>
                                                }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }
                                        }}
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                    <div style="display: flex; gap: 8px;">
                        <button on:click=move |_| set_show_password_modal.set(true) style="background: #e9edf2; color: #334155; border: none; padding: 8px 16px; border-radius: 100px; font-size: 12px; font-weight: 600; cursor: pointer; display: flex; align-items: center; gap: 6px;">
                            <i class="fas fa-plus"></i> "Tambah Lokasi"
                        </button>
                        <span style="background: #f1f5f9; color: #64748b; border: none; padding: 8px 16px; border-radius: 100px; font-size: 12px; font-weight: 600; display: flex; align-items: center; gap: 6px;">
                            <i class="fas fa-lock"></i> "Admin"
                        </span>
                    </div>
                </section>

                // --- Tanda Tangan Digital ---
                <section style="margin-bottom: 24px;">
                    <div style="font-size: 16px; font-weight: 600; color: #0f172a; margin-bottom: 12px; display: flex; align-items: center; gap: 8px;">
                        <i class="fas fa-pen-fancy" style="color: #1a3a5c; font-size: 18px;"></i>
                        "Tanda Tangan Digital"
                    </div>
                    <div on:click=move |_| set_show_sig_modal.set(true) style="background: #fff; border: 1px dashed #cbd5e1; border-radius: 16px; padding: 20px; text-align: center; cursor: pointer; display: flex; flex-direction: column; align-items: center; justify-content: center;">
                        {move || {
                            if let Some(b64) = saved_signature.get() {
                                view! {
                                    <img src=b64 style="max-width: 100%; max-height: 100px; object-fit: contain; margin-bottom: 8px;" />
                                    <div style="font-size: 12px; color: #94a3b8; margin-top: 4px;">"Klik untuk memperbarui"</div>
                                }.into_any()
                            } else {
                                view! {
                                    <i class="fas fa-draw-polygon" style="font-size: 24px; color: #1a3a5c; margin-bottom: 8px; display: block;"></i>
                                    <div style="font-size: 14px; font-weight: 500; color: #334155;">"(Tanda tangan)"</div>
                                    <div style="font-size: 12px; color: #94a3b8; margin-top: 4px;">"Klik untuk membuat"</div>
                                }.into_any()
                            }
                        }}
                    </div>
                </section>

                // --- Signature Modal ---
                <div style=move || if show_sig_modal.get() { "display: block; border: 1px solid black; padding: 1rem; margin-top: 1rem; background: #fff;" } else { "display: none;" }>
                    <h4 style="margin-top: 0;">"Gambar Tanda Tangan"</h4>
                    <SignaturePad on_save=set_saved_signature />
                    <button on:click=move |_| set_show_sig_modal.set(false) style="margin-top: 0.5rem; background: #e2e8f0; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;">"Tutup / Selesai"</button>
                </div>

                // --- Password Modal ---
                <div style=move || if show_password_modal.get() { "display: block; border: 1px solid black; padding: 1rem; margin-top: 1rem; background: #fff;" } else { "display: none;" }>
                    <h4 style="margin-top: 0;">"Otorisasi Admin"</h4>
                    <p style="font-size: 14px; color: #64748b;">"Masukkan password untuk menambahkan lokasi kantor baru."</p>
                    <input type="password" 
                        placeholder="Password Admin"
                        style="border: 1px solid #cbd5e1; padding: 8px; border-radius: 4px; width: 100%; margin-bottom: 8px;"
                        prop:value=move || password_input.get()
                        on:input=move |ev| set_password_input.set(event_target_value(&ev)) />
                    <div style="display: flex; gap: 8px;">
                        <button on:click=verify_password style="background: #1a3a5c; color: white; border: none; padding: 8px 16px; border-radius: 4px;">"Verifikasi"</button>
                        <button on:click=move |_| {
                            set_show_password_modal.set(false);
                            set_password_error.set(false);
                        } style="background: #e2e8f0; border: none; padding: 8px 16px; border-radius: 4px;">"Batal"</button>
                    </div>
                    <p style="color: red; font-size: 12px; margin-top: 8px;">
                        {move || if password_error.get() { "Password salah!" } else { "" }}
                    </p>
                </div>

                // --- Map UI (Task 2.2 Leaflet) ---
                <div style=move || if show_map.get() { "display: block; border: 1px solid blue; padding: 1rem; margin-top: 1rem; background: #fff;" } else { "display: none;" }>
                    <h4 style="margin-top: 0;">"Pilih Lokasi Kantor"</h4>
                    <MapPicker on_success=Callback::new(move |()| {
                        set_show_map.set(false);
                        wasm_bindgen_futures::spawn_local(async move {
                            if let Ok(l_res) = invoke("get_locations", JsValue::NULL).await.dyn_into::<JsValue>() {
                                if let Ok(l) = from_value::<Vec<OfficeLocation>>(l_res) {
                                    set_locations.set(l);
                                }
                            }
                        });
                    }) />
                    <button on:click=move |_| set_show_map.set(false) style="margin-top: 0.5rem; background: #e2e8f0; border: none; padding: 8px 16px; border-radius: 4px;">"Batal / Tutup Peta"</button>
                </div>
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

            // --- Info Modal ---
            <div style=move || if show_info_modal.get() { "display: flex; align-items: center; justify-content: center; position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.5); z-index: 1000;" } else { "display: none;" }>
                <div style="background: white; padding: 2rem; width: 80%; max-width: 300px; border-radius: 16px; text-align: center; box-shadow: 0 10px 25px rgba(0,0,0,0.1);">
                    <i class="fas fa-info-circle" style="font-size: 48px; color: #3b82f6; margin-bottom: 16px;"></i>
                    <h3 style="margin: 0 0 8px 0; color: #0f172a;">"Info Pengembang"</h3>
                    <p style="margin: 0 0 24px 0; color: #64748b; font-size: 14px;">"Pengembang: Mohammad Iznaen Tanggapili"</p>
                    <button 
                        on:click=move |_| set_show_info_modal.set(false)
                        style="background: #1a3a5c; color: white; border: none; padding: 12px 24px; border-radius: 8px; font-weight: 600; width: 100%; cursor: pointer;"
                    >
                        "Tutup"
                    </button>
                </div>
            </div>
        </div>
    }
}
