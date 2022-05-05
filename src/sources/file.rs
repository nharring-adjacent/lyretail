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

use async_trait::async_trait;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    sync::mpsc,
};
use tracing::instrument;

use crate::sources::LogReader;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FileReader<'a> {
    file: &'a PathBuf,
    follow: bool,
}

impl<'a> FileReader<'a> {
    #[instrument(level = "trace")]
    pub(crate) fn new(file: &'a PathBuf, follow: bool) -> Self {
        Self { file, follow }
    }
}

#[async_trait]
impl LogReader for FileReader<'_> {
    #[instrument(level = "trace", skip_all)]
    async fn read_logs(
        &self,
        drain_writer: mpsc::UnboundedSender<String>,
    ) -> Result<(), anyhow::Error> {
        let mut reader = Box::pin(BufReader::new(File::open(self.file).await.unwrap()));
        let mut buffer = String::new();
        while let Ok(b) = reader.read_line(&mut buffer).await {
            if b == 0 && !self.follow {
                break;
            }
            drain_writer.send(buffer.clone())?;
            buffer.clear();
        }
        Ok(())
    }
}
