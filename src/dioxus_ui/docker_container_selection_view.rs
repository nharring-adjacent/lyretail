#![allow(non_snake_case)] // Dioxus components use PascalCase

use dioxus::prelude::*;
use bollard::models::ContainerSummary as BollardContainerSummary;
use crate::sources::docker::list_running_containers;

// Struct to hold displayable container info
#[derive(Clone, Debug, PartialEq)]
struct SelectedContainer {
    id: String,
    name: String,
}

#[derive(Props)]
pub struct DockerContainerSelectionProps<'a> {
    #[props(optional)] // Make it optional to pass None in tests
    pub on_select_container: Option<EventHandler<'a, String>>,
}

pub fn DockerContainerSelectionView<'a>(cx: Scope<'a, DockerContainerSelectionProps<'a>>) -> Element<'a> {
    let containers = use_state(cx, || Vec::<SelectedContainer>::new());
    let manual_input = use_state(cx, String::new);
    let error_message = use_state(cx, || None::<String>);
    let is_loading = use_state(cx, || false);

    // Asynchronous function to fetch containers
    let fetch_containers = {
        let containers = containers.clone();
        let error_message = error_message.clone();
        let is_loading = is_loading.clone();

        move |_| { // The argument is not used, could be () or an event
            let containers_clone = containers.clone();
            let error_message_clone = error_message.clone();
            let is_loading_clone = is_loading.clone();

            cx.spawn(async move {
                is_loading_clone.set(true);
                error_message_clone.set(None); // Clear previous errors

                match list_running_containers().await {
                    Ok(summaries) => {
                        let new_containers: Vec<SelectedContainer> = summaries
                            .into_iter()
                            .map(|summary: BollardContainerSummary| {
                                let name = summary.names.unwrap_or_else(|| vec!["N/A".to_string()])
                                    .get(0)
                                    .unwrap_or(&"N/A".to_string())
                                    .trim_start_matches('/') // Docker names often start with /
                                    .to_string();
                                SelectedContainer {
                                    id: summary.id.unwrap_or_else(|| "N/A".to_string()),
                                    name,
                                }
                            })
                            .collect();
                        containers_clone.set(new_containers);
                    }
                    Err(e) => {
                        error_message_clone.set(Some(format!("Failed to fetch containers: {}", e)));
                    }
                }
                is_loading_clone.set(false);
            })
        }
    };

    // Fetch containers when the component mounts
    use_effect(cx, (), |_| {
        fetch_containers(()); // Call with a dummy value that matches the expected input type if any
        async move {}
    });

    cx.render(rsx! {
        div {
            class: "docker-container-selector",
            h2 { "Select Docker Container" }
            button {
                onclick: move |_mouse_event_data| fetch_containers(()), // Prefixed with underscore
                disabled: *is_loading.get(),
                "Refresh"
            }

            if *is_loading.get() {
                rsx! { p { "Loading containers..." } }
            } else if let Some(err_msg) = error_message.get() {
                rsx! { p { class: "p-2 mb-3 text-red-700 bg-red-100 border border-red-400 rounded", "{err_msg}" } }
            } else {
                rsx! {
                    ul {
                        class: "container-list",
                        for summary in containers.iter() {
                            li {
                                key: "{summary.id}",
                                onclick: move |_| {
                                    if let Some(handler) = &cx.props.on_select_container {
                                        handler.call(summary.id.clone());
                                    }
                                },
                                "{summary.name} ({summary.id.chars().take(12).collect::<String>()})"
                            }
                        }
                    }
                }
            }

            div {
                class: "manual-input-area",
                input {
                    r#type: "text",
                    placeholder: "Or enter Container ID/Name manually",
                    value: "{manual_input}",
                    oninput: move |event| manual_input.set(event.value.clone()),
                }
                button {
                    onclick: move |_| {
                        if !manual_input.get().is_empty() {
                            if let Some(handler) = &cx.props.on_select_container {
                                handler.call(manual_input.get().clone());
                            }
                        }
                    },
                    disabled: manual_input.get().is_empty(),
                    "Stream Logs from Manual Input"
                }
            }
        }
    })
}
