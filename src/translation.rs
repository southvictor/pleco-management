use crate::openai_prompts;


pub async fn generate_translation(character: &str) {
    println!("{}", character);
    let response = openai_prompts::generate_openai_prompt(character, "translation", None).await;
    match response {
        Ok(response) => println!("{}", response),
        Err(e) => eprintln!("Error: {}", e),
    }
}

