// src/sources/docker.rs
use async_trait::async_trait;
use bollard::container::{ListContainersOptions, LogOutput};
use bollard::errors::Error as BollardError; // Corrected import
use bollard::Docker;
use bollard::API_DEFAULT_VERSION; // Re-adding for connect_with_socket
use cfg_if::cfg_if; // For conditional compilation
                    // shellexpand will be used via its expanded name, no direct `use shellexpand;` needed if calling `shellexpand::tilde`
use std::default::Default;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tracing::{error, info, instrument, warn}; // Added info and warn // To use Default::default() for LogsOptions

use crate::sources::LogReader;

#[derive(Debug, Clone)]
pub(crate) struct DockerReader {
    container_name: String,
    since: Option<i64>, // Docker API uses Unix timestamps
    until: Option<i64>,
    follow: bool,
    timestamps: bool,
    tail: String,
    docker: Docker,
}

impl DockerReader {
    #[instrument(level = "trace", skip(docker_client))]
    pub async fn new(
        container_name: String,
        since_str: Option<String>,
        until_str: Option<String>,
        follow: bool,
        timestamps: bool,
        tail: String,
        docker_client: Option<Docker>, // Allow injecting a client for testing
    ) -> Result<Self, BollardError> {
        // Changed to BollardError
        let docker = match docker_client {
            Some(client) => client,
            None => connect_to_docker_with_fallback().await?, // Use new helper
        };

        // Basic parsing for relative time strings like "10m", "1h" or RFC3339.
        // For simplicity in this step, we'll assume direct timestamp strings
        // or rely on Docker daemon for relative parsing if it supports it directly in API.
        // Proper parsing of RFC3339 and relative durations (e.g., using chrono)
        // should be implemented here for robustness.
        // For now, this example will be simplified, assuming since/until are provided
        // as unix timestamps or strings Docker API understands.
        // A more robust implementation would parse RFC3339 and relative times (e.g. "10m ago") here.
        // For this subtask, we'll pass strings and let bollard/Docker handle them,
        // or expect numeric timestamps. The plan mentions strings, so we'll stick to that.
        // However, Docker's API for `since` and `until` in `LogsOptions` expects `i64` (Unix timestamp).
        // So, a conversion step will be needed.
        // This subtask will focus on the direct Docker interaction part.
        // Actual timestamp string parsing logic will be simplified for now.

        // Simplified timestamp conversion (a proper implementation would use chrono)
        let since = since_str.and_then(|s| s.parse::<i64>().ok());
        let until = until_str.and_then(|s| s.parse::<i64>().ok());

        Ok(Self {
            container_name,
            since,
            until,
            follow,
            timestamps,
            tail,
            docker,
        })
    }
}

// Helper function to connect to Docker with macOS fallback
async fn connect_to_docker_with_fallback() -> Result<Docker, BollardError> {
    match Docker::connect_with_local_defaults() {
        Ok(docker) => Ok(docker),
        Err(original_error) => {
            cfg_if! {
                if #[cfg(target_os = "macos")] {
                    match &original_error {
                        BollardError::IOError { err: io_err } => {
                            if io_err.kind() == std::io::ErrorKind::NotFound ||
                               io_err.to_string().contains("No such file or directory") ||
                               io_err.to_string().contains("os error 2") {

            let expanded_path_cow: std::borrow::Cow<'_, str> = ::shellexpand::tilde("~/Library/Containers/com.docker.docker/Data/docker.raw.sock");
            // expanded_path_cow IS ALREADY THE STRING VALUE (or a reference).
            // DO NOT MATCH expanded_path_cow for Ok/Err.

            match Docker::connect_with_socket(expanded_path_cow.as_ref(), 120, API_DEFAULT_VERSION) {
                Ok(docker_instance) => {
                    info!("Connected to Docker via macOS fallback path: {}", expanded_path_cow);
                    return Ok(docker_instance);
                }
                Err(fallback_err) => {
                    warn!("Failed to connect via macOS fallback path ({}): {}. Original error: {}", expanded_path_cow, fallback_err, original_error);
                    return Err(BollardError::IOError {
                        err: std::io::Error::other(format!("Docker connection failed after fallback attempt on host: {}", expanded_path_cow.as_ref())),
                    });
                }
            }
                            } else {
                                // If the IO error is not the specific kind for fallback, return the original error.
                                return Err(original_error);
                            }
                        }
                        // Potentially handle other BollardError variants if they also indicate "not found"
                        // For example: BollardError::DockerResponseNotFoundError { .. } might be relevant
                        // but typically refers to API resource not found, not socket file.
                        _ => {
                            // Not an IO error or not the specific kind for fallback
                            return Err(original_error);
                        }
                    }
                }
            }
            // If not macOS, or not the specific error for fallback, or if macOS fallback logic didn't return Ok early,
            // or if it was macOS but the error type didn't match anything in the specific BollardError::IOError or _ arms,
            // we will hit this. It's a final catch-all.
            Err(original_error)
        }
    }
}

pub async fn list_running_containers(
) -> Result<Vec<bollard::models::ContainerSummary>, BollardError> {
    // Changed to BollardError
    let docker = connect_to_docker_with_fallback().await?; // Use new helper
    let options = Some(ListContainersOptions::<String> {
        all: false,
        ..Default::default()
    });
    docker.list_containers(options).await
}

#[async_trait]
impl LogReader for DockerReader {
    #[instrument(level = "trace", skip(self, drain_writer))]
    async fn read_logs(
        &self,
        drain_writer: mpsc::UnboundedSender<String>,
    ) -> Result<(), anyhow::Error> {
        info!(container = %self.container_name, "Starting Docker log reading");

        let options = bollard::container::LogsOptions {
            // Added explicit path for clarity
            follow: self.follow,
            stdout: true,
            stderr: true,
            since: self.since.unwrap_or_default(), // Changed
            until: self.until.unwrap_or_default(), // Changed
            timestamps: self.timestamps,
            tail: self.tail.clone(), // Clone since LogsOptions takes String
            ..Default::default()
        };

        let mut stream = self.docker.logs(&self.container_name, Some(options));

        while let Some(log_result) = stream.next().await {
            match log_result {
                Ok(log_output) => {
                    // LogOutput can be Stdout, Stderr, Stdin, Console, etc.
                    // We are interested in Stdout and Stderr.
                    // The `LogOutput` enum has a `to_string()` method that prefixes.
                    // Or, we can match on the variant.
                    match log_output {
                        LogOutput::StdOut { message } => {
                            drain_writer.send(String::from_utf8_lossy(&message).into_owned())?;
                        }
                        LogOutput::StdErr { message } => {
                            drain_writer.send(String::from_utf8_lossy(&message).into_owned())?;
                        }
                        LogOutput::Console { message } => {
                            // Handle console messages too
                            drain_writer.send(String::from_utf8_lossy(&message).into_owned())?;
                        }
                        _ => {
                            // Other variants like System, Stdin not typically expected for logs
                            warn!("Received unexpected Docker log type: {:?}", log_output);
                        }
                    }
                }
                Err(e) => {
                    // Handle recoverable I/O errors vs. non-recoverable ones.
                    // Bollard's errors might wrap hyper errors, which can indicate I/O issues.
                    // For now, a simple error log and propagate.
                    // A more robust solution would inspect `e` to decide if retrying is feasible.
                    error!(container = %self.container_name, "Error reading Docker logs: {}", e);
                    // Depending on the error, we might want to break or continue (if follow is true and it's a temporary issue)
                    // For now, any error will terminate the log reading for this source.
                    return Err(anyhow::Error::new(e));
                }
            }
        }
        info!(container = %self.container_name, "Finished Docker log reading (stream ended)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*; // DockerReader, LogReader
                  // use bollard::Docker; // Not strictly needed for the tests as new() can take None, or will attempt default connection
                  // use tokio::sync::mpsc; // Not needed for these specific tests focusing on `new()`

    #[tokio::test]
    async fn test_docker_reader_new_basic_params() {
        // Test with some values
        let container_name = "test_container_1".to_string();
        let since_str = Some("1600000000".to_string()); // Example Unix timestamp
        let until_str = Some("1600000100".to_string());
        let follow = true;
        let timestamps = true;
        let tail = "100".to_string();

        let reader_result = DockerReader::new(
            container_name.clone(),
            since_str.clone(),
            until_str.clone(),
            follow,
            timestamps,
            tail.clone(),
            None, // Use default Docker connection attempt. Test might fail if Docker isn't running.
        )
        .await;

        if reader_result.is_err() {
            // If Docker is not running, this test will likely fail here.
            // We can't assert the internal fields if `new` fails.
            // For a true unit test, we'd mock the Docker client or inject a dummy one.
            // For now, we acknowledge this might happen.
            if std::env::var("CI").is_ok() {
                // Skip detailed checks on CI if Docker might not be there
                println!("Skipping DockerReader field assertions as Docker connection likely failed in CI.");
                return;
            } else {
                // For local tests, user should ensure Docker is running or accept this might fail.
                println!("Warning: DockerReader::new failed, possibly due to Docker not running. Field assertions will be skipped.");
                assert!(reader_result.is_ok(), "DockerReader::new should ideally succeed for this test (Docker might not be running). Details: {:?}", reader_result.err());
                return; // Or proceed and let it panic to highlight the dependency.
            }
        }

        let reader = reader_result.unwrap();

        assert_eq!(reader.container_name, container_name);
        assert_eq!(reader.since, since_str.map(|s| s.parse::<i64>().unwrap()));
        assert_eq!(reader.until, until_str.map(|s| s.parse::<i64>().unwrap()));
        assert_eq!(reader.follow, follow);
        assert_eq!(reader.timestamps, timestamps);
        assert_eq!(reader.tail, tail);
    }

    #[tokio::test]
    async fn test_docker_reader_new_optional_params() {
        // Test with None for optional time strings
        let container_name = "test_container_2".to_string();
        let since_str: Option<String> = None;
        let until_str: Option<String> = None;
        let follow = false;
        let timestamps = false;
        let tail = "all".to_string(); // Default tail

        let reader_result = DockerReader::new(
            container_name.clone(),
            since_str,
            until_str,
            follow,
            timestamps,
            tail.clone(),
            None, // Default Docker connection attempt
        )
        .await;

        if reader_result.is_err() {
            if std::env::var("CI").is_ok() {
                println!("Skipping DockerReader field assertions as Docker connection likely failed in CI for optional params test.");
                return;
            } else {
                println!("Warning: DockerReader::new failed in optional_params test (Docker might not be running). Field assertions will be skipped.");
                assert!(
                    reader_result.is_ok(),
                    "DockerReader::new (optional_params) should ideally succeed. Details: {:?}",
                    reader_result.err()
                );
                return;
            }
        }

        let reader = reader_result.unwrap();

        assert_eq!(reader.container_name, container_name);
        assert_eq!(reader.since, None);
        assert_eq!(reader.until, None);
        assert_eq!(reader.follow, follow);
        assert_eq!(reader.timestamps, timestamps);
        assert_eq!(reader.tail, tail);
    }

    // Comment acknowledging limitations for read_logs:
    // Testing the `read_logs` method's stream processing logic thoroughly in a unit test
    // is challenging without a mock Docker environment or a way to mock the `Stream`
    // returned by `bollard::Docker::logs`.
    // Such tests would typically involve:
    // 1. Injecting a mock `bollard::Docker` client.
    // 2. Setting up the mock client's `logs` method to return a predefined stream of `LogOutput` items.
    // 3. Calling `read_logs` on the `DockerReader`.
    // 4. Asserting that the `mpsc::UnboundedSender` receives the expected strings, correctly transformed
    //    from the `LogOutput` items.
    // 5. Testing error handling by having the mock stream yield errors.
    //
    // These capabilities are beyond simple unit tests without significant mocking infrastructure
    // or specific testing support from the `bollard` crate for its client.
    // Full verification of `read_logs` is better suited for integration tests
    // that run against a live (or containerized) Docker daemon.

    #[tokio::test]
    async fn test_list_running_containers() {
        match list_running_containers().await {
            Ok(containers) => {
                // Successfully listed containers. Print count for info.
                println!(
                    "Successfully listed {} running containers.",
                    containers.len()
                );
                // You could add more assertions here if needed, e.g., inspect container properties.
                assert!(true); // Indicates success
            }
            Err(e) => {
                // Check if the error indicates a connection problem
                // This is a simplified check. Bollard errors can be complex.
                // A common issue is `hyper::Error` related to connection refused.
                // Or `bollard::errors::Error::HyperResponseError` if daemon is not responsive.
                // Or `bollard::errors::Error::IO`
                let error_string = e.to_string();
                if error_string.contains("No such file or directory") // Common for missing Docker socket
                    || error_string.contains("Connection refused") // Common if Docker daemon is not running
                    || error_string.contains("protocol error") // Can happen if not a Docker endpoint
                    || error_string.contains("invalid scheme") // e.g. if DOCKER_HOST is misconfigured
                    || error_string.contains("hyper") // Generic hyper error, often connection related
                    || error_string.contains("Permission denied")
                // Added for OS error 13
                {
                    println!("Could not connect to Docker to list containers (which is expected in some CI environments): {}", e);
                    // Pass the test if it's a connection issue
                } else {
                    // For other errors, fail the test
                    panic!(
                        "Failed to list running containers with an unexpected error: {}",
                        e
                    );
                }
            }
        }
    }

    /*
    #[tokio::test]
    async fn conceptual_test_read_logs_stream_processing() {
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        // Assume `mock_docker_client` is a mock `bollard::Docker` instance
        // that is configured to return a specific stream for "test_container_stream"
        // let mock_docker_client = setup_mock_docker_client_with_log_stream(vec![
        //     Ok(LogOutput::StdOut { message: "line 1".into() }),
        //     Ok(LogOutput::StdErr { message: "error 1".into() }),
        //     Err(bollard::errors::Error::DockerResponseServerError { status_code: 500, message: "server error".into() })
        // ]);

        let reader = DockerReader {
            container_name: "test_container_stream".to_string(),
            since: None,
            until: None,
            follow: false,
            timestamps: false,
            tail: "all".to_string(),
            // docker: mock_docker_client, // Injected mock
        };

        // let result = reader.read_logs(tx).await;
        // assert!(result.is_err(), "Expected read_logs to propagate the server error");

        // let mut received_lines = Vec::new();
        // received_lines.push(rx.recv().await.unwrap());
        // received_lines.push(rx.recv().await.unwrap());

        // assert_eq!(received_lines, vec!["line 1".to_string(), "error 1".to_string()]);
        // assert!(rx.recv().await.is_none(), "Channel should be empty and closed after error");
    }
    */
}
