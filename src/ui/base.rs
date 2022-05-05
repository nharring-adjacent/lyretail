// Copyright Nicholas Harring. All rights reserved.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the Server Side Public License, version 1, as published by MongoDB, Inc.
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the Server Side Public License for more details. You should have received a copy of the
// Server Side Public License along with this program.
// If not, see <http://www.mongodb.com/licensing/server-side-public-license>.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crossterm::event::{Event, KeyCode, KeyModifiers};
use drain_flow::log_group::LogGroup;
use itertools::Itertools;
use tracing::{debug, info, instrument, warn};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::app::LyreTail;

use super::UiState;

#[derive(Clone, Debug)]
pub(crate) struct BaseTable {
    row_count: Arc<AtomicUsize>,
    state: TableState,
    app: Arc<LyreTail>,
}

impl<'a> BaseTable {
    pub(crate) fn new(app: Arc<LyreTail>) -> Self {
        Self {
            row_count: Arc::new(AtomicUsize::new(0)),
            state: TableState::default(),
            app: app.clone(),
        }
    }

    #[instrument(skip(self, f))]
    pub(crate) fn do_render<B: Backend>(&mut self, f: &mut Frame<B>) {
        debug!("starting render_ui");
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(5)
            .split(f.size());
        let header_cells = ["ID", "Event", "Quantity Seen"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(Color::Blue);
        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);
        let rows = self
            .app
            .get_drain_ref()
            .read()
            .iter_groups()
            .iter()
            .flatten()
            .sorted_by(|a, b| Ord::cmp(&b.len(), &a.len()))
            .map(|lg| {
                let cells = vec![
                    Cell::from(lg.event().uid.serialize()),
                    Cell::from(lg.event().to_string()),
                    Cell::from(lg.len().to_string()),
                ];
                Row::new(cells).height(1).bottom_margin(1)
            })
            .collect::<Vec<Row>>();
        self.row_count.store(rows.len(), Ordering::SeqCst);
        let t = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("LogGroups"))
            .highlight_style(selected_style)
            .highlight_symbol(">> ")
            .widths(&[
                Constraint::Percentage(20),
                Constraint::Percentage(70),
                Constraint::Percentage(10),
            ]);
        debug!("finished building table");
        f.render_stateful_widget(t, rects[0], &mut self.state);
    }

    pub(crate) fn handle_events(&mut self, event: Event) -> UiState {
        let rows = self.row_count.load(Ordering::SeqCst);
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up => {
                    debug!("key up handler");
                    let selected = self.state.selected();
                    if let Some(selected) = selected {
                        info!(%selected, %rows, "change selection");
                        // When navigating a list up is down and we stop at 0
                        if selected > 0 {
                            info!("setting selection to {}", selected - 1);
                            self.state.select(Some(selected - 1));
                        }
                    } else {
                        info!("nothing selected key up");
                        self.state.select(Some(0));
                    }
                    return UiState::Base;
                }
                KeyCode::Down => {
                    debug!("key down handler");
                    let selected = self.state.selected();
                    if let Some(selected) = selected {
                        info!(%selected, %rows, "change selection");
                        if selected < rows {
                            info!("setting selection to {}", selected + 1);
                            self.state.select(Some(selected + 1));
                        }
                    } else {
                        info!("nothing selected, setting {}", rows - 1);
                        self.state.select(Some(rows - 1));
                    }

                    return UiState::Base;
                }
                KeyCode::Esc => {
                    debug!("key esc");
                    return UiState::Exiting;
                }
                KeyCode::Char(c) => {
                    // Ctrl-C, q and Esc all trigger exit
                    if (c == 'c' && key.modifiers.contains(KeyModifiers::CONTROL)) || c == 'q' {
                        debug!("key ctrl-c");
                        return UiState::Exiting;
                    } else {
                        return UiState::Base;
                    }
                }
                KeyCode::Enter => {
                    if let Some(selected) = self.state.selected() {
                        let lg = self.get_selected(selected);
                        return UiState::LogGroup(Arc::new(lg));
                    } else {
                        return UiState::Base;
                    }
                }
                u => {
                    warn!(?u, "Unknown key");
                    return UiState::Base;
                }
            }
        }
        UiState::Base
    }

    fn get_selected(&self, idx: usize) -> LogGroup {
        self.app
            .get_drain_ref()
            .read()
            .iter_groups()
            .iter()
            .flatten()
            .sorted_by(|a, b| Ord::cmp(&b.len(), &a.len()))
            .nth(idx)
            .expect("idx is based on selected, should exist")
            .clone()
            .to_owned()
    }
}
