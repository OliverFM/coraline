use reqwest::Client;
use serde_json::json;
use std::io::copy;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: String,

    #[arg(short, long)]
    output_file: String,

    #[arg(long, default_value_t = String::from("alloy"))]
    voice: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_url = "https://api.openai.com/v1/audio/speech";
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("\nYou need to have OPENAI_API_KEY present in your env.\n");
    let args = Args::parse();

    let mut dest = std::fs::File::create(&args.output_file)?;
    let input = std::fs::read_to_string(&args.input_file)?;

    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(
            json!({
                "model": "tts-1",
                "input": input,
                "voice": args.voice,
            })
            .to_string(),
        )
        .send()
        .await?;

    let content = response.bytes().await?;
    copy(&mut content.as_ref(), &mut dest)?;

    Ok(())
}
