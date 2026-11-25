# Changelog

## [Unreleased]

## [0.5.0] - 2026-11-25

### Added
- **Cathedral Pipe Organ Console & GrandOrgue Bach MIDI Depot (Workbench 15)** (`synthcraft-core::pipe_organ`, `synthcraft-app::pipe_organ_ui`): 16-stop *Werkprinzip* pipe organ engine (`Hauptwerk`, `Schwellwerk`, `Pedalwerk`), 8 combination thumb pistons, couplers (`SW->HW`, `HW->PED`, `SW->PED`, `Sub 16'`, `Super 4'`), Swell shutters, Crescendo Walze, Wind-Chest Chiff, Bellows Wind Sag, Tremulant, 5 historical organ temperaments (`Werckmeister III`, `Silbermann Meantone`, `Kirnberger III`, `Quarter-Comma Meantone`, `12-TET`), and direct `.mid` scanner/importer for `grandorgue-mcp/midi_depot`.
- **129-Preset Pro Factory Bank & Serum/Pigments Preset Browser** (`synthcraft-core::preset_bank`, `synthcraft-app::preset_browser_ui`): 129 audio-verified factory presets across 12 categories (including 10 Cathedral Pipe Organ presets), live search, category counts, 18 engine badges, star favorites, and musical `Mutate (+/- 15%)`.
- **Drag-and-Drop Modulation, Visual Wavetable Editor & OS `.wav` Drop** (`synthcraft-app::mod_wavetable_ui`, `synthcraft-core::{granular, fx_rack}`): drag-to-modulate source pills, 16-harmonic wavetable sculptor, and OS `.wav` drag-and-drop into Genopatch, Convolution IR Reverb, and Granular Cloud.
- **Golden-Ear Analog Calibration & 4-Wide SIMD** (`synthcraft-core::analog_simd`): per-voice `VintageSlopParams` component tolerance & thermal drift, ZDF bass-preserving resonance compensation, compact Neural DDSP harmonic MLP, and 4-wide SIMD stereo voice mixer.
- **39-Subpage Documentation Stack & Interactive HTML Handbook** (`docs/workbenches/01..15`, `docs/site/index.html`, `synthcraft-app::help_docs`): dedicated human-readable sub-READMEs with hardware provenance and famous tracks, contextual `? Guide` drawer + searchable `F1` Encyclopedia, and self-contained interactive HTML handbook.

## [0.4.0] - 2026-11-25

### Added
- **3U Eurorack CLAP 1.2.2 Host Module** (`synthcraft-clap-host`, `synthcraft-core::clap_host`): dynamically loads external `.clap` shared libraries inside a 16 HP 3U rack slot with dry/wet mix, CV depth, and live `CLAP_EVENT_PARAM_MOD` modulation from `LFO / ADSR` or `S&H + SLEW` patched into `PARAM CV`. Off-audio-thread plugin swap and drop via `AudioHandle::reclaim`.
- **WebAssembly AudioWorklet Engine** (`synthcraft-wasm`): gated `rusqlite` behind `default = ["sqlite"]` in `synthcraft-core`, added bundler-free WASM linear-memory C ABI (`synthcraft_wasm.wasm`, ~190 KB release) + drop-in `web/synthcraft-worklet.js` `AudioWorkletProcessor`.
- **Morphing Wavetable Oscillator** (`synthcraft-core::wavetable`): `WaveformKind::Wavetable` with 4 built-in morphing tables (`analog_morph`, `formant_vowel`, `metallic_bell`, `acid_fold`) and 16-bit PCM / 32-bit float `.wav` wavetable loader (up to 64 frames x 256 samples).
- **General Modulation Matrix** (`synthcraft-core::mod_matrix`): 8 bipolar routes mapping 6 sources (`LFO`, `ADSR Env`, `Velocity`, `Mod Wheel`, `Sample & Hold`, `Ambient Drift`) to 9 destinations (`Cutoff`, `Resonance`, `FM Index`, `Pulse Width`, `VCO2 Detune`, `Wavefold`, `Stereo Pan`, `Extreme Morph`, `Reverb Mix`).
- **Real-Time Arpeggiator** (`synthcraft-core::arp`): `Off`, `Up`, `Down`, `Up/Down`, `Random` modes with rate division, 1-4 octaves, gate length, and latch.
- **Scala `.scl` Microtonal Tuning** (`synthcraft-core::tuning`): 7 built-in scales (`12-TET`, `5-Limit Just`, `Pythagorean`, `19-TET`, `24-TET`, `31-TET`, `Bohlen-Pierce`) plus full `.scl` parser.
- **Standard MIDI File (`.mid`) Importer** (`synthcraft-core::smf_import`): parses Format 0/1 `.mid` files into the 16-step sequencer with pitch, velocity, accent, legato slide, and tempo BPM.
- **Live Sequencer Record**: `Rec` toggle in the desktop transport bar overdubs live playing (keyboard, on-screen piano, or USB MIDI) when running, or step-records & advances when stopped, with full Undo/Redo support.
- **Undo/Redo & A/B Compare**: 64-deep Undo/Redo (`Ctrl+Z`/`Ctrl+Y`), A/B patch compare, and 3U rack `WidgetInfo` accessibility metadata.
- **VCV Rack 2 Context Menus** (`rack/src/SynthCraft.cpp`): right-click submenus for `VCO 1 waveform`, `VCO 2 waveform`, `Filter mode`, and `Wavetable` via `sc_state_json` (zero C ABI break).
- **64-Step (4-Bar) Acid Sequencer & 8-Voice TR-808/909 Drum Machine** (`synthcraft-core::drums`, `synthcraft-app::drums_ui`): expandable `16/32/48/64`-step sequencer with Bar A/B/C/D pager + allocation-free 8-voice analog/FM percussion synth (`BD`, `SD`, `CP`, `CH`/`OH` with choke, `LT`, `CB`, `CY`, swing, Euclidean fills, 6 club kits).
- **Next-Gen Lab (Workbench 11)** (`synthcraft-core::nextgen`, `granular`, `genopatch`): drawable 16-point MSEG splines + Dual Morphing Filter B (Formant A-E-I-O-U, Comb +/-, MS-20 Screamer) with live animated knob mod rings, 24-grain live-capture Granular Cloud, 64-Partial Spectral Additive Sculptor, and Multi-Core Evolutionary Genopatch Audio-to-Patch matcher.
- **Pro Lab (Workbench 12)** (`synthcraft-core::fm_unison`, `fx_rack`, `physical`): 6-Operator Thru-Zero FM Routing Matrix (6x6), 16-Voice Hyper-Unison stack, Reorderable Studio FX Rack (OTT 3-band compressor, Convolution IR reverb, Shimmer pitch reverb, Analog Tape & Bitcrusher), Goliath 8x8 2D FDTD Physical Modeling Mesh, and 8x Oversampled WDF Circuits (Moog CP3, Buchla 292 Vactrol LPG, TB-303 diode ladder).
- **Visuals, MPE Poly-Mod & Neural DDSP (Workbench 13)** (`synthcraft-core::neural_ddsp`, `synthcraft-app::visualizers_ui`, `synthcraft-clap`): full per-note MPE & CLAP `1.2.2` Poly-Mod (`CLAP_EVENT_NOTE_EXPRESSION` + `CLAP_EVENT_PARAM_MOD`), 24-slice 3D perspective STFT Waterfall Spectrogram, Lissajous Stereo Vectorscope + phase correlation meter, and Neural DDSP Timbre Transfer.
- **Performance & Mastering Suite (Workbench 14)** (`synthcraft-core::chord_mem`, `vocoder`, `scenes`, `mastering`, `synthcraft-app::perf_master_ui`): 1-Finger Polyphonic Chord Memory (10 voicings) + Scale Lock + Riff Generator, 16-Band Analog Vocoder, 8-Scene Live Crossfader & 4-Corner XY Morph Pad, and Studio Mastering Chain (4-Band EQ, Sub-Bass Mono Maker `<120 Hz`, M/S Stereo Width, True-Peak Brickwall Limiter).
- **Live Host Verification**: `SynthCraft.clap` scanned, instantiated, and bounced (`-18.44 LUFS`) in **SoundCraft** and opened in **REAPER**; native `SynthCraft 2.0.0` C++/FFI `plugin.dll` compiled against Rack SDK 2.6.6 and loaded live in **VCV Rack Free 2.6.6**.
- **60 MCP `2026-07-28` Tools & CLI Subcommands** (`synthcraft-mcp`): added `set_mod_route`, `clear_mod_matrix`, `set_arpeggiator`, `set_tuning`, `load_wavetable`, `list_wavetables`, `import_midi_file`, `record_sequencer_notes`, `scan_clap_plugins`, `inspect_clap_plugin`, `configure_clap_host`, `get_clap_host_status`, `export_wasm_bundle`, `set_sequencer_length`, `load_drum_preset`, `set_mseg_filter2`, `set_granular_spectral`, `run_genopatch`, `set_fm6_unison`, `set_studio_fx`, `set_physical_wdf`, `set_ddsp_timbre`, and `set_perf_master`.

### Fixed
- `mcpb/manifest.json` now uses `"__VERSION__"` placeholder for release packaging.
- `synthcraft-mcp` launches `real_main()` on an 8 MiB stack thread to prevent Windows debug-build stack overflows (`0xC00000FD`) in `clap::Parser`.

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
