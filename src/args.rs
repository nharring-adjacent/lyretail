// Copyright Nicholas Harring. All rights reserved.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the Server Side Public License, version 1, as published by MongoDB, Inc.
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the Server Side Public License for more details. You should have received a copy of the
// Server Side Public License along with this program.
// If not, see <http://www.mongodb.com/licensing/server-side-public-license>.

use std::path::PathBuf;

use chrono::Duration;
use clap::Parser;
use duration_str::parse_chrono;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Enable periodic printing of LogGroup information
    #[clap(short, long)]
    pub periodic: bool,
    /// The interval between updates
    #[clap(parse(try_from_str = parse_chrono), short, long, default_value="1s")]
    pub interval: Duration,
    /// The source of input, read from stdin if not specified
    #[clap(short, long)]
    pub source: Option<PathBuf>,
    /// Whether to watch files for changes when the end is reached
    #[clap(short, long)]
    pub follow: bool,
}
