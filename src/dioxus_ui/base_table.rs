#![allow(non_snake_case)] // Common for Dioxus components
#![allow(dead_code)] // As per plan

use dioxus::prelude::*;

// Placeholder for the summary of a log group, similar to what BaseTable might display
#[derive(Props, PartialEq, Clone)]
pub struct LogGroupSummaryProps {
    pub id: String,
    pub event_summary: String,
    pub quantity_seen: u32,
}

#[derive(Props, PartialEq, Clone)]
pub struct BaseTableProps {
    pub log_groups: Vec<LogGroupSummaryProps>,
    // We'll need a way to communicate selection back,
    // but for now, let's keep it simple.
    // pub on_select: EventHandler<String>, // Example for later
}

pub fn BaseTable(cx: Scope<BaseTableProps>) -> Element {
    cx.render(rsx! {
        table {
            class: "table-auto w-full",
            thead {
                tr {
                    th { class: "px-4 py-2", "ID" }
                    th { class: "px-4 py-2", "Event Summary" }
                    th { class: "px-4 py-2", "Quantity Seen" }
                }
            }
            tbody {
                {cx.props.log_groups.iter().map(|log_group| rsx! {
                    tr {
                        key: "{log_group.id}",
                        td { class: "border px-4 py-2", "{log_group.id}" }
                        td { class: "border px-4 py-2", "{log_group.event_summary}" }
                        td { class: "border px-4 py-2", "{log_group.quantity_seen}" }
                    }
                })}
            }
        }
    })
}

// Example of how it might be used (will be dead code for now)
fn _example_usage(cx: Scope) -> Element {
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
    cx.render(rsx! {
        BaseTable { log_groups: sample_log_groups }
    })
}
