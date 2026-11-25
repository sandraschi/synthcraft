# SynthCraft v0.4.0 -- Help & User Guide

## Get Started

Welcome to **SynthCraft**, a Pure-Rust modular synthesizer studio, 64-step acid sequencer, 8-voice TR-808/909 drum machine, 6-operator FM / granular / spectral / neural DDSP workstation, 16-band analog vocoder, mastering desk, native **CLAP `1.2.2` DAW plugin (`SynthCraft.clap`)**, native **VCV Rack 2 module (`SynthCraft 2.0.0`)**, and **DJCraft** live bridge.

Every single subpage in the left sidebar has its own dedicated human-readable guide -- complete with plain-English explanations, hardware history, famous tracks, and CLI/MCP commands:
- Click **`? Guide`** in the top studio bar (or press **`F1`**) from anywhere in the app to open the guide for the exact subpage you are using.
- Open **`docs/site/index.html`** in any browser for the searchable, interactive HTML studio handbook.

## Functions

SynthCraft bundles **14 hierarchical workbenches (37 subpages)**:
1. **VCV 2 Eurorack & VCO/VCF** -- Draggable 3U modular patch cables, 13 playable modules (including the external `CLAP HOST` slot), 12 pre-patched Rack Rigs, dual oscillators, 4-pole ladder filter, and Factory Patch Bank.
2. **1990-Style PCM & Extreme DSP** -- Classic 1990s workstation pianos, organ bass, choirs, and orchestra hits (with 1988 32 kHz/12-bit DAC crunch) + 6 mathematical Extreme DSP generators (Karplus-Strong plucked strings, West-Coast wavefolder, Phase Distortion, Modal struck bar).
3. **MIDI Keyboard & MIDI Depot** -- Live USB MIDI (`midir`), 25-key studio piano, 8 genre-authentic built-in MIDI patterns, and `.mid` file import/export.
4. **Demo Showcase & Audition** -- 1-click club, acid, house piano, and West-Coast presets with instant spectral feedback.
5. **Sequencer, Drums & ADSR/LFO** -- 64-step (4-bar) acid sequencer with Accent/Slide & Live Record, 8-voice pure-Rust TR-808/909 drum machine (6 kits, Euclidean fills, hi-hat choke, swing), ADSR/LFO, 8-route Mod Matrix, Arpeggiator, and Scala `.scl` microtonal tuning.
6. **Oscilloscope & Spectrum** -- Triggered waveform oscilloscope + 32-band DFT spectrum analyzer with RMS/Peak dBFS and pitch tracking.
7. **CLAP & SoundCraft Bridge** -- Native `SynthCraft.clap` (`dev.sandraschi.synthcraft`) with embedded hardware UI, MPE & per-note Poly-Mod, live-verified in SoundCraft and REAPER.
8. **DJCraft Stem & WAV Forge** -- Live `DJLink` tempo sync, `DJPD` pads, `DJFX` stereo vocal/deck routing, Camelot harmonic keys, and 48 kHz `.wav` stem rendering.
9. **Assistant & Help** -- Local Patch Co-Designer Chat and searchable 37-Subpage Encyclopedia (`F1`).
10. **Studio Settings & Prefs (`Ctrl+,`)** -- Audio driver/oversampling, MIDI/MPE, 5 hardware skins & 4 layouts, PCM DAC settings, plugin paths, and SQLite DB status.
11. **Next-Gen Lab** -- Drawable 16-point MSEG splines + Dual Morph Filter B (Formant, Comb +/-, MS-20 Screamer) with animated knob mod rings, 24-grain live-capture Granular Cloud, 64-Partial Spectral Additive Sculptor, and Multi-Core Evolutionary Genopatch Audio-to-Patch matcher.
12. **Pro Lab** -- 6-Operator Thru-Zero FM Matrix (6x6), 16-Voice Hyper-Unison, Reorderable Studio FX Rack (OTT 3-band, Convolution IR, Shimmer, Analog Tape), 8x8 2D FDTD Physical Mesh, and 8x Oversampled WDF Analog Circuits (Moog CP3, Buchla 292 LPG, TB-303).
13. **Visuals, MPE & Neural DDSP** -- 24-slice 3D perspective Waterfall Spectrogram, Lissajous Stereo Vectorscope + phase meter, 2D MPE Expression Touch Pad, and Neural DDSP Timbre Transfer (Violin, Flute, Sax, Cello, Trumpet, Choir).
14. **Performance & Mastering Suite** -- 1-Finger Chord Memory (10 voicings) + Scale Lock + Riff Generator, 16-Band Analog Vocoder, 8-Scene Crossfader & 4-Corner XY Morph Pad, and Studio Mastering Bus (4-Band EQ, Sub-Bass Mono `<120 Hz`, M/S Width, Brickwall Limiter).

All patches auto-save off the UI thread to `%LOCALAPPDATA%\synthcraft\synthcraft.db` with automatic pre-migration backups.

## Usage

### CLI (`synthcraft-mcp`)
```powershell
synthcraft-mcp status --json
synthcraft-mcp list-patches
synthcraft-mcp set-sequencer-length --steps 64
synthcraft-mcp load-drum-preset --preset berlin_techno --enabled true
synthcraft-mcp run-genopatch --target-preset acid_squelch --apply true
synthcraft-mcp set-perf-master --chord-enabled true --chord-mode DubMinor9 --master-enabled true
synthcraft-mcp render-wav --path loop_128bpm_8A.wav --duration-ms 2000
```

### MCP Server (`2026-07-28` stdio, 60 Tools)
```powershell
synthcraft-mcp --mcp --automation-read-root D:\AudioIn --automation-write-root D:\AudioOut
```
Both `--automation-read-root` and `--automation-write-root` are required so external agents stay strictly inside approved folders.

## Tips

- **`F1`** -- Open the 37-Subpage Encyclopedia pre-selected to your current subpage.
- **`? Guide`** (top bar) -- Toggle a live side-panel guide right beside your knobs.
- **`Ctrl+K`** -- Command Palette; **`Ctrl+,`** -- Studio Settings; **`Ctrl+Z` / `Ctrl+Y`** -- 64-deep Undo/Redo; **`Ctrl+MouseWheel`** -- Smooth UI Zoom.
- **`Shift + Drag`** on any rotary knob for fine precision; **Double-click** to reset to default.
- **DAW & Rack 2**: `SynthCraft.clap` loads natively in SoundCraft and REAPER; `SynthCraft 2.0.0` loads natively in VCV Rack 2.

## Other Crafts

SynthCraft is part of the `sandraschi` Pure-Rust ArtCraft ecosystem alongside `djcraft`, `soundcraft`, `musiccraft`, `logiccraft`, `packetcraft`, `graphcraft`, `robotcraft`, and `smbcraft`.
