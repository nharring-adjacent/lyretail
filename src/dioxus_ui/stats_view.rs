// src/dioxus_ui/stats_view.rs
#![allow(non_snake_case)]

use dioxus::prelude::*;
use std::sync::Arc;
use parking_lot::RwLock; // Ensure this is in Cargo.toml if not already by another dep
use crate::app::LogStats; // Assuming LogStats is pub from app.rs
use tokio::time::Duration; // For interval

#[derive(Props, Clone)] // PartialEq will be implemented manually
pub struct StatsViewProps {
    pub stats_ref: Arc<RwLock<LogStats>>,
}

impl PartialEq for StatsViewProps {
    fn eq(&self, other: &Self) -> bool {
        // Compare stats_ref by pointer equality
        Arc::ptr_eq(&self.stats_ref, &other.stats_ref)
        // If there were other fields, they would be compared here too, e.g.:
        // && self.other_field == other.other_field
    }
}

pub fn StatsView(cx: Scope<StatsViewProps>) -> Element {
    // Use a state to trigger re-renders periodically
    let tick = use_state(cx, || 0);

    // This effect will run when the component is mounted and then on an interval
    use_effect(cx, (), {
        let tick = tick.clone();
        move |_| {
            let mut interval = tokio::time::interval(Duration::from_secs(1)); // Update UI every second
            async move {
                loop {
                    interval.tick().await;
                    tick.set(*tick.get() + 1);
                }
            }
        }
    });

    // Read stats. Need to handle potential poisoning of RwLock, though less common.
    let (lines_processed, lps) = {
        let stats_guard = cx.props.stats_ref.read();
        (stats_guard.total_lines_processed, stats_guard.lines_per_second)
    };

    cx.render(rsx! {
        div {
            class: "p-2 my-2 border rounded shadow-sm bg-gray-50",
            h3 { class: "text-md font-semibold mb-1 text-gray-700", "Processing Statistics" }
            p { class: "text-sm text-gray-600", "Total Lines Processed: {lines_processed}" }
            p { class: "text-sm text-gray-600", "Lines Per Second: {lps:.2}" } // Format to 2 decimal places
        }
    })
}
