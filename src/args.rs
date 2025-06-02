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

#[cfg(feature = "aws")]
use chrono::{DateTime, Duration, Utc};
use clap::{CommandFactory, Parser};
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

    // Docker specific arguments
    /// Name of the Docker container to fetch logs from.
    #[clap(long)]
    pub docker_container_name: Option<String>,
    /// Fetch logs since this RFC3339 timestamp or relative duration (e.g., '10m', '1h').
    #[clap(long)]
    pub docker_since: Option<String>,
    /// Fetch logs until this RFC3339 timestamp or relative duration (e.g., '10m', '1h').
    #[clap(long)]
    pub docker_until: Option<String>,
    /// Show timestamps for Docker log entries.
    #[clap(long)]
    pub docker_timestamps: bool,
    /// Number of lines to show from the end of the logs (e.g., "all", "100").
    #[clap(long, default_value = "all")]
    pub docker_tail: String,
}

impl Args {
    /// Run complex validation on arguments
    #[instrument(level = "trace")]
    pub fn validate(&self) -> Result<(), clap::ErrorKind> {
        match self.source_type {
            SourceType::File => {
                // Existing validation for File...
                if self.file.is_none() {
                    // Example: Or handle it in clap with `required_if_eq`
                    // return Err(clap::ErrorKind::MissingRequiredArgument);
                }
            }
            #[cfg(feature = "aws")]
            SourceType::Cloudwatch => {
                if self.window.is_some() && (self.since.is_some() || self.until.is_some()) {
                    return Err(clap::ErrorKind::ArgumentConflict);
                }
                // Potentially add check for cloudwatch_log_group presence
            }
            SourceType::Docker => {
                if self.docker_container_name.is_none() {
                    // This assumes docker_container_name is Option<String>.
                    // If it's String, clap's `required_if_eq` is better.
                    // For now, let's stick to the plan of manual validation.
                    return Err(clap::Error::raw(
                        clap::ErrorKind::MissingRequiredArgument,
                        "Argument --docker-container-name is required when source_type is Docker.",
                    )
                    // .with_cmd(&Args::command()) // Removed this line as with_cmd is private
                    .exit());
                }
                // Potentially validate docker_since/until formats if they are not parsed by clap directly
                // For now, assume they are strings and will be parsed by the DockerReader.
            }
        }
        Ok(())
    }
}
