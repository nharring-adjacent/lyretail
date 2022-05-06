// Copyright Nicholas Harring. All rights reserved.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the Server Side Public License, version 1, as published by MongoDB, Inc.
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the Server Side Public License for more details. You should have received a copy of the
// Server Side Public License along with this program.
// If not, see <http://www.mongodb.com/licensing/server-side-public-license>.

#![feature(associated_type_bounds)]
extern crate enum_kinds;
extern crate tracing;
mod app;
mod args;
mod sources;
mod ui;

use std::{fs::File, sync::Arc};

use app::LyreTail;
use clap::{CommandFactory, Parser};
use drain_flow::SimpleDrain;
use parking_lot::{Mutex, RwLock};
use tracing::debug;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};
use ui::Ui;

use crate::args::Args;

#[tokio::main]
async fn main() {
    let log_file = File::create("/tmp/lyretail.log").unwrap();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_span_events(FmtSpan::ENTER)
                .with_writer(Arc::new(log_file))
                .compact()
                .with_filter(filter_layer),
        )
        .with(console_subscriber::spawn())
        .init();

    let args_inner = Args::parse();
    debug!("got args");
    match args_inner.validate() {
        Ok(_) => {},
        Err(e) => {
            let mut cmd = Args::command();
            cmd.error(e, "Incompatible arguments provided").exit();
        },
    };
    debug!("validated args");
    let args = Arc::new(Mutex::new(args_inner));
    let drain = Arc::new(RwLock::new(SimpleDrain::new(vec![]).unwrap()));
    debug!("got drain");
    let app = LyreTail::create_app(Some(drain), args).unwrap();
    debug!("got app");
    let app_ref = Arc::new(app);
    app_ref.init_input().await;
    debug!("app running");
    let mut ui = Ui::new(app_ref.clone()).unwrap();
    debug!("got ui");
    ui.run_ui().unwrap();
}
