use std::{fs};
use crate::db::Card;
use crate::db::save_db;
use crate::db::DB;
use crate::db::DBError;
use crate::openai_prompts::generate_openai_prompt;
use quick_xml::Reader;
use quick_xml::encoding::EncodingError;
use quick_xml::events::{BytesStart, Event};
use std::string::FromUtf8Error;
use inquire::{Text};
use leptess::LepTess;
use std::path::Path;
use regex::Regex;
use rand::seq::SliceRandom;
use rand::rng;


#[derive(Debug)]
pub struct ImportError(String);

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ImportError {}

impl From<std::io::Error> for ImportError {
    fn from(e: std::io::Error) -> Self {
        ImportError(format!("IO error: {}", e))
    }
}

impl From<std::boxed::Box<dyn std::error::Error>> for ImportError {
    fn from(e: std::boxed::Box<dyn std::error::Error>) -> Self {
        ImportError(format!("Other error: {}", e.as_ref()))
    }
}

impl From<std::str::Utf8Error> for ImportError {
    fn from(e: std::str::Utf8Error) -> Self {
        ImportError(format!("Utf8 error: {}", e))
    }
}

impl From<FromUtf8Error> for ImportError {
    fn from(e: FromUtf8Error) -> Self {
        ImportError(format!("From Utf8 error: {}", e))
    }
}


impl From<EncodingError> for ImportError {
    fn from(e: EncodingError) -> Self {
        ImportError(format!("Encoding error: {}", e))
    }
}

impl From<DBError> for ImportError {
    fn from(e: DBError) -> Self {
        ImportError(format!("Encoding error: {}", e))
    }
}

pub fn import_pleco(import_file: &str, db_location: &str, db: &mut DB) -> Result<(), ImportError> {
    println!("importing {}", import_file);
    let xml_file = fs::read_to_string(import_file)?;
    let mut reader = Reader::from_str(&xml_file);

    let mut buf: Vec<u8> = Vec::new();
    let mut card: bool = false;
    let mut entry: bool = false;
    let mut headword: bool = false;
    let mut pron: bool = false;
    let mut character: String = "".to_string();
    let mut category: String = "".to_string();
    let mut pinyin: String = "".to_string();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name: String = String::from_utf8(e.name().as_ref().to_vec())?;
                if name == "card" {
                    card = true;
                    println!("Entered card: {:?}", card);
                } else if name == "entry" && card {
                    entry = true;
                    println!("Entered entry {:?}", entry);
                } else if name == "headword" && entry && card {
                    headword = true;
                    println!("Entered headword {:?}", headword);
                } else if name == "pron" && entry && card {
                    pron = true;
                    println!("Entered pron {:?}", pron);
                }
                println!("Start element: {:?}", name);
            }
            Ok(Event::Text(e)) => {
                if headword == true && character == "" {
                    character = String::from_utf8(e.to_vec())?;
                }
                if pron == true && pinyin == "" {
                    pinyin = String::from_utf8(e.to_vec())?;
                }
                println!("Text: {}", e.xml_content()?);
            }
            Ok(Event::End(ref e)) => {
                let name: String = String::from_utf8(e.name().as_ref().to_vec())?;
                if name == "card" {
                    let mut categories = Vec::new();
                    categories.push(category.to_string());
                    let new_card: Card = Card {
                        character: character.to_string(),
                        category: categories,
                        pinyin: pinyin.to_string(),
                    };
                    db.insert(character.clone(),new_card);
                    character.clear();
                    category.clear();
                    pinyin.clear();
                    card = false;
                    println!("Left card: {:?}", card);
                } else if name == "entry" {
                    entry = false;
                    println!("Left entry {:?}", entry);
                } else if name == "headword" {
                    headword = false;
                    println!("Left headword {:?}", headword);
                } else if name == "pron" {
                    pron = false;
                    println!("Left pron {:?}", headword);
                }
                println!("End element: {:?}", std::str::from_utf8(e.name().as_ref())?);
            }
            Ok(Event::Empty(ref e)) => {
                handle_element(e, &mut category);
            }
            Ok(Event::Eof) => break,
            Ok(other_event) => {
                // Catch-all for any other Event variants
                println!("Other event: {:?}", other_event);
            }
            Err(e) => return Err(ImportError(e.to_string()))
        }
        buf.clear();
    }

    save_db(db_location, &db)?;
    Ok(())
}

pub async fn import_text(category: &str, text: &str, db: &mut DB, db_location: &str) {
    let response= generate_openai_prompt(text, "generate-csv", None).await;
    match response {
        Ok(response_text) => {
            let text_characters: Vec<String> = extract_chinese_runs(&response_text);
            println!("llm output {:?}", text_characters);
            text_characters.into_iter().for_each(|character| {db.insert(character.to_string(),
                Card { character: character.to_string(), category: vec![category.to_string()], pinyin: "".to_string() }
            );});
            match save_db(db_location, &db) {
                Ok(_) => println!("Succesfully imported cards into category"),
                Err(e) => println!("Failed to save imported cards {:?}", e)
            }

        },
        Err(e) => println!("Failed to parse text input to import {:?}", e),
    }
}

fn handle_element(bytes_start: &BytesStart, category: &mut String) -> () {
    let name: String = String::from_utf8(bytes_start.name().as_ref().to_vec()).unwrap_or("[invalid utf8]".to_string());
    println!("Element name: {}", name);
    for attribute in bytes_start.attributes() {
        if let Ok(attribute_result) = attribute {
            let key = str::from_utf8(attribute_result.key.as_ref()).unwrap_or("invalid key");
            let value = attribute_result.unescape_value().unwrap_or_default();
            println!("Attribute key={} value={}", key, value);
            if name == "catassign" && key == "category" {
                category.clear();
                category.push_str(&value.as_ref());
            }
        }
    }

}

pub async fn import_png(_category: &str, db_location: &str, db: &mut DB) -> Result<(), ImportError> {
    let directory = select_directory()?;
    let mut ocr_pages: Vec<String> = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().map(|e| e == "png").unwrap_or(false) {
            let text = ocr_png(&path)?;
            ocr_pages.push(text);
        }
    }
    let mut parsed_pages: Vec<String> = Vec::new();
    for page in ocr_pages {
        for character in page.split('\n') {
            let cleaned = character.replace(' ', "").replace('"', "");
            parsed_pages.push(cleaned)
        }
    }
    let text = parsed_pages.join("");
    let mut results = extract_chinese_runs(&text);
    let mut rng = rng();

    // Shuffle in-place
    results.shuffle(&mut rng);

    for (index, character) in results.iter().enumerate() {
        let category_index: usize = (index / 50) + 1;
        let card: Card = Card {
            character: character.to_string(),
            category: vec![format!("{}-{}", _category, category_index)],
            pinyin: "".to_string(),
        };
        db.insert(character.to_string(), card);

    }

    save_db(db_location, &db)?;
    return Ok(())
}

fn extract_chinese_runs(input: &str) -> Vec<String> {
    let re = Regex::new(r"[\p{Han}]+").expect("Regex failed to initialize");
    re.find_iter(input)
        .map(|m| m.as_str().to_string())
        .collect()
}

fn ocr_png(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    // chi_sim = Simplified Chinese; use chi_tra for Traditional
    let mut tess = LepTess::new(None, "chi_sim")?;
    tess.set_image(path)?;
    let text = tess.get_utf8_text()?;
    Ok(text)
}

fn select_context() -> Result<String, Box<dyn std::error::Error>> {
    Ok(Text::new("Include any additional context").prompt()?)
}

fn select_directory() -> Result<String, Box<dyn std::error::Error>> {
    Ok(Text::new("Enter the directory with the png files.").prompt()?)
}