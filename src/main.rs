use std::path::PathBuf;

use anyhow::Result;
use clap::{ArgGroup, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Debug, Clone)]
enum Format {
    Wav,
    Mp3,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(version, about = "Lists instruments from SoundFont")]
    Instruments {
        /// Path to SoundFont file
        #[arg(value_name = "FILE")]
        sf2: Option<PathBuf>,
    },
    #[command(version, about = "Renders audio file from Soundfont and MIDI", group(
        ArgGroup::new("instrument")
            .args(&["preset", "bank"])
            .multiple(true)
            .requires_all(&["preset", "bank"])
    ))]
    Synthesize {
        /// Path to SoundFont file
        #[arg(short, long, value_name = "FILE")]
        sf2: Option<PathBuf>,

        /// Path to MIDI file
        #[arg(short, long, value_name = "FILE")]
        midi: Option<PathBuf>,

        /// Output format
        #[arg(short, long, default_value = "wav")]
        format: Option<Format>,

        /// Preset patch number
        #[arg(short, long)]
        preset: Option<i32>,

        /// Preset bank number
        #[arg(short, long)]
        bank: Option<i32>,

        /// Output path
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    println!("Hello, world!");
}
