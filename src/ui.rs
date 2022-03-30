use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::{io::stdout, sync::Arc};

use anyhow::Error;
use chrono::Duration;
use itertools::Itertools;
use parking_lot::RwLock;
use tracing::{debug, instrument};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::app::LyreTail;

#[derive(Clone, Debug)]
pub(crate) struct Ui<'a> {
    state: Arc<RwLock<TableState>>,
    stopping: Arc<AtomicBool>,
    app: &'a LyreTail,
}

impl<'a> Ui<'a> {
    #[instrument]
    pub fn new(app: &'a LyreTail) -> Self {
        Self {
            state: Arc::new(RwLock::new(TableState::default())),
            stopping: Arc::new(AtomicBool::new(false)),
            app,
        }
    }

    #[instrument]
    pub(crate) fn run_ui(&self) -> Result<(), Error> {
        let h = self.stopping.clone();
        ctrlc::set_handler(move || {
            debug!("caught ctrl-c, setting stopping flag");
            h.store(true, Ordering::SeqCst);
        })
        .expect("Installing ctrl-c handler works");
        // setup terminal
        // enable_raw_mode()?;
        let mut stdout = stdout();

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
            sleep(Duration::milliseconds(refresh).to_std()?);
        }
        {
            let _rs = debug!("restoring terminal");
            // disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
        }
        debug!("run_ui complete");
        Ok(())
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
