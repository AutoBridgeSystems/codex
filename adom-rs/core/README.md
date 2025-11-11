# adom-core

This crate implements the business logic for Adom. It is designed to be used by the various Adom UIs written in Rust.

## Dependencies

Note that `adom-core` makes some assumptions about certain helper utilities being available in the environment. Currently, this support matrix is:

### macOS

Expects `/usr/bin/sandbox-exec` to be present.

### Linux

Expects the binary containing `adom-core` to run the equivalent of `adom sandbox linux` (legacy alias: `adom debug landlock`) when `arg0` is `adom-linux-sandbox`. See the `adom-arg0` crate for details.

### All Platforms

Expects the binary containing `adom-core` to simulate the virtual `apply_patch` CLI when `arg1` is `--adom-run-as-apply-patch`. See the `adom-arg0` crate for details.
