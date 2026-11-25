# SynthCraft 0.4.0 (2026-11-25)

## Highlights
- **64-Step (4-Bar) Acid Sequencer & 8-Voice TR-808/909 Drum Machine**: Expandable `16/32/48/64`-step sequencer with Bar A/B/C/D pager + allocation-free 8-voice analog/FM percussion synth (`BD`, `SD`, `CP`, `CH`/`OH` with choke, `LT`, `CB`, `CY`, swing, Euclidean fills, 6 club kits).
- **Next-Gen Lab (Workbench 11)**: Drawable 16-point MSEG splines + Dual Morphing Filter B (Formant A-E-I-O-U, Comb +/-, MS-20 Screamer) with live animated knob mod rings, 24-grain live-capture Granular Cloud, 64-Partial Spectral Additive Sculptor, and Multi-Core Evolutionary Genopatch Audio-to-Patch matcher.
- **Pro Lab (Workbench 12)**: 6-Operator Thru-Zero FM Routing Matrix (6x6), 16-Voice Hyper-Unison stack, Reorderable Studio FX Rack (OTT 3-band compressor, Convolution IR reverb, Shimmer pitch reverb, Analog Tape & Bitcrusher), Goliath 8x8 2D FDTD Physical Modeling Mesh, and 8x Oversampled WDF Circuits (Moog CP3, Buchla 292 Vactrol LPG, TB-303 diode ladder).
- **Visuals, MPE Poly-Mod & Neural DDSP (Workbench 13)**: Full per-note MPE & CLAP `1.2.2` Poly-Mod (`CLAP_EVENT_NOTE_EXPRESSION` + `CLAP_EVENT_PARAM_MOD` in `SynthCraft.clap`), 24-slice 3D perspective STFT Waterfall Spectrogram, Lissajous Stereo Vectorscope + phase correlation meter, and Neural DDSP Timbre Transfer (32-harmonic + filtered-noise decoder).
- **Performance & Mastering Suite (Workbench 14)**: 1-Finger Polyphonic Chord Memory (10 voicings) + Scale Lock + Riff Generator, 16-Band Analog Vocoder (robotic speech or live `DJFX` vocals), 8-Scene Live Crossfader & 4-Corner XY Morph Pad, and Studio Mastering Chain (4-Band EQ, Sub-Bass Mono Maker `<120 Hz`, M/S Stereo Width, True-Peak Brickwall Limiter).
- **3U Eurorack CLAP 1.2.2 Host, WebAssembly AudioWorklet, Live DJCraft Bridge & AI2 Combi**: 13-module 3U Eurorack skiff, ~190 KB `synthcraft_wasm.wasm`, live `DJLink`/`DJPD`/`DJFX` bridge, and 8 CC0 workstation banks with 1988 32 kHz/12-bit DAC.
- **Live Host Verification**: `SynthCraft.clap` scanned, instantiated, and bounced (`-18.44 LUFS`) in **SoundCraft** and opened in **REAPER**; native `SynthCraft 2.0.0` C++/FFI `plugin.dll` compiled against Rack SDK 2.6.6 and loaded live in **VCV Rack Free 2.6.6**.
- **60 MCP `2026-07-28` Tools & CLI Subcommands**: Full CLI and sandboxed MCP coverage across every synthesis, drum, FX, mastering, and host-bridge engine.

## Verification
- `[craft-floor] ok synthcraft`
- `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings across all 7 workspace crates
- `cargo test --workspace`: 104 passed, 0 failed (including headless `egui_kittest` across all `14` workbenches / `37` subpages and `60` MCP tools)
