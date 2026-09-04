use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn SignaturePad() -> impl IntoView {
    view! {
        <div style="display: flex; flex-direction: column; gap: 0.5rem; width: 300px;">
            <div style="border: 1px solid #333; background: #f9f9f9;">
                <canvas id="signature-canvas" width="300" height="150"></canvas>
            </div>
            <div style="display: flex; gap: 0.5rem;">
                <button on:click=move |_| {
                    if let Some(window) = web_sys::window() {
                        if let Some(doc) = window.document() {
                            if let Some(canvas) = doc.get_element_by_id("signature-canvas") {
                                if let Ok(canvas_el) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
                                    let ctx = canvas_el.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
                                    ctx.clear_rect(0.0, 0.0, 300.0, 150.0);
                                }
                            }
                        }
                    }
                }>"Bersihkan"</button>
                <button on:click=move |_| {
                    // In a real app we'd bind mouse/touch events to the canvas.
                    // For this MVP placeholder, we just grab the blank Data URL and send it to Tauri.
                    if let Some(window) = web_sys::window() {
                        if let Some(doc) = window.document() {
                            if let Some(canvas) = doc.get_element_by_id("signature-canvas") {
                                if let Ok(canvas_el) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
                                    if let Ok(data_url) = canvas_el.to_data_url() {
                                        wasm_bindgen_futures::spawn_local(async move {
                                            #[derive(serde::Serialize)]
                                            struct Args { base64_data: String }
                                            let args = serde_wasm_bindgen::to_value(&Args { base64_data: data_url }).unwrap();
                                            invoke("save_signature", args).await;
                                        });
                                    }
                                }
                            }
                        }
                    }
                }>"Simpan Tanda Tangan"</button>
            </div>
        </div>
    }
}
