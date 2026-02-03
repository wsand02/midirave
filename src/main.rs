use std::{
    fmt::Display,
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
    sync::Arc,
};

use anyhow::{Context, Result};
use clap::{ArgGroup, Parser, Subcommand, ValueEnum};
use hound::{SampleFormat, WavSpec, WavWriter};
use rustysynth::{
    MidiFile, MidiFileSequencer, Preset, SoundFont, Synthesizer, SynthesizerError,
    SynthesizerSettings,
};

fn wav_encode(left: &Vec<f32>, right: &Vec<f32>, output: PathBuf) -> Result<()> {
    let spec = WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let file: File = File::create(&output).context("Failed to create output file")?;
    let mut buffer = BufWriter::new(file);
    let mut writer = WavWriter::new(&mut buffer, spec).context("Failed to create WAV writer")?;

    for (l, r) in left.iter().zip(right.iter()) {
        writer.write_sample((*l * i16::MAX as f32) as i16)?;
        writer.write_sample((*r * i16::MAX as f32) as i16)?;
    }

    writer.finalize().context("Failed to finalize WAV file")?;
    Ok(())
}

fn pcm_encode(left: &[f32], right: &[f32]) -> Result<()> {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    for (l, r) in left.iter().zip(right.iter()) {
        let l = (l * i16::MAX as f32) as i16;
        let r = (r * i16::MAX as f32) as i16;

        out.write_all(&l.to_le_bytes())?;
        out.write_all(&r.to_le_bytes())?;
    }

    Ok(())
}

fn sequence(
    midi: &Arc<MidiFile>,
    soundfont: &Arc<SoundFont>,
    preset: &Option<i32>,
    bank: &Option<i32>,
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
    let mut idk = File::open(sf2_path).context("Failed to open Soundfont")?;
    SoundFont::new(&mut idk).context("Error reading Soundfont")
}

fn instruments(sf2_path: &PathBuf) -> Result<()> {
    let sf = sf2_read(sf2_path)?;
    let presets = sf.get_presets();
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

fn midi_read(midi: &PathBuf) -> Result<MidiFile> {
    let mut midi_file = File::open(midi).context("Failed to open MIDI file")?;

    MidiFile::new(&mut midi_file).context("Error reading MIDI file")
}

fn synthesize(
    sf2_path: &PathBuf,
    midi_path: &PathBuf,
    preset: &Option<i32>,
    bank: &Option<i32>,
) -> Result<()> {
    let sf = sf2_read(sf2_path)?;
    let midiff = midi_read(midi_path)?;
    let (left, right) = sequence(&Arc::new(midiff), &Arc::new(sf), preset, bank)?;
    // wav_encode(&left, &right, output_path.clone())?;
    pcm_encode(&left, &right)?;
    Ok(())
}

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
        sf2: PathBuf,

        /// Path to MIDI file
        #[arg(short, long, value_name = "FILE")]
        midi: PathBuf,

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
        Commands::Synthesize {
            sf2,
            midi,
            format: _,
            preset,
            bank,
            output: _,
        } => synthesize(sf2, midi, preset, bank),
    }
}
