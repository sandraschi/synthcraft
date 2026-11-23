# SynthCraft

**Pure-Rust Modular DSP Synthesizer, 16-Step Acid Sequencer, Spectral Analyzer, CLAP 1.2.2 Plugin Bridge for SoundCraft & BPM/Camelot Stem Forge for DJCraft** (`L1 synthcraft-core` + `L2 synthcraft-mcp` + `L3 synthcraft-app`).

## Overview

SynthCraft is a 3-tier Pure-Rust ArtCraft modular audio synthesis studio and MCP `2026-07-28` server designed to work standalone or as a companion synth engine for **SoundCraft** (via CLAP `1.2.2` plugin ABI & 48 kHz `.wav` stems) and **DJCraft** (via BPM-locked, Camelot-keyed loops, acid basslines, and transition risers). It bundles 6 hierarchical workbenches:

1. **Modular Rack & VCO/VCF** -- dual oscillators (`Sine`, `Saw`, `Square`, `Triangle`, `PulsePWM`, `SuperSaw`, `WhiteNoise`, `SubOctave`) with detune, FM index, and 4-pole resonant ladder filter (`LowPass24dB`, `BandPass12dB`, `HighPass12dB`, `AcidResonant`).
2. **ADSR, LFO & Step Sequencer** -- 16-step BPM-locked sequencer (`36..84` MIDI notes, velocity, gate %, accent, TB-303 slide) plus ADSR envelope and tempo-synced LFO modulation.
3. **Oscilloscope & Spectrum** -- 256-sample real-time waveform scope, 32-band harmonic DFT/FFT spectrum analyzer, RMS/Peak dBFS metering, crest factor, and zero-crossing pitch tracker (`Hz`).
4. **CLAP Plugin & SoundCraft** -- native CLAP (`u-he` / Bitwig Clever Audio Plug-in `1.2.2`, ID `dev.sandraschi.synthcraft`) plugin descriptor, polyphonic parameter automation table (`0x1001..0x100C`), and C-ABI plugin bundle manifest for **SoundCraft**.
5. **DJCraft Stem & WAV Forge** -- exports 48 kHz / 44.1 kHz 16-bit PCM `.wav` loops, risers, sub-drops, and stem metadata (`bpm`, `camelot_key`, `bars`, `cue_points`) directly compatible with **DJCraft** decks.
6. **Assistant & 6-Tab Help** -- local-first Patch Co-Designer Chat and comprehensive 6-tab reference documentation.

## Feature Matrix

| Capability | Status | Crate / Module |
|---|---|---|
| Dual VCO (`8` waveforms, FM/detune) + 4-Pole Resonant Ladder VCF (`4` modes) | `Implemented` | `synthcraft-core::model` |
| ADSR Envelope + Tempo-Synced LFO + 16-Step TB-303 Acid Sequencer | `Implemented` | `synthcraft-core::model` |
| 32-Band DFT Spectrum Analyzer, Oscilloscope & RMS/Peak dBFS Metering | `Implemented` | `synthcraft-core::model` |
| CLAP `1.2.2` Plugin Descriptor & Parameter Automation Bridge for SoundCraft | `Implemented` | `synthcraft-core::model` |
| RIFF PCM `.wav` Renderer + BPM/Camelot Stem & Riser Exporter for DJCraft | `Implemented` | `synthcraft-core::model` |
| Bundled SQLite Persistence (`%LOCALAPPDATA%\synthcraft\synthcraft.db`) | `Implemented` | `synthcraft-core::db` |
| CLI (`--json`) + MCP `2026-07-28` Stdio Server (`14` tools) | `Implemented` | `synthcraft-mcp` |
| Native `eframe`/`egui` 0.37 Studio (6 Workbenches, WCAG AAA Dark/Light) | `Implemented` | `synthcraft-app` |

## Quick Start

```powershell
# Launch desktop studio
.\start.ps1

# Run headless smoke test
.\start.ps1 -Smoke

# Run CLI commands
cargo run -p synthcraft-mcp -- status --json
cargo run -p synthcraft-mcp -- list-patches
cargo run -p synthcraft-mcp -- analyze-spectrum --duration-ms 500
cargo run -p synthcraft-mcp -- export-clap
cargo run -p synthcraft-mcp -- export-djcraft-stem --bars 4

# Run MCP 2026-07-28 stdio server
cargo run -p synthcraft-mcp -- --mcp
```

## License

Dual-licensed under `MIT OR Apache-2.0`.
