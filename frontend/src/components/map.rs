use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = window)]
    fn initLeafletMap(id: &str);

    #[wasm_bindgen(js_namespace = window)]
    fn getMapSelection() -> JsValue;
}

#[component]
pub fn MapPicker(on_success: Callback<()>) -> impl IntoView {
    Effect::new(move |_| {
        initLeafletMap("leaflet-map");
    });

    view! {
        <div style="display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 400px; position: relative;">
            <div id="leaflet-map" style="width: 100%; height: 350px; background: #eaeaea; border-radius: 8px; border: 1px solid #cbd5e1; z-index: 1;">
            </div>
            <div style="font-size: 12px; color: #64748b; margin-top: -4px;">
                <i class="fas fa-info-circle"></i> "Geser marker ke lokasi yang tepat"
            </div>
            <button 
                style="background: #1a3a5c; color: white; border: none; padding: 12px; border-radius: 8px; font-weight: 600; cursor: pointer; margin-top: 8px;"
                on:click=move |_| {
                    let coords = getMapSelection();
                    if !coords.is_null() && !coords.is_undefined() {
                        if let Ok(arr) = coords.dyn_into::<js_sys::Array>() {
                            let lat = arr.get(0).as_f64().unwrap_or(0.0);
                            let lng = arr.get(1).as_f64().unwrap_or(0.0);
                            
                            wasm_bindgen_futures::spawn_local(async move {
                                #[derive(serde::Serialize)]
                                struct OfficeLocationInput {
                                    id: i64,
                                    name: String,
                                    latitude: f64,
                                    longitude: f64,
                                    radius_meters: f64,
                                    is_active: bool,
                                }
                                
                                #[derive(serde::Serialize)]
                                struct Args {
                                    location: OfficeLocationInput,
                                }
                                
                                let args = serde_wasm_bindgen::to_value(&Args {
                                    location: OfficeLocationInput {
                                        id: 0,
                                        name: "Lokasi Kustom".to_string(), // In real app, prompt user for name
                                        latitude: lat,
                                        longitude: lng,
                                        radius_meters: 50.0,
                                        is_active: true,
                                    }
                                }).unwrap();
                                
                                invoke("add_location", args).await;
                                on_success.run(());
                            });
                        }
                    }
                }
            >
                <i class="fas fa-save" style="margin-right: 8px;"></i>"Simpan Lokasi Ini"
            </button>
        </div>
    }
}
