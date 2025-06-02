// src/dioxus_ui/docker_config_view.rs
#![allow(non_snake_case)] // Common for Dioxus components

use dioxus::prelude::*;

// Define a struct to hold the Docker configuration state
#[derive(Clone, Default, PartialEq, Debug)] // Added Debug
pub struct DockerConfigState { // Made public
    container_name: String,
    since: String,
    until: String,
    follow: bool,
    timestamps: bool,
    tail: String,
}

// Define props for the component, including a callback for when config is submitted
#[derive(Props)] // Removed PartialEq from derive
pub struct DockerConfigViewProps<'a> {
    // Callback to notify parent about the configuration
    // For now, let's assume it just takes the state.
    // Later, this might involve passing an Application/UI message or specific action.
    pub on_submit: Option<EventHandler<'a, DockerConfigState>>, // Made field public for potential external construction if needed
}

impl<'a> PartialEq for DockerConfigViewProps<'a> {
    fn eq(&self, other: &Self) -> bool {
        // EventHandlers are functions/closures, they don't have a meaningful direct equality.
        // For props comparison, we often want to know if the *identity* of the handler changed,
        // or if other data props changed. If the handler is the only prop,
        // or if we always want to re-render if the parent re-renders (supplying a new closure instance),
        // then returning true (if no other fields to compare) or comparing other fields is appropriate.
        // Dioxus's EventHandler has an internal ID that can be used for PartialEq.
        // So, we can compare them directly.
        self.on_submit == other.on_submit
        // If there were other fields:
        // self.some_other_field == other.some_other_field && self.on_submit == other.on_submit
    }
}

pub fn DockerConfigView<'a>(cx: Scope<'a, DockerConfigViewProps<'a>>) -> Element<'a> {
    // Use a local state to manage the form inputs
    let config_state = use_state(cx, DockerConfigState::default);

    // Variable to hold potential validation error messages
    let error_message = use_state(cx, || String::new());

    let submit_config = move |_| {
        // Basic validation
        if config_state.get().container_name.trim().is_empty() {
            error_message.set("Container name cannot be empty.".to_string());
            return;
        }
        error_message.set("".to_string()); // Clear error on successful validation

        // If an on_submit handler is provided, call it
        if let Some(handler) = &cx.props.on_submit {
            handler.call(config_state.get().clone());
        }
        // Potentially, clear the form or give feedback
        // For now, it just sends the data.
    };

    cx.render(rsx! {
        div {
            class: "p-4 border rounded shadow-md space-y-4",
            h2 { class: "text-xl font-semibold mb-3", "Configure Docker Log Source" }

            // Display error messages
            if !error_message.get().is_empty() {
                rsx! {
                    div {
                        class: "p-2 mb-3 text-red-700 bg-red-100 border border-red-400 rounded",
                        "{error_message}"
                    }
                }
            }

            // Container Name
            div {
                label { class: "block text-sm font-medium text-gray-700", "Container Name:" }
                input {
                    class: "mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm",
                    r#type: "text",
                    placeholder: "e.g., my_awesome_container",
                    value: "{config_state.get().container_name}",
                    oninput: move |evt| {
                        let mut current_state = config_state.get().clone();
                        current_state.container_name = evt.value.clone();
                        config_state.set(current_state);
                    }
                }
            }

            // Since
            div {
                label { class: "block text-sm font-medium text-gray-700", "Since (timestamp or relative, e.g., '10m', '2023-01-01T12:00:00Z'):" }
                input {
                    class: "mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm",
                    r#type: "text",
                    placeholder: "Leave blank for default",
                    value: "{config_state.get().since}",
                    oninput: move |evt| {
                        let mut current_state = config_state.get().clone();
                        current_state.since = evt.value.clone();
                        config_state.set(current_state);
                    }
                }
            }

            // Until
            div {
                label { class: "block text-sm font-medium text-gray-700", "Until (timestamp or relative):" }
                input {
                    class: "mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm",
                    r#type: "text",
                    placeholder: "Leave blank for default (now)",
                    value: "{config_state.get().until}",
                    oninput: move |evt| {
                        let mut current_state = config_state.get().clone();
                        current_state.until = evt.value.clone();
                        config_state.set(current_state);
                    }
                }
            }

            // Tail
            div {
                label { class: "block text-sm font-medium text-gray-700", "Tail (number of lines or 'all'):" }
                input {
                    class: "mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm",
                    r#type: "text",
                    value: "{config_state.get().tail}",
                    placeholder: "all",
                    oninput: move |evt| {
                        let mut current_state = config_state.get().clone();
                        current_state.tail = evt.value.clone();
                        if current_state.tail.is_empty() { // Ensure default if cleared
                            current_state.tail = "all".to_string();
                        }
                        config_state.set(current_state);
                    }
                }
            }

            // Follow Checkbox
            div { class: "flex items-center",
                input {
                    id: "docker_follow",
                    r#type: "checkbox",
                    class: "h-4 w-4 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500",
                    checked: "{config_state.get().follow}",
                    oninput: move |evt| {
                        let mut current_state = config_state.get().clone();
                        current_state.follow = evt.value == "true";
                        config_state.set(current_state);
                    }
                }
                label { class: "ml-2 block text-sm text-gray-900", r#for: "docker_follow", "Follow logs" }
            }

            // Timestamps Checkbox
            div { class: "flex items-center",
                input {
                    id: "docker_timestamps",
                    r#type: "checkbox",
                    class: "h-4 w-4 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500",
                    checked: "{config_state.get().timestamps}",
                    oninput: move |evt| {
                        let mut current_state = config_state.get().clone();
                        current_state.timestamps = evt.value == "true";
                        config_state.set(current_state);
                    }
                }
                label { class: "ml-2 block text-sm text-gray-900", r#for: "docker_timestamps", "Show Timestamps" }
            }

            // Submit Button
            button {
                class: "mt-4 inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                onclick: submit_config,
                "Apply Configuration"
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;
    // std::sync::Arc and Mutex are not strictly needed for the basic render test,
    // but might be if we were testing callbacks.

    #[test]
    fn test_docker_config_view_renders_basic() {
        let mut dom = VirtualDom::new_with_props(
            DockerConfigView,
            DockerConfigViewProps { on_submit: None },
        );
        dom.rebuild_in_place(); // Build the virtual DOM
        let _ = dioxus_ssr::render(&dom); // Render to string (SSR context)
                                         // No panic means it "rendered" in some sense.
                                         // We can add an assertion that the string is not empty, for example.
        assert!(!dioxus_ssr::render(&dom).is_empty(), "Rendered output should not be empty");
    }

    // As noted in the plan, more detailed tests for validation logic, state changes from input,
    // and callback invocation are complex to achieve in Dioxus pure unit tests without
    // either refactoring the component to expose its logic more directly, or using more
    // advanced testing utilities (like a headless browser or specific Dioxus testing crates
    // that might offer more interaction capabilities).
    //
    // For example, to test the submit_config closure's logic:
    // - One would ideally call it directly.
    // - This would require `config_state` and `error_message` to be passed as arguments
    //   or be accessible in a way that the test can set them up and inspect them.
    // - The `on_submit` handler could be a simple FnMut closure in the test.
    //
    // Example of what a more direct logic test might look like (if component was refactored):
    // fn test_submit_logic_empty_container_name() {
    //     let test_config_state = DockerConfigState::default(); // container_name is empty
    //     let mut error_msg = String::new();
    //     let mut submitted = false;
    //     let on_submit_callback = |_data: DockerConfigState| { submitted = true; };
    //
    //     // Hypothetical refactored submit function
    //     // handle_submission(&test_config_state, &mut error_msg, || on_submit_callback);
    //
    //     // assert_eq!(error_msg, "Container name cannot be empty.");
    //     // assert!(!submitted);
    // }
}
