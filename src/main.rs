mod translation;
mod openai_prompts;
mod import;
mod db;
use clap::{Parser, Subcommand};

use crate::translation::generate_translation;
use crate::import::import_pleco;

const DB_LOCATION: &str = "./data";

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
    ImportPleco {
        file_location: String,
    }
}

#[tokio::main]
async fn main() {
    // Fine to panic here
    let db = db::load_db(DB_LOCATION).unwrap();
    let cli = Cli::parse();
    match &cli.command {
        Commands::Greet {} => greet(),
        Commands::Translate { character } => generate_translation(character).await,
        Commands::ImportPleco { file_location } => import_pleco(file_location).unwrap()
    }
}

fn greet() {
    println!("你好, 我可以帮你学中文!");
}
