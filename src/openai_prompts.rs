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
            "Generate ten different examples of english sentences using the translation of the character '{}'.
            The purpose of each sentence is for someone to practice translating the english sentence into colloquial chinese.
            Make each sentence around 9 words long. Don't include where the actual characeter should be in the sentence.",
            character
        ),
        "generate-csv" => format!(
            r#"You are a text transformer. Do NOT invent or reorder words.

            Input: a string that contains Chinese words with optional pinyin/Latin letters mixed in.
            Task:
            1) Split the input into tokens using any of these as separators: newline, comma, Chinese comma \"，\", the double-comma \"，，\", spaces, semicolons.
            2) For each token, delete every character that is NOT a CJK Han character (Unicode Han: \p{{Script=Han}}).
            - This removes pinyin (Latin letters, tone marks), numbers, quotes, brackets, etc.
            - Keep ALL Han characters, in their original order.
            3) Drop empty tokens.
            4) Output exactly one line: the cleaned tokens joined by a single ASCII comma \",\" with no spaces.
            Rules:
            - Never output characters that did not appear as Han characters in the input.
            - Never combine tokens or split a token beyond removing non-Han characters.
            - Words may be 1-6 Han characters; do not drop 1-character words.

            Example:
            Input: 放fàng松sōng,,收shōu垃lā圾jī,,收shōu购gòu
            Output: 放松,收垃圾,收购

            Input: {}
        "#,
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
                Make each sentence around 9 words long. Limit to 20 sentences. Don't include where the actual characeter should be in the sentence.",
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
        max_tokens: 1500,
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