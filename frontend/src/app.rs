use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use crate::pages::{home::Home, settings::Settings, report::Report, report_print::ReportPrint};

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
                    <Route path=path!("/report") view=Report />
                    <Route path=path!("/report_print") view=ReportPrint />
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
