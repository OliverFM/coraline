use log;
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use std::io::copy;

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Voice {
    Alloy,
    Echo,
    Fable,
    Onyx,
    #[default]
    Nova,
    Shimmer,
}

pub async fn tts(
    voice: Voice,
    input_file: &str, // TODO: instead take in a string only
    output_file: &str,
    api_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_url = "https://api.openai.com/v1/audio/speech";
    let mut dest = std::fs::File::create(output_file)?;
    let input = std::fs::read_to_string(input_file)?;

    log::info!("Voice is: {:?}", voice);
    let body = json!({
        "model": "tts-1",
        "input": input,
        "voice": voice,
    })
    .to_string();
    log::debug!("Body is:\n{}", body);
    log::info!("Sending request to OpenAI's API...");
    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?;

    let status = response.status();
    log::info!("Response status: {}", status);
    if status.is_client_error() || status.is_server_error() {
        let error = response.text().await?;
        log::error!("Error: {}", error);
        return Err("Error from OpenAI's API".into());
    }
    let content = response.bytes().await?;
    copy(&mut content.as_ref(), &mut dest)?;
    log::info!("Successfully saved the audio to: {}", output_file);
    Ok(())
}
