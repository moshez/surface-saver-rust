use clap::Parser;
use surface_saver_rust::{Commands, run_command};

#[derive(Parser)]
#[command(name = "surface-saver")]
#[command(about = "A CLI app with subcommands", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = run_command(cli.command).await;

    for line in result.stdout {
        println!("{line}");
    }

    for line in result.stderr {
        eprintln!("{line}");
    }

    if result.exit_code != 0 {
        std::process::exit(result.exit_code);
    }
}
