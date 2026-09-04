use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use crate::pages::{home::Home, settings::Settings};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <AppLayout />
        </Router>
    }
}

#[component]
fn AppLayout() -> impl IntoView {
    view! {
        <main style="padding-top: max(env(safe-area-inset-top), 40px); padding-bottom: max(env(safe-area-inset-bottom), 20px); height: 100vh; display: flex; flex-direction: column; overflow: hidden; box-sizing: border-box;">
            
            // Content Area (scrollable)
            <div style="flex: 1; overflow-y: auto;">
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/settings") view=Settings />
                </Routes>
            </div>

            // Bottom Navigation Bar
            <nav style="display: flex; justify-content: space-around; background: #ffffff; border-top: 1px solid #edf2f7; padding: 10px 0 14px 0;">
                <a 
                    href="/"
                    style="display: flex; flex-direction: column; align-items: center; gap: 4px; text-decoration: none; color: #1a3a5c;"
                >
                    <i class="fas fa-home" style="font-size: 20px;"></i>
                    <span style="font-size: 12px; font-weight: 600;">"Beranda"</span>
                </a>
                
                <a 
                    href="/settings"
                    style="display: flex; flex-direction: column; align-items: center; gap: 4px; text-decoration: none; color: #1a3a5c;"
                >
                    <i class="fas fa-cog" style="font-size: 20px;"></i>
                    <span style="font-size: 12px; font-weight: 600;">"Pengaturan"</span>
                </a>
            </nav>
        </main>
    }
}
