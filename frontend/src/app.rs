use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use crate::pages::{home::Home, settings::Settings, report::Report, report_print::ReportPrint, dev::Dev};

/// Global reactive signal for Dev Mode state.
#[derive(Clone, Copy)]
pub struct DevMode(pub RwSignal<bool>);

#[component]
pub fn App() -> impl IntoView {
    let dev_mode = RwSignal::new(false);
    provide_context(DevMode(dev_mode));

    view! {
        <Router>
            <AppLayout />
        </Router>
    }
}

#[component]
fn AppLayout() -> impl IntoView {
    let dev_mode = use_context::<DevMode>().expect("DevMode context missing");

    view! {
        <main style="padding-top: max(env(safe-area-inset-top), 40px); padding-bottom: max(env(safe-area-inset-bottom), 20px); height: 100vh; display: flex; flex-direction: column; overflow: hidden; box-sizing: border-box;">
            
            // --- Dev Mode Global Indicator Badge ---
            {move || {
                if dev_mode.0.get() {
                    view! {
                        <div style="position: fixed; top: 8px; right: 8px; z-index: 9999; background: #ef4444; color: white; font-size: 10px; font-weight: 800; padding: 3px 8px; border-radius: 100px; letter-spacing: 1px; pointer-events: none;">
                            "⚙ DEV MODE"
                        </div>
                    }.into_any()
                } else {
                    view! { <div style="display:none;"></div> }.into_any()
                }
            }}
            
            // Content Area (scrollable)
            <div style="flex: 1; overflow-y: auto;">
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/settings") view=Settings />
                    <Route path=path!("/report") view=Report />
                    <Route path=path!("/report_print") view=ReportPrint />
                    <Route path=path!("/dev") view=Dev />
                </Routes>
            </div>

            // Bottom Navigation Bar
            {
                let loc = leptos_router::hooks::use_location();
                move || {
                    if loc.pathname.get() != "/report_print" {
                        view! {
                            <nav style="display: flex; justify-content: space-around; background: #ffffff; border-top: 1px solid #edf2f7; padding: 10px 0 14px 0;" id="app-nav-bar">
                                <a href="/" style="display: flex; flex-direction: column; align-items: center; gap: 4px; text-decoration: none; color: #1a3a5c;">
                                    <i class="fas fa-home" style="font-size: 20px;"></i>
                                    <span style="font-size: 12px; font-weight: 600;">"Beranda"</span>
                                </a>
                                <a href="/report" style="display: flex; flex-direction: column; align-items: center; gap: 4px; text-decoration: none; color: #1a3a5c;">
                                    <i class="fas fa-file-alt" style="font-size: 20px;"></i>
                                    <span style="font-size: 12px; font-weight: 600;">"Laporan"</span>
                                </a>
                                <a href="/settings" style="display: flex; flex-direction: column; align-items: center; gap: 4px; text-decoration: none; color: #1a3a5c;">
                                    <i class="fas fa-cog" style="font-size: 20px;"></i>
                                    <span style="font-size: 12px; font-weight: 600;">"Pengaturan"</span>
                                </a>
                                // Dev Mode nav tab — only visible when Dev Mode is active
                                {move || {
                                    if dev_mode.0.get() {
                                        view! {
                                            <a href="/dev" style="display: flex; flex-direction: column; align-items: center; gap: 4px; text-decoration: none; color: #ef4444;">
                                                <i class="fas fa-tools" style="font-size: 20px;"></i>
                                                <span style="font-size: 12px; font-weight: 800;">"Dev"</span>
                                            </a>
                                        }.into_any()
                                    } else {
                                        view! { <span style="display:none;"></span> }.into_any()
                                    }
                                }}
                            </nav>
                        }.into_any()
                    } else {
                        view! { <div style="display: none;"></div> }.into_any()
                    }
                }
            }
        </main>
    }
}
