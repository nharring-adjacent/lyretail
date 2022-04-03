use std::sync::Arc;

use crossterm::event::{Event, KeyCode, KeyModifiers};
use drain_flow::log_group::LogGroup;
use tracing::debug;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Layout},
    text::{Span, Spans},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

use super::UiState;

#[derive(Debug, Clone)]
pub(crate) struct LogGroupTab {
    lg: Arc<LogGroup>,
}

impl<'a> LogGroupTab {
    pub(crate) fn new(lg: Arc<LogGroup>) -> Self {
        Self { lg }
    }

    pub(crate) fn do_render<B: Backend>(&self, f: &mut Frame<B>) {
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(5)
            .split(f.size());
        let lines = vec![Spans::from(vec![Span::raw(format!(
            "Log Group: {}",
            self.lg
        ))])];
        let para = Paragraph::new(lines)
            .block(Block::default().title("Log Group"))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(para, rects[0]);
    }

    pub(crate) fn handle_events(&self, event: Event) -> UiState {
        let current = UiState::LogGroup(self.lg.clone());
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => {
                    return UiState::Base;
                }
                KeyCode::Char(c) => {
                    // Ctrl-C, q and Esc all trigger exit
                    if (c == 'c' && key.modifiers.contains(KeyModifiers::CONTROL)) || c == 'q' {
                        debug!("key ctrl-c");
                        return UiState::Exiting;
                    } else {
                        return current;
                    }
                }
                _ => {
                    return current;
                }
            }
        }
        current
    }
}
