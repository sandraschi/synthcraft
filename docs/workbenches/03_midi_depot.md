# Workbench 3: MIDI Keyboard & MIDI Depot

Whether you have a full-size studio keyboard plugged into your computer or you are designing patches on a laptop with just a mouse, this workbench connects your hands and your MIDI library directly to SynthCraft's live polyphonic audio engine.

---

## Subpage 3.1: MIDI Keyboard & 25-Key Piano

### What It Does (Plain English)
This page handles live musical input. It automatically discovers USB and 5-pin DIN MIDI keyboards connected to your PC via Windows MIDI (`midir`), maps your hardware knobs and mod wheel directly to SynthCraft's filter and envelope controls, and gives you a clickable 25-key (2-octave, `C3` to `C5`) studio piano right on screen for instant auditioning.

### Controls & Workflow
- **Connect Hardware**: Plug in any class-compliant USB MIDI controller (Arturia KeyStep/MiniLab, Akai MPK, Korg nanoKEY, Novation Launchkey, Native Instruments Komplete Kontrol). Select it from the **MIDI Input Device** dropdown, or leave it on `Auto-Detect / Omni` to listen on all 16 channels.
- **Automatic Hardware Knob Mapping**: Your controller's standard MIDI Continuous Controllers (CCs) work out of the box:
  - `CC 1` (Mod Wheel) -> LFO Filter Modulation Depth
  - `CC 74` (Brightness) -> 4-Pole Ladder Filter Cutoff Frequency
  - `CC 71` (Timbre) -> Filter Resonance
  - `CC 73` (Attack) -> Envelope Attack Time
  - `CC 91` (FX Depth) -> Extreme DSP Morph Macro
- **On-Screen Piano & Computer Keyboard**: Click or drag across the 25 keys on screen, or arm **Live Rec** in the transport bar to record what you play directly into the 64-step sequencer.

### Provenance, Hardware Lineage & History
- **MIDI 1.0 Specification (Dave Smith & Ikutaro Kakehashi, NAMM 1983)**: In one of music technology's greatest moments of cooperation, Sequential Circuits (US) and Roland (Japan) connected a Prophet-600 to a Roland Jupiter-6 with a single 5-pin cable at the 1983 NAMM show, creating the universal language that every synthesizer and DAW still speaks over 40 years later.

### Famous Tracks & Iconic Usage
- **Herbie Hancock -- "Rockit" (1983)**: One of the earliest mainstream showcases of interconnected digital keyboards and sequenced MIDI-era studio rigs.
- **Jean-Michel Jarre -- "Rendez-Vous" (1986)**: Massive multi-synthesizer live rigs linked together via MIDI controllers and sequencers.

### CLI & MCP Automation
```powershell
synthcraft-mcp record-sequencer-notes --notes "36,36,48,36,43,41,39,36"
```

---

## Subpage 3.2: MIDI Pattern Depot & .MID Export

### What It Does (Plain English)
Blank sequencer grids can be intimidating when you just want to test how a new bass, piano, or marimba patch grooves. The MIDI Pattern Depot bundles 8 genre-authentic musical patterns that you can load into SynthCraft's sequencer in one click -- or export as standard `.mid` files (480 pulses per quarter note) to drag straight into REAPER, SoundCraft, Ableton Live, or FL Studio.

### Controls & Workflow
- **Browse & Load Patterns**: Pick from 8 built-in patterns:
  1. *1990 Ibiza Rave Piano* (syncopated house chords)
  2. *Club Organ 2 Bass* (bouncy UK garage / house bassline)
  3. *Vienna 303 Acid* (accented, sliding 16th-note acid squelch)
  4. *Universe Pad* (wide atmospheric progression)
  5. *Wavefolder Polyrhythm* (West-Coast percussive ostinato)
  6. *Karplus Harp* (flowing arpeggiated plucked line)
  7. *Modal Marimba* (mallet percussion pattern)
  8. *Neuro Reese* (sustained pitch-gliding sub bass)
- **Export & Import Standard MIDI Files (`.mid`)**: Save any pattern to disk as a Format 0/1 `.mid` file, or import any `.mid` melody from your hard drive directly into SynthCraft's sequencer (preserving notes, velocities, accents, legato slides, and BPM).

### Provenance, Hardware Lineage & History
- **Standard MIDI File Format (Dave Oppenheim / Opcode Systems, 1988)**: Created so musicians could trade musical scores and grooves between different sequencers and operating systems without losing timing, velocity, or tempo metadata.

### Famous Tracks & Iconic Usage
- **Phuture -- "Acid Tracks" (1987)**: The hypnotic, looping 16-step pattern with accents and slides that inspired the *Vienna 303 Acid* depot clip.
- **Robin S. -- "Show Me Love" (1993)** & **Crystal Waters -- "Gypsy Woman" (1991)**: The classic bouncy house piano and organ grooves echoed in the *Ibiza Rave Piano* and *Club Organ 2 Bass* patterns.

### CLI & MCP Automation
```powershell
synthcraft-mcp load-midi-clip --id vienna_303_acid
synthcraft-mcp export-midi --clip-id ibiza_rave_piano --path ibiza_piano.mid
synthcraft-mcp import-midi-file --path my_melody.mid
```
