# Workbench 8: DJCraft Stem & WAV Forge

Designed for live DJs and electronic performers, this workbench bridges studio sound design directly to the club booth. Lock your synthesizer patches to exact deck tempos and harmonic Camelot keys, receive live transport and pad triggers from **DJCraft**, and render beatgrid-accurate `.wav` loops, risers, and sub-drops that drop straight onto a DJ deck with zero drift.

---

## Subpage 8.1: BPM & Camelot Loop Forge

### What It Does (Plain English)
When a DJ mixes a live synth loop or transition riser over a playing track, two things must match perfectly: **tempo (BPM)** and **musical key**. This page locks SynthCraft to your DJ set's exact tempo and **Camelot Wheel key** (like `8A` for A Minor or `4A` for F Minor), and connects directly to **DJCraft** over three real-time bridges:
- **DJLink**: Synchronizes SynthCraft's sequencer and drum machine to DJCraft's live master deck BPM and beat phase.
- **DJPD**: Receives live performance-pad triggers from DJCraft directly into SynthCraft's voices and drum kit.
- **DJFX**: Routes live stereo audio (like vocals or a playing deck) from DJCraft into SynthCraft's 16-Band Vocoder, Granular Cloud, or Studio FX Rack.

### Controls & Workflow
- **Set Camelot Key & Tempo**: Pick your target Camelot key (`1A` through `12B`) and BPM (`60` to `200`).
- **Bar-Grid Cue Points**: Automatically embeds DJ-ready cue markers (`LOOP_IN`, `FILTER_PEAK`, `DROP_OUT`) into the exported stem manifest so DJCraft knows the exact downbeat and climax of your riser or loop.

### Provenance, Hardware Lineage & History
- **The Camelot Wheel (Mark Davis, Mixed In Key, 1990s)**: Translates the classical Circle of Fifths into a simple clock-face numbering system (`1A-12A` minor, `1B-12B` major). Any adjacent number (`8A -> 9A` or `8A -> 8B`) mixes harmonically without clashing notes -- a revolution for club DJs and live hybrid electronic acts.
- **Ableton Link & Pioneer Pro DJ Link (2010s)**: Brought sub-millisecond tempo and phase synchronization between DJ decks, drum machines, and modular synthesizers on stage.

### Famous Tracks & Iconic Usage
- **Richie Hawtin (Plastikman) & Carl Cox Live Hybrid Sets**: Pioneered locking live TR-909 / TB-303 and modular synth rigs directly to CDJ/software DJ decks so live improvised acid lines and risers lock seamlessly onto playing club records.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-camelot-key --key 8A
synthcraft-mcp export-djcraft-stem --bars 4 --json
```

---

## Subpage 8.2: RIFF WAV & Stem Exporter

### What It Does (Plain English)
Turns whatever you have sculpted in SynthCraft -- a 4-bar acid bassline, a 64-step 808/909 drum beat, an 8-bar noise riser, or a single sustained pad chord -- into a crystal-clear, uncompressed **16-bit or 24-bit 48 kHz RIFF PCM `.wav` file** on disk. Because SynthCraft renders offline using the exact same `SynthRt` DSP engine as live playback, what you hear is 100% what gets written to disk.

### Controls & Workflow
- **Choose Length & Target**: Set exact duration in milliseconds or musical bars, choose an output path (sandbox-verified in MCP mode), and click **Forge & Export `.wav`**.
- **1-Click Forge & Load**: Render a stem and immediately pass its path and beatgrid manifest to DJCraft for live deck playback.

### Provenance, Hardware Lineage & History
- **RIFF WAVE Audio Format (Microsoft & IBM, 1991)**: The universal uncompressed studio audio container. Zero lossy compression artifacts, sample-accurate loop boundaries, and instant compatibility with every DAW, sampler, and DJ player on Earth.

### Famous Tracks & Iconic Usage
- **Modern Stem DJing & Live Sampling (Daft Punk *Alive 2007*, Justice, Fred again..)**: Built around pre-forged, tempo-locked vocal, synth, and percussion stems triggered and mangled live on stage.

### CLI & MCP Automation
```powershell
synthcraft-mcp render-wav --path loop_128bpm_8A.wav --duration-ms 2000
```
