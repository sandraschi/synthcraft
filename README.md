# SynthCraft

**Pure-Rust Modular DSP Synthesizer, 16-Step Acid Sequencer, Spectral Analyzer, CLAP 1.2.2 Plugin Bridge for SoundCraft & BPM/Camelot Stem Forge for DJCraft** (`L1 synthcraft-core` + `L2 synthcraft-mcp` + `L3 synthcraft-app`).

## Overview

SynthCraft is a 3-tier Pure-Rust ArtCraft modular audio synthesis studio and MCP `2026-07-28` server designed to work standalone or as a companion synth engine for **SoundCraft** (via CLAP `1.2.2` plugin ABI & 48 kHz `.wav` stems) and **DJCraft** (via BPM-locked, Camelot-keyed loops, acid basslines, and transition risers). It bundles 6 hierarchical workbenches:

1. **Modular Rack & VCO/VCF** -- multi-row scrollable 3U Eurorack skiff (`13` playable modules including `Wavefolder + LPG`, `S&H + Slew`, `Stereo Delay`, `Euclidean Clock`, `Attenuverter`, `3U Scope`, and `CLAP HOST` external plugin slot), `12` curated pre-patched Rack Rigs, dual oscillators (`10` waveforms: `Sine`, `Saw`, `Square`, `Triangle`, `PulsePWM`, `SuperSaw`, `WhiteNoise`, `SubOctave`, `AtmoPad`, `Wavetable`) with detune, FM index, and 4-pole resonant ladder filter (`LowPass24dB`, `BandPass12dB`, `HighPass12dB`, `AcidResonant`).
2. **ADSR, LFO, Mod Matrix, Arp & Step Sequencer** -- 16-step BPM-locked sequencer (`36..84` MIDI notes, velocity, gate %, accent, acid slide) with Live Sequencer Record and SMF `.mid` import/export, General Modulation Matrix (8 bipolar routes), Real-Time Arpeggiator, Scala `.scl` microtonal tuning, ADSR envelope, and tempo-synced LFO modulation.
3. **Oscilloscope & Spectrum** -- 256-sample real-time waveform scope, 32-band harmonic DFT/FFT spectrum analyzer, RMS/Peak dBFS metering, crest factor, and zero-crossing pitch tracker (`Hz`).
4. **CLAP Plugin & SoundCraft** -- native CLAP (`u-he` / Bitwig Clever Audio Plug-in `1.2.2`, ID `dev.sandraschi.synthcraft`) instrument plug-in (`SynthCraft.clap`, passes `clap-validator`) plus polyphonic parameter automation table (`0x1001..0x1019`) exported as JSON for **SoundCraft**.
5. **DJCraft Stem & WAV Forge** -- exports 48 kHz / 44.1 kHz 16-bit PCM `.wav` loops, risers, sub-drops, and stem metadata (`bpm`, `camelot_key`, `bars`, `cue_points`) directly compatible with **DJCraft** decks.
6. **Assistant & 6-Tab Help** -- local-first Patch Co-Designer Chat and comprehensive 6-tab reference documentation.

## Feature Matrix

| Capability | Status | Crate / Module |
|---|---|---|
| Dual VCO (`10` waveforms incl. morphing `Wavetable`, FM/detune) + 4-Pole Resonant Ladder VCF (`4` modes) | `Implemented` | `synthcraft-core::model`, `synthcraft-core::wavetable` |
| Multi-row 3U Eurorack skiff (`13` modules, draggable DSP-wired cables, `12` pre-patched Rack Rigs) | `Implemented` | `synthcraft-core::rack`, `synthcraft-core::rack_presets`, `synthcraft-app::rack_canvas` |
| 3U CLAP `1.2.2` Host module (loads external `.clap` plugins with live `CLAP_EVENT_PARAM_MOD` CV modulation) | `Implemented` | `synthcraft-clap-host`, `synthcraft-core::clap_host` |
| General Modulation Matrix (`8` bipolar routes), Real-Time Arpeggiator & Scala `.scl` Microtonal Tuning | `Implemented` | `synthcraft-core::{mod_matrix, arp, tuning}` |
| ADSR Envelope + Tempo-Synced LFO + 16-Step Acid Sequencer (Live Record + SMF `.mid` import/export) | `Implemented` | `synthcraft-core::{model, smf_import}`, `synthcraft-app::live` |
| Undo/Redo (`Ctrl+Z`/`Ctrl+Y`, 64-deep), A/B patch compare & 3U rack accessibility metadata | `Implemented` | `synthcraft-app` |
| WebAssembly AudioWorklet engine (`synthcraft_wasm.wasm` ~190 KB + `synthcraft-worklet.js`) | `Implemented` | `synthcraft-wasm` |
| 32-Band DFT Spectrum Analyzer, Oscilloscope & RMS/Peak dBFS Metering | `Implemented` | `synthcraft-core::model` |
| CLAP `1.2.2` instrument plug-in (`SynthCraft.clap` with native `CLAP_EXT_GUI` hardware UI): 25 automatable parameters, notes, state; passes `clap-validator` | `Implemented; not yet tried in a DAW` | `synthcraft-clap`, `synthcraft-core::plugin` |
| VCV Rack module (polyphonic V/Oct and gate, CV inputs, bank loading, right-click VCO/VCF/wavetable menus) over a C ABI | `Builds and links against the Rack SDK; not yet loaded in Rack` | `rack/`, `synthcraft-ffi` |
| CLAP descriptor and parameter map export for SoundCraft | `Implemented` | `synthcraft-core::model` |
| RIFF PCM `.wav` Renderer + BPM/Camelot Stem & Riser Exporter for DJCraft | `Implemented` | `synthcraft-core::model` |
| Bundled SQLite Persistence (`%LOCALAPPDATA%\synthcraft\synthcraft.db`), schema v2, backup before migration, bad rows quarantined, saves off the UI thread | `Implemented` | `synthcraft-core::db`, `synthcraft-app::saver` |
| Live audio (cpal) and MIDI input (midir) through the same `SynthRt` engine as offline rendering | `Implemented` | `synthcraft-core::rt`, `synthcraft-app` |
| Sampler: WAV, SFZ, SF2 banks, loop points, automatic sustain loops, free CC0 bank downloader | `Implemented` | `synthcraft-core::sampler`, `synthcraft-core::banks` |
| Atmospheric pad oscillator, chorus, reverb | `Implemented` | `synthcraft-core::dsp` |
| Agent file sandbox (`--automation-read-root` / `--automation-write-root`, required for MCP) | `Implemented` | `synthcraft-core::sandbox` |
| CLI (`--json`) + MCP `2026-07-28` Stdio Server (`48` tools) | `Implemented` | `synthcraft-mcp` |
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
