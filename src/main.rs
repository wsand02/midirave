use std::{
    fmt::{Display, format},
    fs::File,
    path::PathBuf,
    sync::Arc,
};

use anyhow::{Context, Result};
use clap::{ArgGroup, Parser, Subcommand, ValueEnum};
use rustysynth::{
    MidiFile, MidiFileSequencer, Preset, SoundFont, Synthesizer, SynthesizerError,
    SynthesizerSettings,
};

fn sequence(
    midi: &Arc<MidiFile>,
    soundfont: &Arc<SoundFont>,
    preset: Option<i32>,
    bank: Option<i32>,
) -> Result<(Vec<f32>, Vec<f32>), SynthesizerError> {
    let mut settings = SynthesizerSettings::new(44100);
    if preset.is_some() && bank.is_some() {
        settings.instrument_override = true;
    }
    let mut synthesizer = Synthesizer::new(soundfont, &settings)?;
    // if preset or bank are incorrect, the synthesizer will silently handle them...
    // and if override is false this line wont do anything...
    synthesizer.set_override_preset(bank.unwrap_or(0), preset.unwrap_or(0));
    // synthesizer.process_midi_message(channel, 0xB0, 0x00, 0x00); // Bank 0
    // synthesizer.process_midi_message(channel, 0xC0, 0x19, 0x00); // Preset 25 steel guitar
    let mut sequencer = MidiFileSequencer::new(synthesizer);

    sequencer.play(midi, false);

    let sample_count = (settings.sample_rate as f64 * midi.get_length()) as usize;
    let mut left: Vec<f32> = vec![0_f32; sample_count];
    let mut right: Vec<f32> = vec![0_f32; sample_count];
    sequencer.render(&mut left[..], &mut right[..]);

    Ok((left, right))
}

struct Instrument {
    name: String,
    preset: i32,
    bank: i32,
    preset_id: i32,
}

impl Display for Instrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Instrument: {}, Preset: {}, Bank: {}, Preset ID: {}",
            self.name, self.preset, self.bank, self.preset_id
        )
    }
}

fn sf2_read(sf2_path: &PathBuf) -> Result<SoundFont> {
    let mut idk = File::open(sf2_path).with_context(|| format!("Failed to open Soundfont"))?;
    SoundFont::new(&mut idk).with_context(|| format!("Error reading soundfont"))
}

fn instruments(sf2_path: &PathBuf) -> Result<()> {
    let sf_result = sf2_read(sf2_path)?;
    let presets = sf_result.get_presets();
    let mut instruments: Vec<Instrument> = Vec::new();
    for inst in presets.iter().enumerate() {
        let (_, pre): (usize, &Preset) = inst;
        let new_inst = Instrument {
            name: pre.get_name().to_string().clone(),
            preset: pre.get_patch_number(),
            bank: pre.get_bank_number(),
            preset_id: (pre.get_bank_number() << 16) | pre.get_patch_number(), // i forgor
        };
        println!("{new_inst}");
        instruments.push(new_inst);
    }
    Ok(())
}

fn synthesize() {}

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
        sf2: PathBuf,
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Instruments { sf2 } => instruments(sf2),
        Commands::Synthesize { .. } => todo!(),
    }
}
