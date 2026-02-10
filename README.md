# Midirave

Midirave is a small command-line tool that renders MIDI files to audio using SoundFont (`.sf2`) files via a [rustysynth](https://github.com/wsand02/rustysynth/tree/breakallthethings) library fork. 


## Features
- Lists instruments contained in a SoundFont.
- Synthesize MIDI files into WAV audio (44100 Hz, 16-bit stereo).
- Optional MIDI program change override thanks to the beforementioned rustysynth fork.

## Installation

### Install locally

```bash
git clone https://github.com/wsand02/midirave.git
cd midirave
cargo install --path .
```

### Build from source

```bash
git clone https://github.com/wsand02/midirave.git
cd midirave
cargo build --release
```

## Usage

### instruments

Lists all presets (instruments) contained in a SoundFont.

Usage:

```bash
# List instruments
midirave instruments /path/to/soundfont.sf2
```

Example output lines look like:

```
Instrument: Acoustic Grand Piano, Preset: 0, Bank: 0, Preset ID: 0
```

- `Preset` is the patch (program) number.
- `Bank` is the bank number.
- `Preset ID` is computed as `(bank << 16) | preset` (useful for compact identification).

### synthesize

Renders a MIDI file to an audio file.

Basic usage:

```bash
midirave synthesize -s /path/to/soundfont.sf2 -m /path/to/song.mid -o /path/to/out.wav
```

Options:

- `-s, --sf2 <FILE>`: SoundFont file (required)
- `-m, --midi <FILE>`: MIDI file (required)
- `-o, --output <FILE>`: Output audio file (required)
- `-f, --format <wav|mp3>`: Output format (default: `wav`) — currently only WAV output is produced. See notes below.
- `-p, --preset <INT>` and `-b, --bank <INT>`: Override the instrument used for rendering. If you supply one, you must supply both.

Examples:

- Render with the MIDI file's own program changes:
```bash
midirave synthesize -s FluidR3_GM.sf2 -m tune.mid -o tune.wav
```

- Render everything using a single instrument (preset 25, bank 0):
```bash
midirave synthesize -s FluidR3_GM.sf2 -m tune.mid -p 25 -b 0 -o tune-with-guitar.wav
```

Notes:
- If `preset` and `bank` are both provided, the synthesizer is instructed to override instruments so all channels use that instrument.
- The output WAV uses 2 channels (stereo), 44100 Hz sample rate, 16 bits per sample (signed integer).
- The `--format mp3` option exists in the CLI but currently does not produce MP3 directly, if you need MP3, convert the WAV with `ffmpeg` or [nami3](https://github.com/wsand02/nami3):

```bash
ffmpeg -i out.wav -codec:a libmp3lame -qscale:a 2 out.mp3
```

```bash
nami3 -i out.wav -o out.mp3
```

## Implementation Details & Behavior

- The synthesizer uses `rustysynth` and renders the entire MIDI into in-memory buffers before writing to disk. This can use substantial RAM for very long MIDI files.
- When invalid or out-of-range `preset`/`bank` values are supplied, the synthesizer typically ignores them silently (this behavior is delegated to the underlying library).
- Running the `instruments` command enumerates the SF2's presets and prints each one in a human-readable format.

## Limitations

- Overrides files without asking.

## License

Midirave is available under the MIT license. See [LICENSE](LICENSE).
