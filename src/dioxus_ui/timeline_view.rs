#![allow(non_snake_case)]

use crate::app::LogStats;
use dioxus::prelude::*;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::time::Duration;

#[derive(Props, Clone)]
pub struct TimelineViewProps {
    pub stats_ref: Arc<RwLock<LogStats>>,
}

impl PartialEq for TimelineViewProps {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.stats_ref, &other.stats_ref)
    }
}

pub fn TimelineView(cx: Scope<TimelineViewProps>) -> Element {
    let history = use_state(cx, Vec::<f64>::new);

    use_effect(cx, (), {
        let history = history.clone();
        let stats_ref = cx.props.stats_ref.clone();
        move |_| {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            async move {
                loop {
                    interval.tick().await;
                    let val = stats_ref.read().lines_per_second;
                    let mut data = (*history.get()).clone();
                    data.push(val);
                    if data.len() > 60 {
                        data.remove(0);
                    }
                    history.set(data);
                }
            }
        }
    });

    cx.render(rsx! {
        div {
            class: "flex items-end h-20 space-x-1",
            history.iter().map(|v| {
                let height = (v * 4.0).min(80.0) as i32;
                rsx!(div { class: "bg-blue-500 w-1", style: "height: {height}px" })
            })
        }
    })
}
