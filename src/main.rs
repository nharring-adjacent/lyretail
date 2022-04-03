#![feature(associated_type_bounds)]
#[macro_use]
extern crate enum_kinds;
extern crate tracing;
mod app;
mod args;
mod ui;

use std::{io, sync::Arc};

use app::LyreTail;
use clap::Parser;
use drain_flow::SimpleDrain;
use parking_lot::{Mutex, RwLock};
use tracing::{debug, Level};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

use ui::Ui;

use crate::args::Args;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(console_subscriber::spawn())
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_span_events(FmtSpan::NONE)
                .with_writer(io::stderr)
                .compact()
                .with_filter(LevelFilter::from_level(Level::WARN)),
        )
        .init();

    let args = Arc::new(Mutex::new(Args::parse()));
    debug!("got args");
    let drain = Arc::new(RwLock::new(SimpleDrain::new(vec![]).unwrap()));
    debug!("got drain");
    let app = LyreTail::create_app(Some(drain), args).unwrap();
    debug!("got app");
    let app_ref = Arc::new(app);
    app_ref.run();
    debug!("app running");
    let mut ui = Ui::new(app_ref.clone()).unwrap();
    debug!("got ui");
    ui.run_ui().unwrap();
}
