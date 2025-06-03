#![allow(non_snake_case)] // Common for Dioxus components
#![allow(dead_code)] // As per plan

use dioxus::prelude::*;

// Placeholder for the detailed properties of a log group
#[derive(Props, PartialEq, Clone)]
pub struct LogGroupDetailProps {
    pub id: String,
    pub template: String, // Example: "User <*> logged in from <*>"
    pub occurrences: u32,
    // We might want to display a few sample raw log lines later
    // pub sample_lines: Vec<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct LogGroupViewProps {
    pub selected_log_group: Option<LogGroupDetailProps>, // Option because a group might not be selected
}

pub fn LogGroupView(cx: Scope<LogGroupViewProps>) -> Element {
    match &cx.props.selected_log_group {
        Some(details) => cx.render(rsx! {
            div {
                class: "p-4 border rounded shadow-md",
                h2 { class: "text-xl font-semibold mb-2", "Log Group Details" }
                div { class: "mb-1", strong { "ID: " } "{details.id}" }
                div { class: "mb-1", strong { "Template: " } "{details.template}" }
                div { strong { "Occurrences: " } "{details.occurrences}" }
            }
        }),
        None => cx.render(rsx! {
            div {
                class: "p-4 border rounded shadow-md text-gray-500",
                "No log group selected"
            }
        }),
    }
}

// Example of how it might be used (will be dead code for now)
fn _example_usage(cx: Scope) -> Element {
    let sample_detail = LogGroupDetailProps {
        id: "group_1".to_string(),
        template: "User <*> logged in from <*>".to_string(),
        occurrences: 10, // Note: This example occurrence will be u32
    };

    // Explicitly create LogGroupViewProps for the example
    let props_for_selected_example = LogGroupViewProps {
        selected_log_group: Some(sample_detail.clone()), // Clone sample data
    };
    let props_for_empty_example = LogGroupViewProps {
        selected_log_group: None,
    };

    cx.render(rsx! {
        // Use spread syntax for props in the example
        LogGroupView { ..props_for_selected_example }
        br {} // Added a line break for visual separation in example output
        LogGroupView { ..props_for_empty_example }
    })
}
