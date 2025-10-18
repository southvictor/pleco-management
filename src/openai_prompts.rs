use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use reqwest;
use crate::db::get_category_cards;
use crate::db::DB;


#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseV2 {
    output: Vec<OpenAIOutputV2>,
}

#[derive(Debug, Deserialize)]
struct OpenAIOutputV2 {
    content: Vec<OpenAIContentV2>,
}

#[derive(Debug, Deserialize)]
struct OpenAIContentV2 {
    r#type: String,
    text: Option<String>,
}


pub async fn generate_openai_prompt(
    character: &str,
    prompt_type: &str,
    context: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {     
    let base_prompt = match prompt_type {
        "translation" => format!(
            "Generate a english sentence using the translation of the character '{}'.
            The purpose of the sentence is for someone to practice translating the english sentence into colloquial chinese.
            Make the sentence at least 20 words long.",
            character
        ),
        "generate-csv" => format!(
            "Generate a comma separated list of words from the set '{}'.
            It needs to be usable as a input to code (no extra spaces, one comma between each word)
            Each newline or comma indicates a new word. A word will have between 2-4 distinct characters",
            character
        ),
        "generate-csv-png" => format!(
            "Generate a comma separated list of chinese phrases from this set of phrases. '{}'.
            It needs to be usable as a input to code (no extra spaces, one comma between each word)
            The set of phrases is somewhat inconsistently formatted, but generally in the form 607“散步sunZbu where we want to parse 607,散步. 
            Each phrase will have between 1-4 distinct characters",
            character
        ),
        _ => format!(
            "Provide comprehensive information about the Chinese character '{}' including pronunciation, meaning, usage, and cultural context.",
            character
        ),
    };
    let full_prompt = if let Some(ctx) = context {
        format!("{} Context: {}", base_prompt, ctx)
    } else {
        base_prompt
    };

    query_openai(full_prompt).await
}

pub async fn generate_openai_prompt_category(
    category: &str,
    prompt_type: &str,
    context: Option<&str>,
    db: &DB,
) -> Result<String, Box<dyn std::error::Error>> {
    let category_cards = get_category_cards(db);
    if !category_cards.contains_key(&category.to_lowercase()) {
        return Err("Category not found".into());
    }
    if let Some(cards) = category_cards.get(&category.to_lowercase()) {
        let character_prompt : String = cards.iter().map(|card| {card.character.clone()}).collect::<Vec<_>>().join(",");
        let base_prompt = match prompt_type {
            "translation" => format!(
                "Generate a english sentence for each character using the translation of the characters '{}'.
                The purpose of each sentence is for someone to practice translating the english sentence into colloquial chinese.
                Make each sentence around 12 words long. Limit to 20 sentences.",
                character_prompt
            ),
            _ => format!(
                "Provide comprehensive information about the Chinese characters '{}' including pronunciation, meaning, usage, and cultural context.",
                character_prompt
            ),
        };
        let full_prompt = if let Some(ctx) = context {
            format!("{} Context: {}", base_prompt, ctx)
        } else {
            base_prompt
        };

        query_openai(full_prompt).await
    } else {
        Err("Failed to generate translations".into())
    }
}

async fn query_openai(prompt: String)  -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY environment variable not set")?;
    let request: OpenAIRequest = OpenAIRequest {
        model: "gpt-4o-mini".to_string(),
        messages: vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant specializing in Chinese language learning. Provide clear, accurate, and educational responses about Chinese characters, their meanings, usage, and cultural context.".to_string(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ],
        max_tokens: 750,
        temperature: 0.7,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

        let status = response.status();
        let text = response.text().await?; // read the body once
        
        if !status.is_success() {
            // Non-2xx response — show raw body for debugging
            println!("Error {}: {}", status, text);
            return Err(format!("Request failed with status {}", status).into());
        }
        
        // Try to parse JSON
        let parsed: OpenAIResponse = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse JSON: {}\nRaw body: {}", e, text))?;
        
        // Extract the choice content
        if let Some(choice) = parsed.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            println!("No choices found in response.\nRaw body:\n{}", text);
            Err("No response from OpenAI".into())
        }
}