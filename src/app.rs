// Copyright Nicholas Harring. All rights reserved.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the Server Side Public License, version 1, as published by MongoDB, Inc.
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the Server Side Public License for more details. You should have received a copy of the
// Server Side Public License along with this program.
// If not, see <http://www.mongodb.com/licensing/server-side-public-license>.

use std::sync::Arc;

use anyhow::Error;
use drain_flow::SimpleDrain;
use parking_lot::{Mutex, RwLock};
use tokio::{sync::mpsc, task};
use tracing::instrument;

#[cfg(feature = "aws")]
use crate::sources::aws;
use crate::{
    args::Args,
    sources::{file::FileReader, LogReader},
};
#[derive(Clone, Debug)]
pub(crate) struct LyreTail {
    drain: Arc<RwLock<SimpleDrain>>,
    pub args: Arc<Mutex<Args>>,
}

impl LyreTail {
    #[instrument(level = "trace", skip_all)]
    pub(crate) fn create_app(
        drain: Option<Arc<RwLock<SimpleDrain>>>,
        args: Arc<Mutex<Args>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            drain: drain
                .or_else(|| {
                    Some(Arc::new(RwLock::new(
                        SimpleDrain::new(vec![]).expect("creating new drain should work"),
                    )))
                })
                .unwrap(),
            args,
        })
    }

    pub(crate) fn get_drain_ref(&self) -> Arc<RwLock<SimpleDrain>> {
        self.drain.clone()
    }

    // init_input sets up the async background tasks which read and process lines from the source
    //
    #[instrument(level = "trace", skip_all)]
    pub(crate) async fn init_input(&self) {
        let drain = self.get_drain_ref();

        let follow = self.args.lock().follow;
        let (writer, reader) = mpsc::unbounded_channel::<String>();
        let source_type = self.args.lock().source_type;
        match source_type {
            crate::sources::SourceType::File => {
                let file = self.args.lock().file.clone().unwrap();
                task::spawn(async move {
                    let reader = FileReader::new(&file, follow);
                    reader.read_logs(writer).await.unwrap();
                });
            },
            #[cfg(feature = "aws")]
            crate::sources::SourceType::Cloudwatch => {
                let args = self.args.lock();
                let log_group = args
                    .cloudwatch
                    .clone()
                    .expect("Must specify log stream when using cloudwatch sources");
                let since = args.since.clone();
                let until = args.until.clone();
                let window = args.window.clone();
                task::spawn(async move {
                    let reader =
                        aws::cloudwatch::CloudwatchReader::new(since, until, window, log_group)
                            .await;
                    reader.read_logs(writer).await.unwrap();
                });
            },
        };

        task::spawn(async move { process_lines(drain, reader).await });
    }
}

#[instrument(skip_all, level = "trace")]
async fn process_lines(
    drain: Arc<RwLock<SimpleDrain>>,
    mut drain_reader: mpsc::UnboundedReceiver<String>,
) -> Result<(), anyhow::Error> {
    loop {
        tokio::select! {
            maybe_line = drain_reader.recv() => {
                if let Some(line) = maybe_line {
                    drain.write().process_line(line)?;
                }
            }
        }
    }
}
