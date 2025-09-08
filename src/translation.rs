use crate::{db::DB, openai_prompts};


pub async fn generate_translation(character: &str) {
    let response = openai_prompts::generate_openai_prompt(character, "translation", None).await;
    match response {
        Ok(response) => println!("{}", response),
        Err(e) => eprintln!("Error: {}", e),
    }
}

pub async fn generate_translation_category(category: &str, db: &DB) {
    let context: &str = "13. **调 (tiáo)**: The chef adjusted the seasoning in the dish to achieve the perfect balance of flavors.";
    let response = openai_prompts::generate_openai_prompt_category(category, "translation", Some(context), db).await;
    match response {
        Ok(response) => println!("{}", response),
        Err(e) => eprintln!("Error: {}", e),
    }
}

