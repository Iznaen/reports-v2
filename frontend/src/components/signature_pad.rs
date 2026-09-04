use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = window)]
    fn initSignaturePad(id: &str);
}

#[component]
pub fn SignaturePad(on_save: WriteSignal<Option<String>>) -> impl IntoView {
    let (saved, set_saved) = signal(false);

    Effect::new(move |_| {
        initSignaturePad("signature-canvas");
    });

    view! {
        <div style="display: flex; flex-direction: column; gap: 0.5rem; width: 100%; align-items: center;">
            <div style="border: 1px solid #cbd5e1; background: #f8fafc; border-radius: 8px; overflow: hidden; width: 300px;">
                <canvas id="signature-canvas" width="300" height="150" style="touch-action: none; cursor: crosshair; display: block;"></canvas>
            </div>
            <div style="display: flex; gap: 8px; width: 300px; justify-content: space-between;">
                <button style="flex: 1; background: #e2e8f0; color: #334155; border: none; padding: 10px; border-radius: 8px; font-weight: 600; cursor: pointer;" on:click=move |_| {
                    if let Some(window) = web_sys::window() {
                        if let Some(doc) = window.document() {
                            if let Some(canvas) = doc.get_element_by_id("signature-canvas") {
                                if let Ok(canvas_el) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
                                    let ctx = canvas_el.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
                                    ctx.clear_rect(0.0, 0.0, 300.0, 150.0);
                                    set_saved.set(false);
                                }
                            }
                        }
                    }
                }>"Bersihkan"</button>
                <button style="flex: 1; background: #1a3a5c; color: #ffffff; border: none; padding: 10px; border-radius: 8px; font-weight: 600; cursor: pointer;" on:click=move |_| {
                    // Send Base64 data to Tauri backend
                    if let Some(window) = web_sys::window() {
                        if let Some(doc) = window.document() {
                            if let Some(canvas) = doc.get_element_by_id("signature-canvas") {
                                if let Ok(canvas_el) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
                                    if let Ok(data_url) = canvas_el.to_data_url() {
                                        let data_url_clone = data_url.clone();
                                        wasm_bindgen_futures::spawn_local(async move {
                                            #[derive(serde::Serialize)]
                                            #[serde(rename_all = "camelCase")]
                                            struct Args { base64_data: String }
                                            let args = serde_wasm_bindgen::to_value(&Args { base64_data: data_url }).unwrap();
                                            invoke("save_signature", args).await;
                                            set_saved.set(true);
                                            on_save.set(Some(data_url_clone));
                                            
                                            set_timeout(move || {
                                                set_saved.set(false);
                                            }, std::time::Duration::from_millis(1500));
                                        });
                                    }
                                }
                            }
                        }
                    }
                }>"Simpan"</button>
            </div>
            <div style="height: 20px; font-size: 12px; color: #16a34a; font-weight: 600;">
                {move || if saved.get() { "✓ Tanda tangan berhasil disimpan!" } else { "" }}
            </div>
        </div>
    }
}
