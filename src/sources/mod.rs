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
    async fn read_logs(&self, drain_writer: mpsc::UnboundedSender<String>) -> Result<(), anyhow::Error>;
}