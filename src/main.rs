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
    ImportPleco {
        file_location: String,
    },
    ImportText {
        text: String,
        category: String,
    },
    CategoryDescribe {
        category: String,
    },
    TranslateCategory {
        category: String
    },
    ExportPleco {
        category: String,
    },
    ImportPDF {
        category: String
    }
}

#[tokio::main]
async fn main() {
    // Fine to panic here
    let mut db = db::load_db(DB_LOCATION).unwrap();
    let cli = Cli::parse();
    match &cli.command {
        Commands::Greet {} => greet(&db),
        Commands::Translate { character } => generate_translation(character).await,
        Commands::TranslateCategory { category } => generate_translation_category(category, &db).await,
        Commands::ImportPleco { file_location } => import_pleco(file_location, DB_LOCATION, &mut db).unwrap(),
        Commands::CategoryDescribe { category } => describe_category(category.to_string(), &db),
        Commands::ExportPleco { category } => export_pleco(category, &db),
        Commands::ImportText { text, category } => import_text(category, text, &mut db, DB_LOCATION).await,
        Commands::ImportPDF {category} => import_png(category, DB_LOCATION, &mut db).await
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
