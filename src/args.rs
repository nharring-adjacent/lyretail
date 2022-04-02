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
