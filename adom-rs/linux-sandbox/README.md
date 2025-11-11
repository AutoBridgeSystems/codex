# adom-linux-sandbox

This crate is responsible for producing:

- a `adom-linux-sandbox` standalone executable for Linux that is bundled with the Node.js version of the Adom CLI
- a lib crate that exposes the business logic of the executable as `run_main()` so that
  - the `adom-exec` CLI can check if its arg0 is `adom-linux-sandbox` and, if so, execute as if it were `adom-linux-sandbox`
  - this should also be true of the `adom` multitool CLI
