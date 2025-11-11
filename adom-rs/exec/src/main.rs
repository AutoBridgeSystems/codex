//! Entry-point for the `adom-exec` binary.
//!
//! When this CLI is invoked normally, it parses the standard `adom-exec` CLI
//! options and launches the non-interactive Adom agent. However, if it is
//! invoked with arg0 as `adom-linux-sandbox`, we instead treat the invocation
//! as a request to run the logic for the standalone `adom-linux-sandbox`
//! executable (i.e., parse any -s args and then run a *sandboxed* command under
//! Landlock + seccomp.
//!
//! This allows us to ship a completely separate set of functionality as part
//! of the `adom-exec` binary.
use clap::Parser;
use adom_arg0::arg0_dispatch_or_else;
use adom_common::CliConfigOverrides;
use adom_exec::Cli;
use adom_exec::run_main;

#[derive(Parser, Debug)]
struct TopCli {
    #[clap(flatten)]
    config_overrides: CliConfigOverrides,

    #[clap(flatten)]
    inner: Cli,
}

fn main() -> anyhow::Result<()> {
    arg0_dispatch_or_else(|adom_linux_sandbox_exe| async move {
        let top_cli = TopCli::parse();
        // Merge root-level overrides into inner CLI struct so downstream logic remains unchanged.
        let mut inner = top_cli.inner;
        inner
            .config_overrides
            .raw_overrides
            .splice(0..0, top_cli.config_overrides.raw_overrides);

        run_main(inner, adom_linux_sandbox_exe).await?;
        Ok(())
    })
}
