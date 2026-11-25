# Workbench 7: CLAP & SoundCraft Bridge

SynthCraft is not trapped inside a standalone window. The exact same Pure-Rust synthesis engine compiles into a native **CLAP `1.2.2` instrument plugin (`SynthCraft.clap`)** that loads directly inside **SoundCraft**, **REAPER**, and **Bitwig Studio** -- complete with an embedded hardware-styled GPU interface, full MPE / per-note modulation, and shared JSON patch state.

---

## Subpage 7.1: CLAP 1.2.2 Plugin Descriptor

### What It Does (Plain English)
This page shows the live plugin identity, capabilities, and installation status of `SynthCraft.clap` (plugin ID `dev.sandraschi.synthcraft`). It also exports the complete CLAP descriptor JSON so DAWs, test harnesses, and automation scripts can verify every supported extension (Audio Ports, Note Ports, Parameters, State, Embedded Win32 GUI, Polyphonic Modulation, and Note Expressions).

### Controls & Workflow
- **Build & Install**: Run `cargo build --release -p synthcraft-clap && just clap-install` to install `SynthCraft.clap` into `%LOCALAPPDATA%\Programs\Common\CLAP\SynthCraft.clap`.
- **Use in SoundCraft & REAPER**: Rescan your CLAP folder in SoundCraft or REAPER (`Options -> Preferences -> Plug-ins -> CLAP`). Insert `SynthCraft` (`dev.sandraschi.synthcraft`) onto any instrument track -- you get sub-5ms audio playback, full DAW automation, and the embedded `CLAP_EXT_GUI` hardware faceplate.
- **Live Verification**: `SynthCraft.clap` passes `clap-validator` (34 passed, 0 failed) and has been live-tested and bounced directly inside SoundCraft (`-18.44 LUFS`) and opened inside REAPER.

### Provenance, Hardware Lineage & History
- **CLAP -- Clever Audio Plug-in Standard (u-he & Bitwig, 2022)**: Created by Urs Heckmann (u-he, makers of Diva and Zebra) and Alexandre Bique (Bitwig) under the open MIT license as a modern, thread-safe, license-free successor to proprietary Steinberg VST3. Its killer feature is **non-destructive polyphonic modulation**: a DAW can modulate filter cutoff or pitch on *one single held note* in a chord without moving the main knob or affecting the other notes.

### Famous Tracks & Iconic Usage
- **Modern Film Scoring & Electronic Production (Bitwig Studio, REAPER, u-he Diva/Hive)**: CLAP's per-note modulation powers expressive electronic scores and generative ambient works where individual notes inside a chord drift independently in timbre and stereo space.

### CLI & MCP Automation
```powershell
synthcraft-mcp export-clap --json
```

---

## Subpage 7.2: SoundCraft Parameter Map

### What It Does (Plain English)
Every knob you turn inside SynthCraft corresponds to a stable, numbered **Parameter ID** (`0x1001` through `0x1019`) that DAWs like SoundCraft and REAPER use to draw automation lanes and apply per-note modulation. This page lists the full parameter table, showing each parameter's hex ID, display name, range, and whether it supports polyphonic per-note modulation (`CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID`).

### Controls & Workflow
- **Inspect Stable Parameter IDs**: Check exact IDs before writing DAW automation scripts or hardware MIDI mappings.
- **Polyphonic Modulation Flags**: Parameters `0x1001..0x1006` and `0x100A` (oscillator mix, detune, FM, filter cutoff, resonance, LFO depth) support live per-note `CLAP_EVENT_PARAM_MOD` offsets directly from SoundCraft or Bitwig modulators.
- **Zero-Copy Patch Sharing**: Because SoundCraft and SynthCraft share the exact same `SynthPatch` JSON format, any patch you sculpt in standalone SynthCraft loads identically inside your DAW session.

### Provenance, Hardware Lineage & History
- **DAW Automation & Parameter Hashing**: Classic plugin formats often broke saved songs whenever a developer reordered a knob list. By pinning explicit hexadecimal parameter IDs (`0x1001..0x1019`) in `synthcraft-core::params`, your saved REAPER and SoundCraft projects remain permanently recallable even as SynthCraft adds new workbenches and engines.

### Famous Tracks & Iconic Usage
- **Precision Filter Automation in Modern Electronic Music (Deadmau5 -- "Strobe", 2009; Eric Prydz -- "Opus", 2015)**: Iconic 5-minute slow filter-cutoff and resonance climbs rely on sample-accurate DAW parameter automation curves.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-patch-param --param cutoff_hz --value 1850
synthcraft-mcp export-clap --json
```
