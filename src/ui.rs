use crossterm::event::{self, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread::sleep;
use std::{io::stdout, sync::Arc};

use anyhow::Error;
use chrono::Duration;
use itertools::Itertools;
use parking_lot::RwLock;
use tracing::{debug, instrument, warn};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

use crossterm::{
    event::Event,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::app::LyreTail;

#[derive(Clone, Debug)]
pub(crate) struct Ui<'a> {
    state: Arc<RwLock<TableState>>,
    stopping: Arc<AtomicBool>,
    app: &'a LyreTail,
    row_count: Arc<AtomicUsize>,
}

impl<'a> Ui<'a> {
    #[instrument]
    pub fn new(app: &'a LyreTail) -> Self {
        Self {
            state: Arc::new(RwLock::new(TableState::default())),
            stopping: Arc::new(AtomicBool::new(false)),
            app,
            row_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[instrument]
    pub(crate) fn run_ui(&self) -> Result<(), Error> {
        // setup terminal
        let mut stdout = stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let ui_args = self.app.args.clone();
        let refresh = ui_args.lock().interval.num_milliseconds();
        loop {
            if self.stopping.load(Ordering::SeqCst) {
                debug!("stopping flag seen, exiting loop");
                break;
            }
            terminal.draw(|f| self.render_ui(f))?;
            if crossterm::event::poll(Duration::milliseconds(0).to_std()?)? {
                if let Event::Key(key) = event::read()? {
                    let rows = self.row_count.load(Ordering::SeqCst);
                    match key.code {
                        KeyCode::Up => {
                            // This assignment prevents a deadlock inside the if let body
                            let selected = self.state.read().selected();
                            if let Some(selected) = selected {
                                debug!(%selected, "key up processing");
                                // When navigating a list up is down and we stop at 0
                                if selected > 0 {
                                    self.state.write().select(Some(selected - 1));
                                    continue;
                                }
                            }
                            debug!("nothing selected key up");
                            self.state.write().select(Some(0));
                        }
                        KeyCode::Down => {
                            // This assignment prevents a deadlock inside the if let body
                            let selected = self.state.read().selected();
                            if let Some(selected) = selected {
                                warn!(%selected, "key down processing");
                                if selected < rows {
                                    self.state.write().select(Some(selected + 1));
                                    continue;
                                }
                            }
                            debug!("nothing selected key down");
                            self.state.write().select(Some(rows - 1))
                        }
                        KeyCode::Esc => {
                            debug!("key esc");
                            self.trigger_exit();
                            break;
                        }
                        KeyCode::Char(c) => {
                            // Ctrl-C, q and Esc all trigger exit
                            if (c == 'c' && key.modifiers.contains(KeyModifiers::CONTROL))
                                || c == 'q'
                            {
                                debug!("key ctrl-c");
                                self.trigger_exit();
                                break;
                            }
                        }
                        u => {
                            warn!(?u, "Unknown key");
                        }
                    }
                }
            }
            sleep(Duration::milliseconds(refresh).to_std()?);
        }
        {
            let _rs = debug!("restoring terminal");
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
        }
        debug!("run_ui complete");
        Ok(())
    }

    fn trigger_exit(&self) {
        warn!("Triggering program exit");
        self.stopping.store(true, Ordering::SeqCst);
    }

    fn render_ui<B: Backend>(&self, f: &mut Frame<B>) {
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
        f.render_stateful_widget(t, rects[0], &mut self.state.clone().write());
    }
}
