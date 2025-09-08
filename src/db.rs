use std::collections::HashMap;
use std::fs;
use std::io::Write;
use serde::{Deserialize, Serialize};
use serde_json;
use chrono;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Card {
    pub character: String,
    pub category: Vec<String>,
    pub pinyin: String
}

#[derive(Debug)]
pub struct DBError(String);

pub type DB = HashMap<String, Card>;

impl std::fmt::Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DBError {}

impl From<serde_json::Error> for DBError {
    fn from(e: serde_json::Error) -> Self {
        DBError(format!("Serde error: {}", e))
    }
}

impl From<std::io::Error> for DBError {
    fn from(e: std::io::Error) -> Self {
        DBError(format!("IO error: {}", e))
    }
}

pub fn load_db(path: &str) -> Result<DB, DBError> {
    let contents: String = fs::read_to_string(path).unwrap_or_default();
    let mut db: HashMap<String, Card> = HashMap::new();
    for line in contents.lines() {
        let kv_option: Option<(&str, &str)> = line.split_once('=');
        if let Some((k, v)) = kv_option {
            let card: Card = serde_json::from_str(v.trim())?;
            db.insert(k.trim().to_string(), card);
        }
        
    }
    return Ok(db);
}

pub fn save_db(path: &str, contents: &DB) -> Result<(), DBError>{
    let temp_path  = format!("{}.{}", path, "temp");
    let backup_path  = format!("backups/{}", chrono::Local::now().to_rfc3339());
    let file_path = path.to_string();
    fs::File::create(&temp_path)?;
    if !(fs::exists(&file_path)?) {
        fs::File::create(&file_path)?;
    }
    if !(fs::exists("backups")?) {
        fs::create_dir("backups")?;
    }
    fs::copy(&file_path, &backup_path)?;
    let mut temp_file = fs::OpenOptions::new().write(true).create(true).append(true).open(&temp_path)?;
    for (key,value) in contents {
        temp_file.write(format!("{}={}\n", key, serde_json::to_string(value)?).as_bytes())?;
    }
    fs::copy(&temp_path, &file_path)?;
    fs::remove_file(temp_path)?;
    Ok(())
}

pub fn get_category_cards(db: &DB) -> HashMap<String, Vec<Card>> {
    let mut category_cards: HashMap<String, Vec<Card>> = HashMap::new();
    for card in db.values() {
        card.category.iter().for_each(
            |category| {
                category_cards.entry(category.to_string().to_lowercase()).and_modify(|category_cards| {
                    category_cards.push(card.clone());
                }).or_insert(vec![card.clone()]);
            }
        );
    }
    category_cards
}