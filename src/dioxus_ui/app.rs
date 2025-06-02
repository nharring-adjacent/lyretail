#![allow(non_snake_case)] // Common for Dioxus components
#![allow(dead_code)]     // As per plan

use dioxus::prelude::*;
use crate::dioxus_ui::base_table::{BaseTable, LogGroupSummaryProps};
// Import LogGroupViewProps alongside LogGroupView and LogGroupDetailProps
use crate::dioxus_ui::log_group_view::{LogGroupView, LogGroupDetailProps, LogGroupViewProps};

// Import DockerConfigView and its props
use crate::dioxus_ui::docker_config_view::{DockerConfigView, DockerConfigState};
// Import StatsView and its props
use crate::dioxus_ui::stats_view::StatsView; // StatsViewProps not directly used in App's render call signature, but good for context
use crate::app::LogStats; // For AppProps
use std::sync::Arc; // For AppProps
use parking_lot::RwLock; // For AppProps
use tracing::info; // For logging

// Define UI states
#[derive(Clone, PartialEq, Debug)]
pub enum CurrentView {
    Dashboard, // Default view showing logs or summaries
    ConfigureDocker,
    // Potentially other views like ConfigureFile, ConfigureCloudwatch etc.
}

#[derive(Props, Clone)] // Removed PartialEq here, will implement manually
pub struct AppProps {
    pub log_groups: Vec<LogGroupSummaryProps>,
    pub stats_ref: Arc<RwLock<LogStats>>, // Added stats_ref
}

impl PartialEq for AppProps {
    fn eq(&self, other: &Self) -> bool {
        self.log_groups == other.log_groups && Arc::ptr_eq(&self.stats_ref, &other.stats_ref)
    }
}

// This is the main application component for the Dioxus UI.
pub fn App(cx: Scope<AppProps>) -> Element {
    // State for the current view
    let current_view = use_state(cx, || CurrentView::Dashboard);

    // Placeholder for Docker configuration received from the form
    let _docker_config = use_state(cx, || Option::<DockerConfigState>::None); // Changed to _docker_config as it's not read yet

    // Sample data for LogGroupView (remains for now)
    let selected_log_group_sample = LogGroupDetailProps {
        id: "group_1_detail".to_string(),
        template: "User <*> logged in from <*>".to_string(),
        occurrences: 10,
    };
    let props_for_selected_view = LogGroupViewProps {
        selected_log_group: Some(selected_log_group_sample.clone())
    };
    let props_for_empty_view = LogGroupViewProps {
        selected_log_group: None
    };

    // Callback for DockerConfigView submission
    let handle_docker_config_submit = {
        let current_view = current_view.clone();
        let docker_config_state = _docker_config.clone(); // Renamed for clarity
        move |config: DockerConfigState| {
            info!("Docker configuration submitted: {:?}", config); // Use tracing::info!
            docker_config_state.set(Some(config));
            // Here, you would typically trigger the backend to restart with new settings
            // or update a shared application state.
            current_view.set(CurrentView::Dashboard); // Switch back to dashboard
        }
    };

    cx.render(rsx! {
        div {
            class: "container mx-auto p-4", // Basic container styling
            h1 { class: "text-2xl font-bold mb-4", "LyreTail - Dioxus UI" }

            // Render StatsView here, e.g., right below the title
            StatsView { stats_ref: cx.props.stats_ref.clone() } // Pass the stats_ref

            // Navigation (simple buttons for now)
            nav {
                class: "mb-4 flex space-x-2",
                button {
                    class: "px-3 py-1 border rounded hover:bg-gray-100",
                    onclick: move |_| current_view.set(CurrentView::Dashboard),
                    "Dashboard"
                }
                button {
                    class: "px-3 py-1 border rounded hover:bg-gray-100",
                    onclick: move |_| current_view.set(CurrentView::ConfigureDocker),
                    "Configure Docker Logs"
                }
                // Add other source config buttons here later
            }

            // Conditional content based on current_view
            match current_view.get() {
                CurrentView::Dashboard => rsx! {
                    // Existing layout (or a refined dashboard)
                    BaseTable { log_groups: cx.props.log_groups.clone() }
                    hr {}
                    LogGroupView { ..props_for_selected_view } // Example
                    hr {}
                    LogGroupView { ..props_for_empty_view }    // Example
                    // Later, this dashboard should show actual logs or stats
                },
                CurrentView::ConfigureDocker => rsx! {
                    DockerConfigView {
                        on_submit: handle_docker_config_submit
                    }
                },
                // Handle other views here
            }
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
