mod translation;
mod openai_prompts;

use clap::{Parser, Subcommand};

use crate::translation::generate_translation;

#[derive(Parser)]
#[command(name = "chinese-pratice-tool")]
#[command(about = "A tool to help you practice Chinese", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Greet {},
    Translate {
        character: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Greet {} => greet(),
        Commands::Translate { character } => generate_translation(character).await,
    }
}

fn greet() {
    println!("你好, 我可以帮你学中文!");
}
