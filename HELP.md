# SynthCraft Help & User Guide

## Get Started

**What is SynthCraft?**
SynthCraft is a 3-tier Pure-Rust ArtCraft Modular Audio Synthesizer, 16-Step Acid Sequencer, Spectral Analyzer, CLAP (`1.2.2`) Plugin Bridge for **SoundCraft**, and BPM/Camelot Stem & Riser Forge for **DJCraft**.

It provides 6 integrated workbenches in the left sidebar:
1. **Modular Rack & VCO/VCF** -- dual anti-aliased oscillators (`Sine`, `Saw`, `Square`, `Triangle`, `PulsePWM`, `SuperSaw`, `WhiteNoise`, `SubOctave`) with detune, FM index, and 4-pole resonant ladder filter (`LowPass24dB`, `BandPass12dB`, `HighPass12dB`, `AcidResonant`).
2. **ADSR, LFO & Step Sequencer** -- 16-step BPM-locked sequencer (`36..84` MIDI notes, velocity, gate %, accent, acid slide) plus ADSR envelope and tempo-synced LFO modulation.
3. **Oscilloscope & Spectrum** -- 256-sample real-time waveform scope, 32-band harmonic DFT/FFT spectrum analyzer, RMS/Peak dBFS metering, crest factor, and zero-crossing fundamental frequency (`Hz`) tracker.
4. **CLAP Plugin & SoundCraft** -- native CLAP (`u-he` / Bitwig Clever Audio Plug-in `1.2.2`, ID `dev.sandraschi.synthcraft`) plugin descriptor, polyphonic parameter automation table (`clap_id`, modulatable flags), and preset exchange with **SoundCraft**.
5. **DJCraft Stem & WAV Forge** -- exports 48 kHz / 44.1 kHz 16-bit PCM `.wav` loops, risers, sub-drops, and stem metadata (`bpm`, `camelot_key`, `bars`, `cue_points`) directly compatible with **DJCraft** decks and **SoundCraft** tracks.
6. **Assistant & Help** -- local-first Patch Co-Designer Chat and 6-tab interactive documentation.

## Functions

- **6 Factory Patches**: `vienna_acid_303`, `goliath_reese_sub`, `danube_dub_chord`, `stephansdom_bell_fm`, `djcraft_riser_sweep`, `soundcraft_warm_pad`.
- **SQLite Persistence**: Auto-saves patch parameters, 16-step sequences, and custom user patches to `%LOCALAPPDATA%\synthcraft\synthcraft.db` (or `--db <PATH>`).

## Usage

### CLI
```powershell
synthcraft-mcp status --json
synthcraft-mcp list-patches
synthcraft-mcp load-patch --id vienna_acid_303
synthcraft-mcp analyze-spectrum --duration-ms 500
synthcraft-mcp export-clap
synthcraft-mcp export-djcraft-stem --bars 4
synthcraft-mcp render-wav --path loop_128bpm_8A.wav --duration-ms 2000
```

### MCP Server (`2026-07-28` stdio)
```powershell
synthcraft-mcp --mcp
```
Exposes 14 MCP tools: `synth_status`, `synth_summary`, `list_patches`, `load_patch`, `update_oscillators`, `update_filter`, `update_envelope_lfo`, `set_sequencer_step`, `synthesize_preview`, `analyze_spectrum`, `render_wav_file`, `export_clap_manifest`, `export_djcraft_stem`, and `reset_factory_patches`.

## Tips

- **SoundCraft Integration**: SoundCraft hosts CLAP (`1.2.2`) plugins rather than Steinberg VST3. Use Workbench 4 (`CLAP Plugin & SoundCraft`) to inspect the `dev.sandraschi.synthcraft` CLAP descriptor, parameter IDs (`0x1001..0x100C`), and C-ABI entry point, or render 48 kHz `.wav` stems directly into a SoundCraft session.
- **DJCraft Integration**: Use Workbench 5 (`DJCraft Stem & WAV Forge`) to lock your patch to a Camelot key (`8A Am`, `4A Fm`, etc.) and deck BPM (`120..174 BPM`) before exporting a 4-bar loop or transition riser.
- Press `Ctrl+K` for the Command Palette, `F1` for Help, and `Ctrl+MouseWheel` to zoom the UI.

## Other Crafts

SynthCraft is part of the `sandraschi` Pure-Rust ArtCraft ecosystem alongside `djcraft`, `musiccraft`, `logiccraft`, `packetcraft`, `graphcraft`, `robotcraft`, and `smbcraft`.
