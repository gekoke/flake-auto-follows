use color_eyre::eyre::Result;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

pub fn configure_tracing(verbose: bool) -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .display_location_section(false)
        .panic_section("Sorry about that! You may report the bug at https://github.com/gekoke/flake-auto-follows/issues")
        .install()?;

    let logging_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_level(true)
        .without_time();

    if verbose {
        tracing_subscriber::registry().with(logging_layer).init();
    } else {
        tracing_subscriber::registry().init();
    }

    Ok(())
}
