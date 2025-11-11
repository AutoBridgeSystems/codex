#!/bin/bash

# Set "chatgpt.cliExecutable": "/Users/<USERNAME>/code/adom/scripts/debug-adom.sh" in VSCode settings to always get the 
# latest adom-rs binary when debugging Adom Extension.


set -euo pipefail

ADOM_RS_DIR=$(realpath "$(dirname "$0")/../adom-rs")
(cd "$ADOM_RS_DIR" && cargo run --quiet --bin adom -- "$@")