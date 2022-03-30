#!/bin/bash

RUST_LOG=error cargo run -- --interval 500ms --file <(while cat /var/log/wifi.log; do :; done)
