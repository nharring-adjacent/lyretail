# lyretail
Lyretail is a streaming text parser and categorizer based on the "Drain" algorithm.

Right now this is barely past the proof of concept stage, it was initially written entirely to provide a test-harness for using the `drain-flow` crate which does the actual algorithm implementation and took this form since I have occasionally wished for this tool when not working in places which have their own version. 

## What it does

LyreTail consumes a stream of input lines, tokenizes them, and then processes them according to the Drain algorithm (as implemented in the `drain-flow` crate). This allows it to identify patterns and templates in log data without prior knowledge of the log structure. It periodically outputs metadata about the lines processed and the "buckets" (log templates) they have been sorted into.

## Features

LyreTail supports various log sources and provides options for continuous monitoring:

### File and Stdin Sources
*   **File Input**: Process logs from a specified file using the `--file <PATH>` option.
*   **Stdin Input**: Read logs directly from standard input (default behavior if no file is specified).
*   **Follow Mode**: Continuously monitor files for new log entries using the `--follow` flag (similar to `tail -f`). This is also applicable to Docker logs.
*   **Periodic Output**: When reading from stdin or in follow mode, metadata summaries can be printed periodically (controlled by `--periodic` and `--interval`). For static files, a summary is printed at the end.

### Docker Log Source (New!)

LyreTail can now stream logs directly from your Docker containers. This is useful for developers who want to monitor containerized applications in real-time.

*   **Connect to Local Docker**: Automatically connects to the local Docker daemon.
*   **Specify Container**: Target a specific container by its name or ID.
*   **Time-based Filtering**: Fetch logs since a particular time (`--docker-since`) or until a time (`--docker-until`). Supports RFC3339 timestamps (e.g., `2023-10-27T10:00:00Z`) and relative durations (e.g., `10m`, `1h`).
*   **Timestamp Display**: Option to show timestamps provided by Docker for each log entry (`--docker-timestamps`).
*   **Tail Logs**: Fetch a specific number of recent log lines (`--docker-tail <NUMBER_OR_ALL>`).
*   **Live Streaming**: Use the general `--follow` flag to continuously stream new logs from the container.

## Why would I use it?

Imagine a busy service generating a large volume of log data. Understanding the primary activities or identifying common errors can be challenging, especially if the logs lack a consistent format. Traditional tools often require operators to define parsing rules or patterns upfront.

LyreTail differs by requiring no initial configuration of log formats. It dynamically sifts through the log stream, separating constant parts of messages from variable data, and groups similar log entries together. This helps in quickly understanding the main event types in your logs.

## Command-Line Arguments

Here are some of the key command-line arguments:

*   `--source-type <TYPE>`: Specify the type of log source.
    *   `file`: Read from a local file (requires `--file` argument).
    *   `docker`: Read logs from a Docker container (requires `--docker-container-name`).
    *   (If not specified, defaults to stdin or file if `--file` is present).
*   `--file <PATH>`: Path to the log file to read from (when `source-type` is `file` or implicitly).
*   `--follow`: Enable follow mode to continuously watch for new log entries. Applicable to file and Docker sources.
*   `--cloudwatch-log-group <GROUP_NAME>`: (AWS CloudWatch specific) Name of the CloudWatch log group.
    *   *(Other CloudWatch arguments like `--cloudwatch-log-stream`, `--since`, `--until`, `--window` are also available but omitted here for brevity in this update.)*

**Docker Source Specific Arguments:**
*   `--docker-container-name <NAME_OR_ID>`: Specify the Docker container name or ID. (Required for `--source-type docker`)
*   `--docker-since <TIMESTAMP_OR_DURATION>`: Show logs since a specific time (e.g., `2023-10-27T10:00:00Z` or relative `10m`).
*   `--docker-until <TIMESTAMP_OR_DURATION>`: Show logs until a specific time.
*   `--docker-timestamps`: Display timestamps attached to log entries by Docker.
*   `--docker-tail <NUMBER_OR_ALL>`: Number of lines to show from the end of the logs (e.g., `100`, `all`). Defaults to `all`.

## UI Capabilities (Experimental)

LyreTail is developing a new desktop UI using the Dioxus framework. While still experimental and not fully integrated, it aims to provide:

*   A visual way to configure log sources, including a dedicated "Configure Docker Logs" view where container name, time filters, tailing options, follow mode, and timestamp display can be set.
*   Display of log templates and their occurrences.
*   Real-time statistics on log processing, such as total lines processed and current lines per second.

## Example usage

This recording was generated using the `demo.sh` script (available in the repository) with no arguments on a Macbook Air, which processes `/var/log/wifi.log`.
[![asciicast](https://asciinema.org/a/481881.png)](https://asciinema.org/a/481881?autoplay=1&preload=1)

To stream logs from a Docker container named `my_app_container`, you might run:
```bash
lyretail --source-type docker --docker-container-name my_app_container --follow --docker-timestamps
```

## What's with the name?
[Lyretail Coralfish](https://en.wikipedia.org/wiki/Sea_goldie) are members of the grouper (and sea bass!) family. In my mind, that's what this tool does: it groups lines! There are many groupers, but this is the only one with `tail` in its name, which seemed too fitting not to use!

## Development Setup

This project is currently undergoing a migration of its UI from a terminal-based interface (TUI) to a web-based/desktop UI using Dioxus.

To work on the Dioxus UI components, you'll need the Dioxus CLI:

1.  **Install `cargo-binstall`**:
    ```bash
    cargo install cargo-binstall
    ```
    This utility helps install Rust binary crates.

2.  **Install `dioxus-cli`**:
    ```bash
    cargo binstall dioxus-cli --locked
    ```
    The Dioxus CLI provides tools for building and serving Dioxus applications (e.g., the `dx` command).

You may also need to install system dependencies for `gtk` and `webkit` if you intend to build or check `dioxus-desktop` components. On Debian/Ubuntu-based systems, these can typically be installed with:
```bash
sudo apt-get update
sudo apt-get install libgtk-3-dev libwebkit2gtk-4.1-dev
```

## UI Migration Disclaimer

The user interface of LyreTail is currently being migrated. New UI components are being developed using the Dioxus framework. These components are not yet fully integrated into the application and are considered experimental. The existing TUI remains the primary interface for now.
