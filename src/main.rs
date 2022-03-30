extern crate tracing;
mod app;
mod args;
mod ui;

use std::sync::Arc;

use app::LyreTail;
use clap::Parser;
use drain_flow::SimpleDrain;
use parking_lot::{Mutex, RwLock};
use tracing::debug;

use ui::Ui;

use crate::args::Args;

#[tokio::main]
async fn main() {
    // console_subscriber::init();
    tracing_subscriber::fmt::init();

    let args = Arc::new(Mutex::new(Args::parse()));
    debug!("got args");
    let drain = Arc::new(RwLock::new(SimpleDrain::new(vec![]).unwrap()));
    debug!("got drain");
    let app = LyreTail::create_app(Some(drain), args).unwrap();
    debug!("got app");

    app.run();
    debug!("app running");
    let ui = Ui::new(&app);
    debug!("got ui");
    ui.run_ui().unwrap();
}
