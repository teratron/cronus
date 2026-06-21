mod cli;
mod commands;
mod output;

fn main() -> std::process::ExitCode {
    use clap::Parser;
    let args = cli::Cli::parse();
    let ctx = output::Context::new(args.format);
    if commands::dispatch(args.command, &ctx) == 0 {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}
