# Version 0.5.0

## Features
- Cloudwatch LogStream source, requires `aws` feature
- Tracing integration improvement so `RUST_LOG` can be used to control level output
- Flags to control log output to file or STDERR

## Changes and improvements
- Significant refactoring of input handling to make adding new sources easier
- Migrated to `#[tokio::main]` and adopted `async-trait` to simplify using futures for concurrency

## Bug Fixes
- Key up/down logic was broken so only top/bottom rows could be selected, this is now fixed.

## Deprecations
- Update interval made no sense after making everything async