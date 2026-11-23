# Changelog

## [Unreleased]

### Added
- Real-time audio (cpal) and MIDI input (midir) through the single `SynthRt` engine shared with offline rendering.
- Sampler with WAV, SFZ and SF2 loading, Hermite interpolation, loop points and automatic sustain loops; `AtmoPad` pad oscillator, chorus and reverb; ambient factory patches.
- `fetch-bank` / `list-banks` and an in-app downloader for checksum-verified CC0 sample packs (VSCO 2 CE strings), manifest in `banks/manifest.json`.
- Agent file sandbox: MCP requires `--automation-read-root` and `--automation-write-root`.
- 23 MCP tools and 26 CLI subcommands.
- `export_djcraft_stem` can render the mono loop to a WAV; manifest uses the engine sample rate and bar-grid cue points.
- CLAP instrument plug-in (`synthcraft-clap`, installs as `SynthCraft.clap`): stereo output, CLAP note and MIDI input, 22 automatable parameters, patch JSON state with parameter rescan; passes `clap-validator` 0.4.1 (34 passed, 0 failed).
- C ABI crate `synthcraft-ffi` (`include/synthcraft.h`, ABI version 1) and a VCV Rack module in `rack/` (polyphonic V/Oct and gate, velocity, CV for cutoff, resonance, morph and pulse width, bank loading, patch stored in the Rack patch).
- `synthcraft_core::plugin::PluginEngine` and `params` (stable parameter ids) shared by every front end; `SynthRt::patch_mut`.
- `docs/PLUGINS.md`, `docs/SAMPLES_AND_M1.md`, `docs/ROADMAP.md`; `just clap-install`.

### Changed
- SQLite persistence rewritten: schema v2, backup before migration, bad rows quarantined, per-row upserts, background saves, UI settings persisted. `rusqlite` 0.41 (fixes RUSTSEC-2026-0459).
- Offline render limit raised from 15 s to 5 minutes.
- Trademarked product and song names removed from presets, programs and docs; the PCM programs are synthesized, not sampled ROM.
- `unsafe` is now allowed in exactly two crates (`synthcraft-ffi`, `synthcraft-clap`); the rest of the workspace still forbids it.

### Fixed
- `SubOctave` reference wave had a DC offset.

## [0.2.0] - 2026-11-23

### Added
- Built 3-tier Pure-Rust ArtCraft workspace (`synthcraft-core`, `synthcraft-mcp`, `synthcraft-app`).
- Added 6 hierarchical workbenches (`Modular Rack & VCO/VCF`, `ADSR, LFO & Step Sequencer`, `Oscilloscope & Spectrum`, `CLAP Plugin & SoundCraft`, `DJCraft Stem & WAV Forge`, `Assistant & Help`).
- Added dual VCOs, 4-pole ladder VCF, ADSR + LFO, 16-step acid sequencer, 32-band DFT spectrum analyzer, RIFF `.wav` renderer, CLAP `1.2.2` plugin bridge (`dev.sandraschi.synthcraft`) for SoundCraft, BPM/Camelot stem exporter for DJCraft, and bundled SQLite persistence (`rusqlite` 0.40).
- Added 14 MCP `2026-07-28` tools and 14 CLI subcommands sharing `Command::dispatch`.
