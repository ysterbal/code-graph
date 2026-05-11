use reqwest::Client;
use serde_json::{json, Value};
use std::env;

pub async fn send_to_llm(
    prompt: String,
    context: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();

    // Support OpenAI or OpenAI-compatible endpoints
    let base_url =
        env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let api_key = env::var("OPENAI_API_KEY").unwrap_or_default();
    let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

    let body = json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful coding assistant analyzing a codebase. Answer the user's question based on the provided context."
            },
            {
                "role": "user",
                "content": format!("Context:\n{}\n\nUser Question:\n{}", context, prompt)
            }
        ],
        "temperature": 0.2
    });

    let mut request = client.post(&url).json(&body);

    if !api_key.is_empty() {
        request = request.bearer_auth(api_key);
    }

    let response = request.send().await?;

    // Get raw text for better error debugging
    let status = response.status();
    let text = response.text().await?;

    if !status.is_success() {
        return Err(format!("HTTP error {}: {}", status, text).into());
    }

    let response_json: Value = serde_json::from_str(&text)
        .map_err(|e| format!("JSON decode error: {} (raw response: {})", e, text))?;

    if let Some(content) = response_json["choices"][0]["message"]["content"].as_str() {
        Ok(content.to_string())
    } else {
        Err(format!("Unexpected response format: {}", response_json).into())
    }
}
