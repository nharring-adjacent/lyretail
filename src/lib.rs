// src/lib.rs

// Declare the dioxus_ui module so it becomes part of the library.
pub mod dioxus_ui;

// If other top-level modules from src/ (like app, args, sources, ui)
// are also intended to be part of the library, they should be declared here too.
// For now, focusing on what dioxus_desktop.rs needs.
// pub mod app; // Example: if LyreTail struct or other app logic is needed by other binaries/tests
// pub mod args; // Example: if Args struct is needed

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    use crate::dioxus_ui::app::{App, AppProps}; // Assuming AppProps might be needed or adapted
    use crate::dioxus_ui::base_table::LogGroupSummaryProps; // Or fetch data differently for web

    #[wasm_bindgen(start)]
    pub fn main_wasm() {
        // Setup logging if needed
        // wasm_logger::init(wasm_logger::Config::default());
        // console_error_panic_hook::set_once();

        // For now, use placeholder data similar to the initial desktop version
        // This will need to be replaced with actual data loading/integration
        let initial_props = AppProps {
            log_groups: vec![
                LogGroupSummaryProps {
                    id: "web_placeholder_1".to_string(),
                    event_summary: "Placeholder event from Wasm".to_string(),
                    quantity_seen: 10,
                },
                LogGroupSummaryProps {
                    id: "web_placeholder_2".to_string(),
                    event_summary: "Another placeholder".to_string(),
                    quantity_seen: 25,
                },
            ],
        };
        dioxus_web::launch_with_props(
            App,
            initial_props,
            dioxus_web::Config::new().hydrate(true), // Or hydrate(false) if not SSR
        );
    }
}
