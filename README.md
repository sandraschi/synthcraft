# SynthCraft

**Pure-Rust Modular DSP Synthesizer, 64-Step Acid Sequencer, 8-Voice TR-808/909 Drum Machine, 6-Op FM, Granular/Spectral, Neural DDSP, 16-Band Vocoder, Mastering Chain, CLAP 1.2.2 Plugin (SoundCraft/REAPER), Native VCV Rack 2 Module & DJCraft Stem Forge** (`L1 synthcraft-core` + `L2 synthcraft-mcp` + `L3 synthcraft-app`).

## Overview

SynthCraft is a 3-tier Pure-Rust ArtCraft modular audio synthesis studio and MCP `2026-07-28` server (`60` tools) designed to work standalone, as a native **CLAP `1.2.2` instrument** in **SoundCraft** and **REAPER**, as a native **VCV Rack 2** module, or live-linked with **DJCraft** (`DJLink`, `DJPD` pads, `DJFX` Aux FX). It bundles **14 hierarchical workbenches (37 subpages)** with 5 hardware skins, 4 switchable studio layouts, and GPU-rendered rotary knobs/faders:

1. **VCV 2 Eurorack & VCO/VCF** -- multi-row scrollable 3U Eurorack skiff (`13` playable modules including `Wavefolder + LPG`, `S&H + Slew`, `Stereo Delay`, `Euclidean Clock`, `Attenuverter`, `3U Scope`, and `CLAP HOST` external plugin slot), `12` curated pre-patched Rack Rigs, dual oscillators (`10` waveforms incl. morphing `Wavetable`) with detune, FM index, and 4-pole resonant ladder filter.
2. **1990-Style PCM Workstation & Extreme DSP** -- AI2 dual-layer Combi stacking (`8` CC0 banks incl. 2 cembalos & 2 pipe organs, 1988 32 kHz/12-bit DAC crunch) + Extreme DSP modes.
3. **MIDI Keyboard & MIDI Depot** -- live USB MIDI (`midir`), 25-key studio piano, built-in MIDI Depot & `.mid` export.
4. **Demo Showcase & Audition** -- 1-click club, acid, PCM, and West-Coast audition cards.
5. **64-Step Sequencer, TR-808/909 Drum Machine & ADSR/LFO** -- 16/32/48/64-step (4-bar) BPM-locked acid sequencer with Live Record & SMF `.mid` import/export, **8-voice pure-Rust TR-808/909 analog/FM drum machine** (BD/SD/CP/CH/OH with choke/LT/CB/CY, swing, Euclidean fills, 6 kits), 8-route Mod Matrix, Arpeggiator, and Scala `.scl` microtuning.
6. **Oscilloscope & 32-Band DFT Spectrum** -- real-time waveform scope, 32-band spectrum, RMS/Peak dBFS metering, and pitch tracker.
7. **CLAP `1.2.2` & SoundCraft Bridge** -- native `SynthCraft.clap` instrument (ID `dev.sandraschi.synthcraft`) with full **MPE & CLAP Poly-Mod** (`CLAP_EVENT_NOTE_EXPRESSION` + `CLAP_EVENT_PARAM_MOD`), verified live in **SoundCraft** and **REAPER**.
8. **DJCraft Stem & WAV Forge** -- live `DJLink` BPM/Camelot sync, `DJPD` pad receiver, `DJFX` stereo Aux FX input, and 1-click Forge & Load `.wav` stems.
9. **Assistant & 6-Tab Help** -- Patch Co-Designer Chat and 6-tab reference docs.
10. **Studio Settings & Preferences (`Ctrl+,`)** -- 6-tab audio driver, MIDI/MPE, skin/knob physics, DAC/DSP tuning, CLAP/DJCraft paths, and SQLite settings.
11. **Next-Gen Lab (MSEG / Granular / Spectral / Genopatch)** -- Drawable 16-point MSEG splines + Dual Morphing Filter B (Formant A-E-I-O-U, Comb +/-, MS-20 Screamer) with animated knob mod rings, 24-grain live-capture Granular Cloud, 64-Partial Spectral Additive Sculptor, and **Multi-Core Evolutionary Genopatch** audio-to-patch matcher.
12. **Pro Lab (6-Op FM / Unison / Reorderable FX / 2D Physics & 8x WDF)** -- 6x6 Thru-Zero FM Matrix, 16-voice Hyper-Unison stack, Reorderable Studio FX Rack (OTT 3-band, Convolution IR, Shimmer, Analog Tape & Bitcrusher), **8x8 2D FDTD Physical Modeling Mesh**, and **8x Oversampled WDF Analog Circuits** (Moog CP3, Buchla 292 Vactrol LPG, TB-303 diode ladder).
13. **Visuals, MPE & Neural DDSP** -- 24-slice 3D perspective scrolling STFT Waterfall Spectrogram, Lissajous Stereo Vectorscope + phase correlation meter, 2D MPE Expression Touch Pad, and **Neural DDSP Timbre Transfer** (32-harmonic + filtered-noise decoder with 6 latent instrument embeddings).
14. **Performance & Mastering Suite** -- 1-Finger Polyphonic Chord Memory (10 voicings) + Scale Lock + Riff Generator, **16-Band Analog Vocoder** (built-in robotic speech or live `DJFX` vocals), **8-Scene Live Crossfader & 4-Corner XY Morph Pad**, and **Studio Mastering Bus** (4-Band EQ, Sub-Bass Mono Maker `<120 Hz`, M/S Stereo Width, True-Peak Brickwall Limiter).

## Feature Matrix

| Capability | Status | Crate / Module |
|---|---|---|
| Dual VCO (`10` waveforms incl. morphing `Wavetable`, FM/detune) + 4-Pole Resonant Ladder VCF (`4` modes) | `Implemented` | `synthcraft-core::model`, `synthcraft-core::wavetable` |
| Multi-row 3U Eurorack skiff (`13` modules, draggable DSP-wired cables, `12` pre-patched Rack Rigs) | `Implemented` | `synthcraft-core::rack`, `synthcraft-core::rack_presets`, `synthcraft-app::rack_canvas` |
| 3U CLAP `1.2.2` Host module (loads external `.clap` plugins with live `CLAP_EVENT_PARAM_MOD` CV modulation) | `Implemented` | `synthcraft-clap-host`, `synthcraft-core::clap_host` |
| General Modulation Matrix (`8` bipolar routes), Real-Time Arpeggiator & Scala `.scl` Microtonal Tuning | `Implemented` | `synthcraft-core::{mod_matrix, arp, tuning}` |
| ADSR Envelope + Tempo-Synced LFO + 64-Step (4-Bar) Acid Sequencer (Live Record + SMF `.mid` import/export) | `Implemented` | `synthcraft-core::{model, smf_import}`, `synthcraft-app::live` |
| 8-Voice Pure-Rust TR-808/909 Analog & FM Drum Machine (64 steps, Hi-Hat choke, swing, Euclidean fills, 6 kits) | `Implemented` | `synthcraft-core::drums`, `synthcraft-app::drums_ui` |
| Drawable 16-Point MSEG Envelopes + Dual Morphing Filter B (Formant A-E-I-O-U, Comb +/-, MS-20) + Animated Knob Mod Rings | `Implemented` | `synthcraft-core::nextgen`, `synthcraft-app::{controls, nextgen_ui}` |
| 24-Grain Live-Capture Granular Cloud + 64-Partial Spectral Additive Resynthesis | `Implemented` | `synthcraft-core::granular`, `synthcraft-app::nextgen_ui` |
| Multi-Core Evolutionary Genopatch Audio-to-Patch Matcher (`std::thread::scope`) | `Implemented` | `synthcraft-core::genopatch`, `synthcraft-app::nextgen_ui` |
| 6-Operator Thru-Zero FM Matrix (6x6) + 16-Voice Hyper-Unison Stack | `Implemented` | `synthcraft-core::fm_unison`, `synthcraft-app::pro_lab_ui` |
| Reorderable Studio FX Rack (OTT 3-Band, Convolution IR Reverb, Shimmer Reverb, Analog Tape & Bitcrusher) | `Implemented` | `synthcraft-core::fx_rack`, `synthcraft-app::pro_lab_ui` |
| Goliath 8x8 2D FDTD Physical Modeling Mesh + 8x Oversampled WDF Circuits (Moog CP3, Buchla 292 LPG, TB-303) | `Implemented` | `synthcraft-core::physical`, `synthcraft-app::pro_lab_ui` |
| Full MPE & CLAP 1.2.2 Poly-Mod + 3D Waterfall Spectrogram + Lissajous Vectorscope + Neural DDSP Timbre Transfer | `Implemented` | `synthcraft-core::{rt, neural_ddsp}`, `synthcraft-clap`, `synthcraft-app::visualizers_ui` |
| 1-Finger Chord Memory / Scale Lock + 16-Band Analog Vocoder + 8-Scene XY Morph + Studio Mastering Chain | `Implemented` | `synthcraft-core::{chord_mem, vocoder, scenes, mastering}`, `synthcraft-app::perf_master_ui` |
| Undo/Redo (`Ctrl+Z`/`Ctrl+Y`, 64-deep), A/B patch compare & 3U rack accessibility metadata | `Implemented` | `synthcraft-app` |
| WebAssembly AudioWorklet engine (`synthcraft_wasm.wasm` ~190 KB + `synthcraft-worklet.js`) | `Implemented` | `synthcraft-wasm` |
| 32-Band DFT Spectrum Analyzer, Oscilloscope & RMS/Peak dBFS Metering | `Implemented` | `synthcraft-core::model` |
| CLAP `1.2.2` instrument plug-in (`SynthCraft.clap` with `CLAP_EXT_GUI`, MPE & Poly-Mod): passes `clap-validator` | `Implemented; live-verified in SoundCraft & REAPER` | `synthcraft-clap`, `synthcraft-core::plugin` |
| VCV Rack 2 module (`SynthCraft 2.0.0`, polyphonic V/Oct, gate, CV inputs, bank loading, context menus) over C ABI | `Implemented; live-loaded in VCV Rack Free 2.6.6 (Win x64)` | `rack/`, `synthcraft-ffi` |
| CLAP descriptor and parameter map export for SoundCraft | `Implemented` | `synthcraft-core::model` |
| RIFF PCM `.wav` Renderer + BPM/Camelot Stem & Riser Exporter for DJCraft | `Implemented` | `synthcraft-core::model` |
| Bundled SQLite Persistence (`%LOCALAPPDATA%\synthcraft\synthcraft.db`), schema v2, backup before migration, bad rows quarantined, saves off the UI thread | `Implemented` | `synthcraft-core::db`, `synthcraft-app::saver` |
| Live audio (cpal) and MIDI input (midir) through the same `SynthRt` engine as offline rendering | `Implemented` | `synthcraft-core::rt`, `synthcraft-app` |
| Sampler: WAV, SFZ, SF2 banks, loop points, automatic sustain loops, free CC0 bank downloader | `Implemented` | `synthcraft-core::sampler`, `synthcraft-core::banks` |
| Atmospheric pad oscillator, chorus, reverb | `Implemented` | `synthcraft-core::dsp` |
| Agent file sandbox (`--automation-read-root` / `--automation-write-root`, required for MCP) | `Implemented` | `synthcraft-core::sandbox` |
| CLI (`--json`) + MCP `2026-07-28` Stdio Server (`60` tools) | `Implemented` | `synthcraft-mcp` |
| Native `eframe`/`egui` 0.37 Studio (`14` Workbenches / `37` Subpages, 5 Hardware Skins, 4 Layouts, WCAG AAA) | `Implemented` | `synthcraft-app` |

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

## Real Sample Banks

The sampler loads WAV, SFZ and SF2 files (Banks folder: `Documents/SynthCraft/Banks`). Free CC0 packs can be fetched with checksum verification and an auto-generated SFZ:

```
synthcraft-mcp list-banks
synthcraft-mcp fetch-bank vsco2ce-cello-section-sus
```

Then enter the printed `.sfz` path in the sample bank field of the rack. Sample files are never stored in this repository; `banks/manifest.json` pins source commit, SHA-256 and licence. Bring your own SF2/SFZ files for anything else.

See `docs/SAMPLES_AND_M1.md` for licensing rules, adding packs and the M1 question, and `docs/ROADMAP.md` for plans (CLAP, VCV Rack module, features).

## Plug-ins

SynthCraft runs as a CLAP instrument and as a VCV Rack module, on the same engine as the desktop app.

```
cargo build --release -p synthcraft-clap
just clap-install        # copies SynthCraft.clap to %LOCALAPPDATA%\Programs\Common\CLAP
```

The Rack module is built from `rack/` against the Rack SDK. Full details, parameter ids, limits and what has been verified are in `docs/PLUGINS.md`.

## License

Dual-licensed under `MIT OR Apache-2.0`.
