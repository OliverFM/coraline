use clap::{Parser, Subcommand};
use env_logger::Env;
use log;
use tokio;

mod listen;
mod text_to_speech;

use text_to_speech::{tts, Voice};

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

    match std::path::Path::new(&args.output_file).try_exists() {
        Ok(true) => (),
        Ok(false) => {
            log::error!(
                "Output file already exists. Please provide a different file name.\nFile: {}",
                &args.output_file
            );
            return Err("Output file already exists.".into());
        }
        Err(e) => {
            log::error!("Could not check if the output file exists this likely means that the file path is invalid.

                        Exact Error for debugging:
                        {}", e);
            return Err("Could not check if the output file exists.".into());
        }
    };

    match args.command {
        Commands::Speak { voice } => {
            tts(voice, &args.input_file, &args.output_file, &api_key).await?;
        }
        Commands::Listen => {
            listen::listen(&args.input_file, &args.output_file, &api_key).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_composition() {
        let api_key = &std::env::var("OPENAI_API_KEY").unwrap();
        tts(Voice::Alloy, "sample.txt", "intermediate.mp3", api_key)
            .await
            .unwrap();
        listen::listen("intermediate.mp3", "testing_output.txt", api_key)
            .await
            .unwrap();
        std::fs::remove_file("intermediate.mp3").unwrap();
        let input_string = std::fs::read_to_string("sample.txt").unwrap();
        let converted_string = std::fs::read_to_string("testing_output.txt").unwrap();
        assert_eq!(input_string, converted_string);
        std::fs::remove_file("testing_output.txt").unwrap();
    }
}
