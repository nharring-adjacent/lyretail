use dioxus_desktop::Config;
use rfd::FileDialog;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc; // Added for stats_ref
use parking_lot::RwLock; // Added for stats_ref

use lyretail::dioxus_ui::app::{App, AppProps};
use lyretail::dioxus_ui::base_table::LogGroupSummaryProps;
use lyretail::app::LogStats; // Added for stats_ref

fn open_file_dialog() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("Text files", &["txt", "log"])
        .pick_file()
}

fn read_and_process_file(file_path: PathBuf) -> std::io::Result<Vec<LogGroupSummaryProps>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let line_count = reader.lines().count();

    // Placeholder processing: create one summary item based on line count
    Ok(vec![LogGroupSummaryProps {
        id: "file_summary".to_string(),
        event_summary: format!("File has {} lines", line_count),
        quantity_seen: line_count as u32, // Assuming quantity_seen is u32
    }])
}

fn main() {
    let log_groups_data = match open_file_dialog() {
        Some(path) => {
            match read_and_process_file(path) {
                Ok(data) => data,
                Err(e) => {
                    // Simple error handling: print to console and use empty data
                    eprintln!("Error reading or processing file: {}", e);
                    vec![]
                }
            }
        }
        None => {
            // No file selected, use empty data
            println!("No file selected.");
            vec![]
        }
    };

    dioxus_desktop::launch_with_props(
        App,
        AppProps {
            log_groups: log_groups_data,
            stats_ref: Arc::new(RwLock::new(LogStats::default())), // Provide default stats_ref
        },
        Config::new().with_window(dioxus_desktop::WindowBuilder::new().with_title("LyreTail Log Analyzer Desktop")) // Corrected typo
    );
}
