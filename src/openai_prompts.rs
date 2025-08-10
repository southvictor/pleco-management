use serde::{Deserialize, Serialize};
use std::env;
use reqwest;

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



pub async fn generate_openai_prompt(
    character: &str,
    prompt_type: &str,
    context: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY environment variable not set")?;
    
    let base_prompt = match prompt_type {
        "translation" => format!(
            "Generate a english sentence with the definition of the chinese character '{}'. The purpose of the sentence is for someone to practice translating to easy to understand chinese.",
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

    let request = OpenAIRequest {
        model: "gpt-4.1".to_string(),
        messages: vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant specializing in Chinese language learning. Provide clear, accurate, and educational responses about Chinese characters, their meanings, usage, and cultural context.".to_string(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: full_prompt,
            },
        ],
        max_tokens: 500,
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