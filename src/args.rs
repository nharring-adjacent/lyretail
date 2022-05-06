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

use chrono::{DateTime, Duration, Utc};
use clap::{ErrorKind, Parser};
use dateparser::parse as dateparser;
use duration_str::parse_chrono;
use tracing::instrument;

use crate::sources::SourceType;
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// The type of source to read from
    #[clap(arg_enum, long)]
    pub source_type: SourceType,
    /// File path to read from, default to stdin if source_type is file
    #[clap(long)]
    pub file: Option<PathBuf>,
    /// Whether to watch files for changes when the end is reached
    #[clap(long)]
    pub follow: bool,
    /// Cloudwatch Log Group to use
    #[cfg(feature = "aws")]
    #[clap(long)]
    pub cloudwatch_log_group: String,
    /// Cloudwatch Logstream to read from, if not supplied will attempt to read from all streams in group
    #[cfg(feature = "aws")]
    #[clap(long)]
    pub cloudwatch_log_strean: Option<String>,
    /// Timestamp to start reading from
    #[cfg(feature = "aws")]
    #[clap(parse(try_from_str = dateparser), short, long)]
    pub since: Option<DateTime<Utc>>,
    /// Timestamp to stop reading at
    #[cfg(feature = "aws")]
    #[clap(parse(try_from_str = dateparser), short, long)]
    pub until: Option<DateTime<Utc>>,
    #[cfg(feature = "aws")]
    #[clap(parse(try_from_str = parse_chrono), short, long)]
    pub window: Option<Duration>,
}

impl Args {
    /// Run complex validation on arguments
    #[instrument(level = "trace")]
    pub fn validate(&self) -> Result<(), clap::ErrorKind> {
        match self.source_type {
            SourceType::File => {},
            #[cfg(feature = "aws")]
            SourceType::Cloudwatch => {
                if self.window.is_some() && (self.since.is_some() || self.until.is_some()) {
                    return Err(ErrorKind::ArgumentConflict);
                }
            },
        }
        Ok(())
    }
}
