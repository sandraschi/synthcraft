# Changelog

## [Unreleased]

## [0.3.0] - 2026-11-25

### Added
- Multi-row scrollable 3U Eurorack skiff (`rack_canvas.rs`, `rack.rs`) with categorized Module Browser (`+ Add Module`), right-click move/bypass/remove, and 6 new playable DSP modules: Wavefolder + LPG, Sample & Hold + Slew, Stereo Ping-Pong Delay, Euclidean Clock, Dual Attenuverter, and Inline 3U Oscilloscope.
- 12 curated 1-click pre-patched modular Rack Rigs (`rack_presets.rs`) with connected audio/CV cables and matched voice tuning (UI dropdown + CLI + MCP).
- Always-visible `< [Patch] >` selector in both `synthcraft-app` and `SynthCraft.clap`.
- 35 MCP `2026-07-28` stdio tools and 38 CLI subcommands (including 12 rack module, cable, and rig preset commands).
- Native `CLAP_EXT_GUI` embedded hardware UI in `SynthCraft.clap` (Windows `CLAP_WINDOW_API_WIN32` + `egui-wgpu`): 25 rotary knobs with DAW automation gestures, live stereo peak meter and oscilloscope, patch/VCO1/VCF selectors, sample bank picker, and audition keyboard.
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
- Sampler layers (a note plays every matching zone, up to 4) and one-shot zones (SFZ `loop_mode=one_shot`, ignores note-off); SFZ `pan`.
- True stereo engine output: `SynthRt::process_stereo`, per-voice spread (`stereo_spread`), stereo chorus, decorrelated stereo reverb. CLAP, C ABI, Rack and the desktop audio callback now output real stereo.
- Ambient mode: slow seeded drift on cutoff, morph and reverb send (`ambient_depth`, `ambient_rate_hz`, `ambient_seed`). Three new host parameters (25 in total, ids 0x1017 to 0x1019).
- Interactive 3U Eurorack bay: all 12 module knobs are draggable (vertical drag, Shift fine adjust, scroll wheel, double-click reset), module mode labels click-to-cycle, and patch cables can be dragged jack-to-jack, moved, unplugged, or reset via `Command` (`ListRackCables`, `ConnectRackCable`, `DisconnectRackCable`, `ResetRackCables`).
- Live rack cable DSP routing (`RackRouting` in `SynthRt`): patching and unplugging cables in the 3U bay alters live audio and CV modulation routing (defaults preserve existing patch sound).
- Native `CLAP_EXT_GUI` plug-in UI in `synthcraft-clap` (Windows `CLAP_WINDOW_API_WIN32` embedded child `HWND` + `egui-wgpu`): 25 hardware rotary knobs with DAW automation gestures (`PARAM_GESTURE_BEGIN`/`VALUE`/`END`), live stereo peak meter and oscilloscope, VCO1/VCF selectors, sample bank picker (`Documents/SynthCraft/Banks`), and 1-octave audition keyboard.

### Changed
- SQLite persistence rewritten: schema v2, backup before migration, bad rows quarantined, per-row upserts, background saves, UI settings persisted. `rusqlite` 0.41 (fixes RUSTSEC-2026-0459).
- Offline render limit raised from 15 s to 5 minutes.
- Trademarked product and song names removed from presets, programs and docs; the PCM programs are synthesized, not sampled ROM.
- `unsafe` is now allowed in exactly two crates (`synthcraft-ffi`, `synthcraft-clap`); the rest of the workspace still forbids it.

### Fixed
- `SubOctave` reference wave had a DC offset.
- Pitch bend was applied twice to sampler playback.
- Default stereo spread of 0.3 slightly changes the sound of existing patches (voices are now panned); set `stereo_spread` to 0 for the old mono image.

## [0.2.0] - 2026-11-23

### Added
- Built 3-tier Pure-Rust ArtCraft workspace (`synthcraft-core`, `synthcraft-mcp`, `synthcraft-app`).
- Added 6 hierarchical workbenches (`Modular Rack & VCO/VCF`, `ADSR, LFO & Step Sequencer`, `Oscilloscope & Spectrum`, `CLAP Plugin & SoundCraft`, `DJCraft Stem & WAV Forge`, `Assistant & Help`).
- Added dual VCOs, 4-pole ladder VCF, ADSR + LFO, 16-step acid sequencer, 32-band DFT spectrum analyzer, RIFF `.wav` renderer, CLAP `1.2.2` plugin bridge (`dev.sandraschi.synthcraft`) for SoundCraft, BPM/Camelot stem exporter for DJCraft, and bundled SQLite persistence (`rusqlite` 0.40).
- Added 14 MCP `2026-07-28` tools and 14 CLI subcommands sharing `Command::dispatch`.
