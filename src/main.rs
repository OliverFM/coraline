use reqwest::Client;
use serde_json::json;
use std::fs::File;
use std::io::copy;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: String,

    #[arg(short, long)]
    output_file: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_url = "https://api.openai.com/v1/audio/speech";
    let api_key = std::env::var("OPENAI_API_KEY").expect("you should have an api key"); // Replace with your actual API key
    let args = Args::parse();

    let input = std::fs::read_to_string(&args.input_file)?;

    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(
            json!({
                "model": "tts-1",
                "input": input,
                "voice": "alloy"
            })
            .to_string(),
        )
        .send()
        .await?;

    let mut dest = { std::fs::File::create(&args.output_file)? };
    let content = response.bytes().await?;
    copy(&mut content.as_ref(), &mut dest)?;

    Ok(())
}
