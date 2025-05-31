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
use drain_flow::drains::simple::SingleLayer;
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
pub struct LyreTail {
    drain: Arc<RwLock<SingleLayer>>,
    pub args: Arc<Mutex<Args>>,
}

impl LyreTail {
    #[instrument(level = "trace", skip_all)]
    pub(crate) fn create_app(
        drain: Option<Arc<RwLock<SingleLayer>>>,
        args: Arc<Mutex<Args>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            drain: drain
                .or_else(|| {
                    Some(Arc::new(RwLock::new(
                        SingleLayer::new(vec![]).expect("creating new drain should work"),
                    )))
                })
                .unwrap(),
            args,
        })
    }

    pub(crate) fn get_drain_ref(&self) -> Arc<RwLock<SingleLayer>> {
        self.drain.clone()
    }

    // init_input sets up the async background tasks which read and process lines from the source
    //
    #[instrument(level = "trace", skip_all)]
    pub(crate) async fn init_input(&self) {
        let drain = self.get_drain_ref();
        let (writer, reader) = mpsc::unbounded_channel::<String>();

        let args_guard = self.args.lock();
        let source_type = args_guard.source_type;
        let follow = args_guard.follow;
        // Keep args_guard alive if other arms need it, or clone necessary fields earlier.

        match source_type {
            crate::sources::SourceType::File => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(file_path) = args_guard.file.clone() {
                        // Native file reading logic
                        task::spawn(async move {
                            let fr = crate::sources::file::FileReader::new(&file_path, follow);
                            if let Err(e) = fr.read_logs(writer).await {
                                tracing::error!("File reader task failed: {}", e);
                            }
                        });
                    } else {
                        tracing::warn!("File source type selected, but no file path provided.");
                        // If no file path, writer is not used by this branch, drop it or let it close when
                        // fr goes out of scope if it held the writer. Here, writer is moved to task.
                        // If task is not spawned, writer is dropped.
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    tracing::info!("File source type on Wasm: Native file reading disabled. Implement web-specific file handling.");
                    // Drop writer as this path won't produce data for now
                    drop(writer);
                }
            },
            #[cfg(feature = "aws")]
            crate::sources::SourceType::Cloudwatch => {
                // This block is already conditional on the "aws" feature.
                // Further conditional compilation for wasm32 if "aws" feature could be active on wasm.
                // Assuming 'aws' feature implies non-wasm or wasm-compatible aws sdk usage.
                // If aws sdk is not wasm compatible, this also needs #[cfg(not(target_arch = "wasm32"))]
                // or the "aws" feature itself should not be enabled for wasm builds.
                // With current Cargo.toml, "aws" is not a default feature, so less likely an issue for wasm.

                // Example of being more explicit:
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let log_group = args_guard.cloudwatch_log_group.clone();
                    let log_stream = args_guard.cloudwatch_log_strean.clone(); // Original typo here
                    let since = args_guard.since.clone();
                    let until = args_guard.until.clone();
                    let window = args_guard.window.clone();
                    task::spawn(async move {
                        let cwr = crate::sources::aws::cloudwatch::CloudwatchReader::new(
                            since, until, window, log_stream, log_group,
                        )
                        .await;
                        if let Err(e) = cwr.read_logs(writer).await {
                            tracing::error!("Cloudwatch reader task failed: {}", e);
                        }
                    });
                }
                #[cfg(target_arch = "wasm32")]
                {
                    // If AWS feature is somehow enabled for Wasm, this path is taken.
                    // Ensure AWS SDK calls here are Wasm-compatible.
                    tracing::info!("CloudWatch source type on Wasm: Ensure AWS SDK usage is Wasm-compatible.");
                    // Drop writer if this path won't/can't produce data on Wasm.
                    drop(writer);
                }
            },
        }
        // Drop args_guard if it's still held and not needed by process_lines_task
        // drop(args_guard);

        task::spawn(async move { process_lines(drain, reader).await });
    }
}

#[instrument(skip_all, level = "trace")]
async fn process_lines(
    drain: Arc<RwLock<SingleLayer>>,
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
