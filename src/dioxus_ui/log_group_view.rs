#![allow(non_snake_case)] // Common for Dioxus components
#![allow(dead_code)]     // As per plan

use dioxus::prelude::*;

// Placeholder for the detailed properties of a log group
#[derive(Props, PartialEq, Clone)]
pub struct LogGroupDetailProps {
    pub id: String,
    pub template: String, // Example: "User <*> logged in from <*>"
    pub occurrences: usize,
    // We might want to display a few sample raw log lines later
    // pub sample_lines: Vec<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct LogGroupViewProps {
    pub log_group: Option<LogGroupDetailProps>, // Option because a group might not be selected
}

pub fn LogGroupView(cx: Scope<LogGroupViewProps>) -> Element {
    if let Some(details) = &cx.props.log_group {
        cx.render(rsx! {
            div {
                h2 { "Log Group Details (Dioxus)" }
                p { "ID: {details.id}" }
                p { "Template: {details.template}" }
                p { "Occurrences: {details.occurrences}" }
                // Placeholder for future elements like sample lines or navigation buttons
            }
        })
    } else {
        cx.render(rsx! {
            div {
                h2 { "Log Group Details (Dioxus)" }
                p { "No log group selected." }
            }
        })
    }
}

// Example of how it might be used (will be dead code for now)
fn _example_usage(cx: Scope) -> Element {
    let sample_detail = LogGroupDetailProps {
        id: "group_1".to_string(),
        template: "User <*> logged in from <*>".to_string(),
        occurrences: 10,
    };
    cx.render(rsx! {
        LogGroupView { log_group: sample_detail }
        LogGroupView {} // For the None case, rely on default
    })
}
