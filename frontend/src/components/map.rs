use leptos::prelude::*;

#[component]
pub fn MapPicker() -> impl IntoView {
    view! {
        <div style="display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 400px;">
            <div id="leaflet-map" style="width: 100%; height: 300px; background: #eaeaea; border: 1px solid #ccc; display: flex; align-items: center; justify-content: center;">
                <p style="color: #666;">"(Leaflet Map Initialization)"</p>
            </div>
            <button on:click=move |_| {
                // Here we would extract the lat/lng from the Leaflet marker
                // and call invoke("add_location")
            }>"Simpan Lokasi Ini"</button>
        </div>
    }
}
