use clap::Parser;

use blockchain_sim::cli::{run, Cli};

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("Hata: {}", err);
        std::process::exit(1);
    }
}
