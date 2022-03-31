#!/bin/bash
set -eo pipefail
if [ -z "$1" ]
 then
    FILE=/var/log/wifi.log
else
    FILE=$1
fi
RUST_LOG=error cargo run -- --interval 500ms --source <(while cat $FILE; do :; done)
