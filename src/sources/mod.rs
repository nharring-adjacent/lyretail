// Copyright Nicholas Harring. All rights reserved.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the Server Side Public License, version 1, as published by MongoDB, Inc.
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the Server Side Public License for more details. You should have received a copy of the
// Server Side Public License along with this program.
// If not, see <http://www.mongodb.com/licensing/server-side-public-license>.

#[cfg(feature = "aws")]
pub(crate) mod aws;

pub(crate) mod file;

use async_trait::async_trait;
use clap::ArgEnum;
use tokio::sync::mpsc;

/// Supported source types
#[derive(ArgEnum, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SourceType {
    File,
    #[cfg(feature = "aws")]
    Cloudwatch,
}

#[async_trait]
pub(crate) trait LogReader {
    async fn read_logs(
        &self,
        drain_writer: mpsc::UnboundedSender<String>,
    ) -> Result<(), anyhow::Error>;
}
