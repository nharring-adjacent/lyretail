use crossterm::event::{self, Event};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use drain_flow::log_group::LogGroup;
use tui::backend::Backend;
use tui::Frame;

use std::io::Stdout;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::{io::stdout, sync::Arc};

use anyhow::Error;
use chrono::Duration;

use tracing::{debug, instrument, warn};
use tui::{backend::CrosstermBackend, Terminal};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::app::LyreTail;

use self::base::BaseTable;
use self::log_group::LogGroupTab;

mod base;
mod log_group;

pub(crate) struct Ui {
    app: Arc<LyreTail>,
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

impl Ui {
    // #[instrument(skip(app))]
    pub fn new(app: Arc<LyreTail>) -> Result<Self, Error> {
        // setup terminal
        let mut stdout = stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            app: app.clone(),
            state: UiState::Base,
            stopping: Arc::new(AtomicBool::new(false)),
            base: BaseTable::new(app.clone()),
            terminal,
            log_group: None,
        })
    }

    #[instrument(skip(self))]
    fn trigger_exit(&self) {
        warn!("Triggering program exit");
        self.stopping.store(true, Ordering::SeqCst);
    }
    #[instrument(skip(self))]
    pub(crate) fn run_ui(&mut self) -> Result<(), Error> {
        let ui_args = self.app.args.clone();
        let refresh = ui_args.lock().interval.num_milliseconds();
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
                }
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
                }
                UiState::Exiting => {
                    self.stopping.store(true, Ordering::SeqCst);
                    UiState::Exiting
                }
            };
            sleep(Duration::milliseconds(refresh).to_std()?);
        }
        let _rs = debug!("restoring terminal");
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
