use leptos::prelude::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <main style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; font-family: sans-serif;">
            <h1>"Hello World - reports-v2"</h1>
            <p>"Tauri v2 + Leptos (Trunk) is running!"</p>
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
