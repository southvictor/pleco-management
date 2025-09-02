use std::collections::HashMap;
use std::fs;
use std::io::Write;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    character: String,
    category: Vec<String>,
}

#[derive(Debug)]
pub struct DBError(String);

type DB = HashMap<String, Card>;

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
    let mut backup_file = fs::OpenOptions::new().write(true).create(true).append(true).open(format!("{}.{}", path, "backup"))?;
    for (key,value) in contents {
        backup_file.write(format!("{}={}\n", key, serde_json::to_string(value)?).as_bytes())?;
    }
    Ok(())
}