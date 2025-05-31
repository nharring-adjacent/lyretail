#![allow(non_snake_case)] // Common for Dioxus components
#![allow(dead_code)]     // As per plan

use dioxus::prelude::*;
use crate::dioxus_ui::base_table::{BaseTable, LogGroupSummaryProps};
use crate::dioxus_ui::log_group_view::{LogGroupView, LogGroupDetailProps};

// Define a simple enum for the view state, similar to the TUI's UiState
// We won't implement the logic to switch states yet.
pub enum DioxusUiState {
    Base,
    LogGroupSelected(LogGroupDetailProps),
}

#[derive(Props, PartialEq, Clone)]
pub struct AppProps {
    pub log_groups: Vec<LogGroupSummaryProps>,
}

// This is the main application component for the Dioxus UI.
// It's not launched, just defined.
pub fn App(cx: Scope<AppProps>) -> Element {
    // For now, let's just show the BaseTable with sample data
    // and the LogGroupView with sample data or as empty.
    // In a real app, state would determine which view is more prominent
    // or if LogGroupView is shown at all.

    // Sample data for LogGroupView is kept for now.
    let selected_log_group_sample = LogGroupDetailProps {
        id: "group_1_detail".to_string(),
        template: "User <*> logged in from <*>".to_string(),
        occurrences: 10,
    };

    cx.render(rsx! {
        div {
            h1 { "LyreTail - Dioxus UI Shell" }
            BaseTable { log_groups: cx.props.log_groups.clone() }
            hr {}
            // For LogGroupView, we use selected_log_group prop name
            LogGroupView { selected_log_group: Some(selected_log_group_sample) }
            hr {}
            LogGroupView { selected_log_group: None } // Example of no log group selected
        }
    })
}

// A function that could theoretically launch this (but won't be called)
pub fn _launch_dioxus_desktop_app() {
    // dioxus_desktop::launch(App);
    // This line is commented out as we are not integrating or running it.
    // It's here to show how it *could* be launched with dioxus-desktop.
}

pub fn _launch_dioxus_web_app() {
    // dioxus_web::launch(App);
    // This line is commented out as we are not integrating or running it.
    // It's here to show how it *could* be launched with dioxus-web.
}
