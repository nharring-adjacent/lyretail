use dioxus_desktop::{Config, WindowBuilder};
use lyretail::app::LyreTail;
use lyretail::args::Args;
use lyretail::dioxus_ui::app::{App, AppProps};
use lyretail::dioxus_ui::base_table::LogGroupSummaryProps; // May need to change/adapt this
use std::sync::Arc;
use parking_lot::Mutex;
use clap::Parser; // For Args::parse()

// This will be the main entry point for the desktop application.
// It needs to initialize LyreTail, potentially parse args, and launch the Dioxus app.
fn main() {
    // Initialize tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main());
}

async fn async_main() {
    // 1. Parse Arguments (simplified for now)
    // In a real scenario, you might get file paths or AWS config from here.
    // For now, let's assume some defaults or that LyreTail/Args handle this.
    let args = Args::parse(); // This will parse command line arguments

    // 2. Initialize LyreTail
    // The drain initialization might need to be adapted based on LyreTail's API.
    // The `None` here means LyreTail will create its own default drain.
    let lyretail_app = match LyreTail::create_app(None, Arc::new(Mutex::new(args))) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Failed to create LyreTail app: {}", e);
            // Consider showing an error in the UI instead of just exiting
            return;
        }
    };

    // 3. Initialize Input Sources (e.g., start reading files/logs)
    // This is an async operation.
    let app_ref = Arc::new(lyretail_app);
    let app_clone_for_init = app_ref.clone();
    tokio::spawn(async move {
        app_clone_for_init.init_input().await;
    });

    // 4. Prepare Props for the Dioxus App
    // This is where we need to get data from LyreTail's drain.
    // The structure of LogGroupSummaryProps might need to change
    // to match what the drain actually provides.
    // For now, let's try to get an initial snapshot.
    // This part is a placeholder and needs to correctly interface with the drain.
    let initial_log_groups: Vec<LogGroupSummaryProps> = {
        let drain_guard = app_ref.get_drain_ref().read();
        // Assuming drain_guard has a method like `get_summaries()` or similar.
        // This is a placeholder and depends on the actual API of SingleLayer drain.
        // For now, let's return empty data or a placeholder.
        // drain_guard.get_all_summaries().iter().map(|s| LogGroupSummaryProps {
        //     id: s.id.clone(), // Fictional field
        //     event_summary: s.summary_string.clone(), // Fictional field
        //     quantity_seen: s.count as u32, // Fictional field
        // }).collect()
        vec![
            LogGroupSummaryProps{
                id: "initial_desktop".to_string(),
                event_summary: "Waiting for LyreTail data... (Desktop)".to_string(),
                quantity_seen: 0
            }
        ]
    };

    let app_props = AppProps {
        log_groups: initial_log_groups,
    };

    // 5. Launch the Dioxus Desktop Application
    dioxus_desktop::launch_with_props(
        App,
        app_props,
        Config::new().with_window(WindowBuilder::new().with_title("Lyretail Log Analyzer Desktop (Integrated)")),
    );

    // Note: Real-time updates from LyreTail to the Dioxus UI are not yet implemented here.
    // This would typically involve Dioxus state management (e.g., use_shared_state)
    // and a mechanism for LyreTail to signal updates.
}
