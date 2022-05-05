// Copyright Nicholas Harring. All rights reserved.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the Server Side Public License, version 1, as published by MongoDB, Inc.
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the Server Side Public License for more details. You should have received a copy of the
// Server Side Public License along with this program.
// If not, see <http://www.mongodb.com/licensing/server-side-public-license>.

use async_trait::async_trait;
use aws_sdk_cloudwatchlogs::Client;
use chrono::{DateTime, Duration, Utc};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use crate::sources::LogReader;
#[derive(Debug, Clone)]
pub(crate) struct CloudwatchReader {
    client: Client,
    log_stream: String,
    since: DateTime<Utc>,
    until: DateTime<Utc>,
}

impl CloudwatchReader {
    pub async fn new(
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
        window: Option<Duration>,
        log_stream: String,
    ) -> CloudwatchReader {
        let client_config = Box::new(aws_config::load_from_env().await);
        let (start, end) = match window {
            Some(window) => {
                let start = Utc::now()
                    .checked_sub_signed(window)
                    .expect("valid duration should not wrap");
                let end = Utc::now();
                (start, end)
            },
            None => {
                let start = if let None = since {
                    // default to 1 hour ago
                    Utc::now()
                        .checked_sub_signed(Duration::hours(1))
                        .expect("1 hour ago does not wrap")
                } else {
                    since.unwrap().to_owned()
                };
                let end = if let None = until {
                    // default to now
                    Utc::now()
                } else {
                    until.unwrap().to_owned()
                };
                (start, end)
            },
        };
        CloudwatchReader {
            client: aws_sdk_cloudwatchlogs::Client::new(&client_config),
            since: start,
            until: end,
            log_stream,
        }
    }
}

#[async_trait]
impl LogReader for CloudwatchReader {
    async fn read_logs(&self, lines: mpsc::UnboundedSender<String>) -> Result<(), anyhow::Error> {
        let mut event_fetcher = self
            .client
            .get_log_events()
            .log_stream_name(self.log_stream.clone())
            .set_start_time(Some(self.since.timestamp_millis()))
            .set_end_time(Some(self.until.timestamp_millis()))
            .start_from_head(true)
            .into_paginator()
            .send();

        while let Some(event) = event_fetcher.next().await {
            if let Ok(log_events) = event {
                for log_event in log_events.events().unwrap_or_default() {
                    lines.send(
                        log_event
                            .message()
                            .expect("log events have messages")
                            .to_string(),
                    )?;
                }
            }
        }
        Ok(())
    }
}
