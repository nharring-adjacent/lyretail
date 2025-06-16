#![allow(non_snake_case)]

use crate::dioxus_ui::docker_config_view::{DockerConfigState, DockerConfigView};
use crate::dioxus_ui::docker_container_selection_view::DockerContainerSelectionView;
use dioxus::prelude::*;

#[derive(Props)]
pub struct DockerWizardProps<'a> {
    #[props(optional)]
    pub on_submit: Option<EventHandler<'a, DockerConfigState>>,
}

#[derive(Clone, Copy, PartialEq)]
enum WizardStep {
    Select,
    Configure,
}

pub fn DockerConfigWizard<'a>(cx: Scope<'a, DockerWizardProps<'a>>) -> Element<'a> {
    let step = use_state(cx, || WizardStep::Select);
    let selected = use_state(cx, || Option::<String>::None);

    let handle_select = {
        let step = step.clone();
        let selected = selected.clone();
        move |name: String| {
            selected.set(Some(name));
            step.set(WizardStep::Configure);
        }
    };

    let handle_submit = {
        let on_submit = cx.props.on_submit.clone();
        move |config: DockerConfigState| {
            if let Some(handler) = &on_submit {
                handler.call(config);
            }
        }
    };

    match *step.get() {
        WizardStep::Select => cx.render(rsx! {
            DockerContainerSelectionView { on_select_container: handle_select }
        }),
        WizardStep::Configure => {
            let name = selected.get().clone();
            cx.render(rsx! {
                DockerConfigView { on_submit: handle_submit, initial_container_name: name }
            })
        }
    }
}
