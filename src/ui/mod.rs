// Copyright Nicholas Harring. All rights reserved.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the Server Side Public License, version 1, as published by MongoDB, Inc.
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the Server Side Public License for more details. You should have received a copy of the
// Server Side Public License along with this program.
// If not, see <http://www.mongodb.com/licensing/server-side-public-license>.

use std::{
    io::{stdout, Stdout},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::Error;
use chrono::Duration;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use drain_flow::log_group::LogGroup;
use tracing::{debug, instrument, warn};
use tui::{
    backend::{Backend, CrosstermBackend},
    Frame,
    Terminal,
};

use self::{base::BaseTable, log_group::LogGroupTab};
use crate::app::LyreTail;

mod base;
mod log_group;

pub(crate) struct Ui {
    base: BaseTable,
    stopping: Arc<AtomicBool>,
    state: UiState,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    log_group: Option<Arc<LogGroup>>,
}

#[derive(Clone)]
pub(crate) enum UiState {
    Base,
    LogGroup(Arc<LogGroup>),
    Exiting,
}

pub(crate) trait LyreUIWidget<B: Backend> {
    fn do_render(&mut self, app: &LyreTail, f: &mut Frame<B>);
    fn handle_events(&mut self, event: Event) -> UiState;
}

impl<'a> Ui {
    #[instrument(level = "trace", skip_all)]
    pub fn new<'b>(app: Arc<LyreTail>) -> Result<Self, Error> {
        // setup terminal
        let mut stdout = stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            state: UiState::Base,
            stopping: Arc::new(AtomicBool::new(false)),
            base: BaseTable::new(app.clone()),
            terminal,
            log_group: None,
        })
    }

    #[instrument(level = "trace", skip(self))]
    fn trigger_exit(&self) {
        warn!("Triggering program exit");
        self.stopping.store(true, Ordering::SeqCst);
    }

    #[instrument(level = "trace", skip(self))]
    pub(crate) fn run_ui(&mut self) -> Result<(), Error> {
        loop {
            if self.stopping.load(Ordering::SeqCst) {
                debug!("stopping flag seen, exiting loop");
                break;
            }
            self.state = match &self.state {
                UiState::Base => {
                    self.terminal.draw(|f| self.base.do_render(f))?;
                    if crossterm::event::poll(Duration::milliseconds(10).to_std()?)? {
                        let event = event::read()?;
                        self.base.handle_events(event)
                    } else {
                        UiState::Base
                    }
                },
                UiState::LogGroup(log_group) => {
                    self.log_group = Some(log_group.clone());
                    let lg_view = LogGroupTab::new(log_group.clone());
                    self.terminal.draw(|f| lg_view.do_render(f))?;
                    if crossterm::event::poll(Duration::milliseconds(10).to_std()?)? {
                        let event = event::read()?;
                        lg_view.handle_events(event)
                    } else {
                        UiState::LogGroup(log_group.clone())
                    }
                },
                UiState::Exiting => {
                    self.stopping.store(true, Ordering::SeqCst);
                    UiState::Exiting
                },
            };
        }
        let _rs = debug!("restoring terminal");
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
