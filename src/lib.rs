// src/lib.rs
pub mod app;
pub mod args;
pub mod dioxus_ui;
pub mod sources;


#[cfg(target_arch = "wasm32")]
mod wasm {
    use crate::dioxus_ui::app::{App, AppProps};
    // LogGroupSummaryProps is not directly used here anymore for initial data
    // use crate::dioxus_ui::base_table::LogGroupSummaryProps;
    use console_error_panic_hook;
    use wasm_bindgen::prelude::*;
    use wasm_logger;

    #[wasm_bindgen(start)]
    pub fn main_wasm() {
        // Setup logging and panic hook for Wasm
        wasm_logger::init(wasm_logger::Config::default());
        console_error_panic_hook::set_once();

        let initial_props = AppProps {
            log_groups: vec![], // Initialize with no data for Wasm
        };

        dioxus_web::launch_with_props(
            App,
            initial_props,
            dioxus_web::Config::new().hydrate(true), // Or hydrate(false) if not SSR
        );
    }
}
