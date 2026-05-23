use std::process::ExitCode;

use sptf_tui::{cli, run};
use tracing::error;

#[tokio::main]
async fn main() -> ExitCode {
    let args = <cli::Cli as clap::Parser>::parse();

    match run(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            // Logging may not be initialized if the failure happened very early,
            // so write to stderr as well.
            error!(error = ?err, "fatal error");
            eprintln!("sptf: fatal error: {err:#}");
            ExitCode::FAILURE
        }
    }
}
