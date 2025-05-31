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

// This is the main application component for the Dioxus UI.
// It's not launched, just defined.
pub fn App(cx: Scope) -> Element {
    // For now, let's just show the BaseTable with sample data
    // and the LogGroupView with sample data or as empty.
    // In a real app, state would determine which view is more prominent
    // or if LogGroupView is shown at all.

    let sample_log_groups = vec![
        LogGroupSummaryProps {
            id: "group_1".to_string(),
            event_summary: "User logged in".to_string(),
            quantity_seen: 10,
        },
        LogGroupSummaryProps {
            id: "group_2".to_string(),
            event_summary: "File not found".to_string(),
            quantity_seen: 5,
        },
    ];

    let selected_log_group_sample = LogGroupDetailProps {
        id: "group_1_detail".to_string(),
        template: "User <*> logged in from <*>".to_string(),
        occurrences: 10,
    };

    cx.render(rsx! {
        div {
            h1 { "LyreTail - Dioxus UI Shell" }
            BaseTable { log_groups: sample_log_groups }
            hr {}
            LogGroupView { log_group: selected_log_group_sample } // Corrected based on previous learning
            hr {}
            LogGroupView { } // Example of no log group selected
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
