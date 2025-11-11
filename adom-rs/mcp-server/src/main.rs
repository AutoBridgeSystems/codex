use adom_arg0::arg0_dispatch_or_else;
use adom_common::CliConfigOverrides;
use adom_mcp_server::run_main;

fn main() -> anyhow::Result<()> {
    arg0_dispatch_or_else(|adom_linux_sandbox_exe| async move {
        run_main(adom_linux_sandbox_exe, CliConfigOverrides::default()).await?;
        Ok(())
    })
}
