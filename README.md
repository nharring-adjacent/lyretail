# lyretail

Lyretail is a streaming text parser and categorizer based on the "Drain" algorithm, now featuring a user interface built with Dioxus for both desktop and web.

## What it does

Lyretail consumes a stream of input lines (from files or eventually other sources like AWS CloudWatch), tokenizes them, and then processes them according to the Drain algorithm as implemented in the crate [drain-flow](https://github.com/nharring-adjacent/drain-flow). The application displays metadata about the processed lines and the event groups they are sorted into.

## Why would I use it?

If you have a service generating a large volume of log data without a consistent format, Lyretail can help you understand the general patterns and common messages. Unlike tools that require predefined parsing rules, Lyretail attempts to automatically sift out the constant parts of log messages from the variable parts.

## Architecture

The core log processing logic is handled by the `drain-flow` crate. The user interface is built using the [Dioxus](https://dioxuslabs.com/) framework, allowing Lyretail to run as a native desktop application and as a web application in any modern browser.

## Development Setup

To build and run Lyretail, you'll need Rust installed.

### Common Dependencies

Ensure you have the necessary system dependencies for `dioxus-desktop`. On Debian/Ubuntu-based systems, these can typically be installed with:
```bash
sudo apt-get update
sudo apt-get install libgtk-3-dev libwebkit2gtk-4.1-dev
```
Other operating systems will have similar requirements for GTK and WebKit development libraries.

### Building and Running the Desktop Application

1.  **Build:**
    ```bash
    cargo build --release
    ```
2.  **Run:**
    The executable will be located at `target/release/lyretail`.
    ```bash
    ./target/release/lyretail [OPTIONS]
    ```
    You can pass command-line arguments as defined (e.g., for specifying a file source). For example:
    ```bash
    ./target/release/lyretail --source-type file --file /path/to/your/logfile.log
    ```
    If no arguments are provided, it might default to requiring a file selection via a dialog or specific default behavior. (Note: The current implementation parses arguments; behavior without them depends on `args.rs` and `main.rs` logic).


### Building and Running the Web Application

The web application is compiled to WebAssembly (Wasm). You'll need `wasm-pack`.

1.  **Install `wasm-pack`** (if you haven't already):
    ```bash
    cargo install wasm-pack
    ```

2.  **Build the Wasm package:**
    Navigate to the root of the project directory and run:
    ```bash
    wasm-pack build --target web
    ```
    This will create a `pkg` directory containing the Wasm binary and JavaScript bindings.

3.  **Serve the Web Application:**
    You'll need a simple HTTP server to serve the `index.html` and the `pkg` directory.
    If you have Python installed:
    ```bash
    python -m http.server 8080
    ```
    Or using `miniserve`:
    ```bash
    cargo install miniserve
    miniserve . --index index.html --port 8080
    ```
    Then open your browser and navigate to `http://localhost:8080`.

## What's with the name?
[Lyretail Coralfish](https://en.wikipedia.org/wiki/Sea_goldie) are members of the grouper (and sea bass!) family, and in my mind
that's what this tool does: it groups lines! There are many groupers, but this is the only one with `tail` in its name which seemed
too fitting not to use!
