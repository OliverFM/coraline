use log;
use reqwest::Client;
use std::io::copy;
use tokio;

pub async fn listen(
    input_file: &str,
    output_file: &str,
    api_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_url = "https://api.openai.com/v1/audio/transcriptions";

    let audio_file: tokio::fs::File = match tokio::fs::File::open(input_file).await {
        Ok(f) => f,
        Err(e) => {
            log::error!("Oh no! Could not read your audio file: {}", input_file);
            return Err(e.into());
        }
    };

    let mime_type = mime_guess::from_path(input_file).first_or_octet_stream();
    let form = reqwest::multipart::Form::new()
        .text("model", "whisper-1")
        .text("response_format", "text")
        .part(
            "file",
            reqwest::multipart::Part::stream(audio_file)
                .file_name("file")
                .mime_str(&mime_type.to_string())?,
        )
        .text("model", "whisper-1");
    log::info!("Sending request to OpenAI's API...");
    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
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

    let mut dest = std::fs::File::create(output_file)?;
    copy(&mut content.as_ref(), &mut dest)?;
    log::info!("Successfully saved the text to: {}", output_file);
    Ok(())
}
