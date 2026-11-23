## SynthCraft Unreleased (after v0.2.0)

### Added
- Real-time audio (cpal) and MIDI input (midir) through the single `SynthRt` engine shared with offline rendering.
- Sampler with WAV, SFZ and SF2 loading, Hermite interpolation, loop points and automatic sustain loops; `AtmoPad` pad oscillator, chorus and reverb; ambient factory patches.
- `fetch-bank` / `list-banks` and an in-app downloader for checksum-verified CC0 sample packs (VSCO 2 CE strings), manifest in `banks/manifest.json`.
- Agent file sandbox: MCP requires `--automation-read-root` and `--automation-write-root`.
- 23 MCP tools and 26 CLI subcommands.
- `export_djcraft_stem` can render the mono loop to a WAV; manifest uses the engine sample rate and bar-grid cue points.

### Changed
- SQLite persistence rewritten: schema v2, backup before migration, bad rows quarantined, per-row upserts, background saves, UI settings persisted. `rusqlite` 0.41 (fixes RUSTSEC-2026-0459).
- Offline render limit raised from 15 s to 5 minutes.
- Trademarked product and song names removed from presets, programs and docs; the PCM programs are synthesized, not sampled ROM.
- CLAP is documented honestly: descriptor and parameter map only, no loadable plugin yet.

### Fixed
- `SubOctave` reference wave had a DC offset.

## SynthCraft v0.2.0 (2026-11-23)

- 6 integrated workbenches: Modular VCO/VCF Rack, 16-Step Acid Sequencer & ADSR/LFO, Real-Time Oscilloscope & 32-Band Spectrum Analyzer, CLAP 1.2.2 Plugin Bridge for SoundCraft, BPM/Camelot WAV Stem Forge for DJCraft, and 6-Tab Help.
- Bundled SQLite persistence (`%LOCALAPPDATA%\synthcraft\synthcraft.db`) and 14 MCP `2026-07-28` stdio tools.
