# SynthCraft 0.4.0 (2026-11-25)

## Highlights
- **3U Eurorack CLAP 1.2.2 Host Module (`synthcraft-clap-host`)**: Host external `.clap` plugins inside a 16 HP 3U rack slot with dry/wet mix, bipolar CV depth, and live `CLAP_EVENT_PARAM_MOD` modulation from `LFO / ADSR` or `S&H + SLEW` patched into `PARAM CV`.
- **WebAssembly AudioWorklet Engine (`synthcraft-wasm`)**: Compiles the full `synthcraft-core` DSP engine to ~190 KB of standalone `wasm32-unknown-unknown` WebAssembly (`synthcraft_wasm.wasm`) with a bundler-free `synthcraft-worklet.js` AudioWorklet bridge.
- **Morphing Wavetable Oscillator**: `WaveformKind::Wavetable` with 4 built-in morphing tables (`analog_morph`, `formant_vowel`, `metallic_bell`, `acid_fold`) and a 16-bit PCM / 32-bit float `.wav` wavetable loader.
- **General Modulation Matrix, Arpeggiator & Scala `.scl` Tuning**: 8 bipolar mod routes (6 sources -> 9 destinations), real-time Arpeggiator (`Off`, `Up`, `Down`, `Up/Down`, `Random`, latch), and 7 built-in microtonal scales + custom `.scl` parser.
- **Live Sequencer Record & SMF `.mid` Import**: Overdub live playing (keyboard or USB MIDI) into the 16-step sequencer or step-record when stopped; import Format 0/1 `.mid` files directly into the sequencer.
- **Undo/Redo, A/B Compare & Accessibility**: 64-deep Undo/Redo (`Ctrl+Z`/`Ctrl+Y`), A/B patch compare, and 3U rack `WidgetInfo` a11y metadata.
- **VCV Rack 2 Context Menus**: Right-click `VCO 1 waveform`, `VCO 2 waveform`, `Filter mode`, and `Wavetable` submenus in `rack/src/SynthCraft.cpp` (zero C ABI break).
- **48 MCP `2026-07-28` Tools & CLI Subcommands**: Full CLI and sandboxed MCP coverage for CLAP hosting, wavetables, mod matrix, arpeggiator, Scala tuning, MIDI import, step recording, and WASM bundle export.

## Verification
- `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings across all 7 workspace crates
- `cargo test --workspace`: 108/108 passed (including live `.clap` hosting against `SynthCraft.clap` and `wasm32-unknown-unknown` release build)
