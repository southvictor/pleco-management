mod translation;
mod openai_prompts;
mod import;
mod db;
mod export;

use clap::{Parser, Subcommand};

use crate::translation::generate_translation;
use crate::translation::generate_translation_category;
use crate::import::import_pleco;
use crate::import::import_text;
use crate::import::import_png;
use crate::export::export_pleco;

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
    #[clap(subcommand)]
    Import(Import),

    #[clap(subcommand)]
    Export(Export)
}

#[derive(Subcommand)]
enum Import {
    Pleco {
        file_location: String,
    },
    PDF {
        category: String
    },
    Text {
        text: String,
        category: String,
    },
}

#[derive(Subcommand)]
enum Export {
    Pleco {
        category: String,
    },
    Text {
        category: String,
    },
    Examples {
        category: String
    },
}

#[tokio::main]
async fn main() {
    // Fine to panic here
    let mut db = db::load_db(DB_LOCATION).unwrap();
    let cli = Cli::parse();
    match &cli.command {
        Commands::Greet {} => greet(&db),
        Commands::Translate { character } => generate_translation(character).await,
        Commands::Import(import) => match import {
            Import::Pleco { file_location } => import_pleco(file_location, DB_LOCATION, &mut db).unwrap(),
            Import::PDF {category} => import_png(category, DB_LOCATION, &mut db).await,
            Import::Text { text, category } => import_text(category, text, &mut db, DB_LOCATION).await,
        },
        Commands::Export(export) => match export {
            Export::Pleco{category}=>export_pleco(category, &db),
            Export::Text{category}=>describe_category(category.to_string(), &db),
            Export::Examples { category } => generate_translation_category(category, &db).await,
        }
    }
}

fn greet(db: &db::DB) {
    let category_count = db::get_category_cards(db);
    category_count.iter().for_each(|entry| {
        println!("Category: {}, Count: {}", entry.0, entry.1.len());
    });
}

fn describe_category(category: String, db: &db::DB) {
    let category_count = db::get_category_cards(db);
    if !category_count.contains_key(&category.to_lowercase()) {
        println!("Category not found");
    } else {
        println!("{}", category_count.get(&category.to_lowercase()).unwrap().len());
        category_count.get(&category.to_lowercase()).unwrap().iter().enumerate().for_each(|(i, card   )| {
            print!("{} {},", card.character, card.pinyin)
        });
    }
}
