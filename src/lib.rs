// src/lib.rs

// Declare the dioxus_ui module so it becomes part of the library.
#![allow(dead_code)]
pub mod dioxus_ui;

// If other top-level modules from src/ (like app, args, sources, ui)
// are also intended to be part of the library, they should be declared here too.
// For now, focusing on what dioxus_desktop.rs needs.
pub mod app; // Made public for LogStats and LyreTail access from UI/bin
pub mod args; // Args struct might be needed by app or other modules
pub mod sources; // Log sources logic
                 // pub mod ui; // Assuming TUI is separate or main binary only for now
