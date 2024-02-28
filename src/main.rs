use clap::{Parser, Subcommand};
use env_logger::Env;
use log;
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use std::io::copy;
use tokio;

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Voice {
    #[default]
    Alloy,
    Echo,
    Fable,
    Onyx,
    Nova,
    Shimmer,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg(
        short,
        long,
        help = "The path to the file that you would like coraline to read"
    )]
    input_file: String,

    #[arg(short, long, help = "Path to the save the output audio.")]
    output_file: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(alias("text-to-speech"))]
    Speak {
        #[arg(long, value_enum, default_value_t=Voice::Alloy, help = "The voice to use.")]
        voice: Voice,
    },

    #[clap(alias("speech-to-text"))]
    Listen,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("\nYou need to have OPENAI_API_KEY present in your env.\n");
    let args = Args::parse();

    if std::path::Path::new(&args.output_file).exists() {
        log::error!(
            "Output file already exists. Please provide a different file name.\nFile: {}",
            &args.output_file
        );
        return Err("Output file already exists.".into());
    };

    match args.command {
        Commands::Speak { voice } => {
            tts(voice, &args.input_file, &args.output_file, &api_key).await?;
        }
        Commands::Listen => {
            listen(&args.input_file, &args.output_file, &api_key).await?;
        }
    }

    Ok(())
}

async fn listen(
    input_file: &str,
    output_file: &str,
    api_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_url = "https://api.openai.com/v1/audio/transcriptions";
    let mut dest = std::fs::File::create(output_file)?;

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
    copy(&mut content.as_ref(), &mut dest)?;
    log::info!("Successfully saved the text to: {}", output_file);
    Ok(())
}

async fn tts(
    voice: Voice,
    input_file: &str,
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

