use std::path::PathBuf;

use clap::Parser;
use duration_str::parse_chrono;
use chrono::Duration; 

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Enable periodic printing of LogGroup information
    #[clap(short, long)]
    pub periodic: bool,
    /// The interval between updates
    #[clap(parse(try_from_str = parse_chrono), short, long, default_value="10s")]
    pub interval: Duration,
    /// The source of input, read from stdin if not specified
    #[clap(short, long)]
    pub file: Option<PathBuf>
}
