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
use std::time::Instant; // Added for LogStats

use anyhow::Error;
// use drain_flow::drains::simple::SingleLayer; // Already imported effectively by line below
use parking_lot::{Mutex, RwLock};
use tokio::{sync::mpsc, task}; // mpsc also used in process_lines
use tracing::{error, instrument}; // error also used in process_lines

#[cfg(feature = "aws")]
use crate::sources::aws;
use crate::{
    args::Args,
    sources::{docker::DockerReader, file::FileReader, LogReader as _}, // LogReader for FileReader etc, SingleLayer from drain_flow used below
};
use drain_flow::drains::simple::SingleLayer; // Explicit import for clarity, though LogReader might bring it.

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LogStats {
    pub total_lines_processed: u64,
    pub lines_per_second: f64,
}

#[derive(Clone, Debug)]
pub(crate) struct LyreTail {
    drain: Arc<RwLock<SingleLayer>>,
    pub args: Arc<Mutex<Args>>,
    log_stats: Arc<RwLock<LogStats>>, // Added field
}

impl LyreTail {
    #[instrument(level = "trace", skip_all)]
    pub(crate) fn create_app(
        drain_param: Option<Arc<RwLock<SingleLayer>>>, // Renamed for clarity
        args: Arc<Mutex<Args>>,
    ) -> Result<Self, Error> {
        let initialized_drain = drain_param
            .or_else(|| {
                Some(Arc::new(RwLock::new(
                    SingleLayer::new(vec![]).expect("creating new drain should work"),
                )))
            })
            .unwrap();
        Ok(Self {
            drain: initialized_drain,
            args,
            log_stats: Arc::new(RwLock::new(LogStats::default())), // Initialize new field
        })
    }

    pub(crate) fn get_drain_ref(&self) -> Arc<RwLock<SingleLayer>> {
        self.drain.clone()
    }

    pub(crate) fn get_stats_ref(&self) -> Arc<RwLock<LogStats>> {
        // Added getter
        self.log_stats.clone()
    }

    // init_input sets up the async background tasks which read and process lines from the source
    //
    #[instrument(level = "trace", skip_all)]
    pub(crate) async fn init_input(&self) {
        let _drain = self.get_drain_ref(); // Changed to _drain

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
            }
            #[cfg(feature = "aws")]
            crate::sources::SourceType::Cloudwatch => {
                let args = self.args.lock();
                let log_group = args.cloudwatch_log_group.clone();
                let log_stream = args.cloudwatch_log_strean.clone();
                let since = args.since.clone();
                let until = args.until.clone();
                let window = args.window.clone();
                task::spawn(async move {
                    let reader = aws::cloudwatch::CloudwatchReader::new(
                        since, until, window, log_stream, log_group,
                    )
                    .await;
                    reader.read_logs(writer).await.unwrap();
                });
            }
            crate::sources::SourceType::Docker => {
                let args_locked = self.args.lock();
                // Ensure docker_container_name is present, validated by Args::validate but good to handle robustly
                if let Some(container_name) = args_locked.docker_container_name.clone() {
                    let since_str = args_locked.docker_since.clone();
                    let until_str = args_locked.docker_until.clone();
                    let timestamps = args_locked.docker_timestamps;
                    let tail = args_locked.docker_tail.clone();
                    let follow_logs = args_locked.follow; // Use the general 'follow' flag

                    // Keep the writer from the mpsc channel
                    let task_writer = writer.clone();

                    task::spawn(async move {
                        match DockerReader::new(
                            container_name,
                            since_str,
                            until_str,
                            follow_logs,
                            timestamps,
                            tail,
                            None, // Use local Docker connection
                        )
                        .await
                        {
                            Ok(reader) => {
                                if let Err(e) = reader.read_logs(task_writer).await {
                                    error!("Docker log reading task failed: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("Failed to create DockerReader: {}", e);
                            }
                        }
                    });
                } else {
                    // This case should ideally be prevented by Args::validate,
                    // but good to log if it somehow occurs.
                    error!("Docker source type selected but no container name provided.");
                }
            }
        };

        // 'reader' is the mpsc::UnboundedReceiver<String> from the unbounded_channel call earlier in init_input
        let drain_for_processing = self.get_drain_ref();
        let stats_clone_for_processing = self.log_stats.clone();
        task::spawn(async move {
            if let Err(e) =
                process_lines(drain_for_processing, reader, stats_clone_for_processing).await
            {
                error!("Error in process_lines: {}", e);
            }
        });
    }
}

#[instrument(skip_all, level = "trace")]
async fn process_lines(
    drain: Arc<RwLock<SingleLayer>>,
    mut drain_reader: mpsc::UnboundedReceiver<String>,
    log_stats: Arc<RwLock<LogStats>>,
) -> Result<(), anyhow::Error> {
    let mut last_update_time = Instant::now();
    let mut lines_since_last_update: u64 = 0;
    let update_interval = std::time::Duration::from_secs(2);

    loop {
        tokio::select! {
            biased;
            maybe_line = drain_reader.recv() => {
                if let Some(line) = maybe_line {
                    drain.write().process_line(line)?;
                    {
                        let mut stats_guard = log_stats.write();
                        stats_guard.total_lines_processed += 1;
                        lines_since_last_update += 1;
                    }
                } else {
                    // Channel closed
                    if lines_since_last_update > 0 {
                         let elapsed_final = last_update_time.elapsed();
                         if elapsed_final.as_secs_f64() > 0.0 {
                            log_stats.write().lines_per_second = lines_since_last_update as f64 / elapsed_final.as_secs_f64();
                         } else if lines_since_last_update > 0 {
                            log_stats.write().lines_per_second = f64::INFINITY;
                         }
                    } else {
                        log_stats.write().lines_per_second = 0.0;
                    }
                    break;
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                // This branch ensures the loop doesn't spin tightly if there are no messages,
                // and allows the time check below to run periodically.
            }
        }

        if last_update_time.elapsed() >= update_interval {
            let mut stats_guard = log_stats.write();
            let elapsed = last_update_time.elapsed();
            if elapsed.as_secs_f64() > 0.0 {
                stats_guard.lines_per_second =
                    lines_since_last_update as f64 / elapsed.as_secs_f64();
            } else if lines_since_last_update > 0 {
                stats_guard.lines_per_second = f64::INFINITY;
            } else {
                stats_guard.lines_per_second = 0.0;
            }
            lines_since_last_update = 0;
            last_update_time = Instant::now();
        }
    }
    Ok(())
}
