# Workbench 1: VCV 2 Eurorack & VCO/VCF

Welcome to the heart of SynthCraft: a tactile, cable-patched 3U modular synthesizer rack paired with a classic dual-oscillator analog voice and resonant ladder filter. Think of this workbench as having both a wall of patchable studio hardware and a complete library of instant-recall presets at your fingertips.

---

## Subpage 1.1: VCV 3U Eurorack & Patch Cables

### What It Does (Plain English)
Instead of locking you into one fixed signal path, the 3U Eurorack bay lets you wire individual sound modules together with virtual patch cables that drape across the panel with realistic gravity. Sound starts at an oscillator (a tone generator), gets sculpted by a filter (which brightens or darkens the tone), and can be folded, delayed, modulated, or even routed through an external third-party CLAP plugin living right inside the rack.

### Controls & Workflow
- **Add, Move, Bypass, or Remove Modules**: Click `+ Add Module` to browse all 13 playable 3U modules -- Dual Oscillator, 4-Pole Ladder Filter, Envelope Generator, Low-Frequency Oscillator, Wavefolder + Low-Pass Gate, Sample & Hold + Slew Limiter, Stereo Ping-Pong Delay, Euclidean Clock Generator, Dual Attenuverter, Inline 3U Oscilloscope, and the 16 HP **CLAP Host** slot for loading external `.clap` plugins. Right-click any module faceplate to move, bypass, or remove it.
- **Drag Patch Cables**: Click and drag from any output jack to any input jack. Moving a cable or unplugging it updates the audio routing live without audio dropouts.
- **12 Pre-Patched Rack Rigs**: Not sure where to plug your first cable? Pick one of the 12 curated Rack Rigs from the dropdown (such as West-Coast Buchla Pluck, Acid Squelch, or Dub Ping-Pong) to wire up an entire modular patch in one click.
- **Dual Oscillators & 4-Pole Ladder Filter**: Shape the core voice using 10 waveforms (including classic analog saw, square, pulse-width modulation, supersaw, and morphing wavetables), fine detuning in cents, linear frequency modulation between oscillators, and a 4-mode resonant filter (24 dB/oct Low-Pass, 12 dB/oct Band-Pass, 12 dB/oct High-Pass, and Acid Resonant).
- **Knob Gestures**: Drag vertically to adjust, hold `Shift` for fine precision, scroll the mouse wheel, or double-click any knob to reset it to its default value.

### Provenance, Hardware Lineage & History
- **3U Eurorack Format (Doepfer A-100, 1995)**: Dieter Doepfer standardized the 3U (128.5 mm tall) horizontal-pitch (`HP`) modular rack standard in Munich in 1995, democratizing modular synthesis that previously cost as much as a luxury car.
- **24 dB/Octave Transistor Ladder Filter (Moog Minimoog Model D, 1970)**: Robert Moog's patented four-stage ladder filter gave analog synthesizers their unmistakable warm, creamy roll-off and singing self-oscillation at high resonance.
- **Wavefolder & Low-Pass Gate (Buchla 259 & 292, 1970s)**: On the US West Coast, Don Buchla took the opposite approach from East Coast subtractive filtering: start with a pure sine wave, fold it back on itself to create rich metallic harmonics, and gate it through a light-sensitive vactrol ("Low-Pass Gate") that decays like a struck wooden marimba or bongo.
- **Sample & Hold + Slew (ARP 2600, 1971)**: Classic stepped random voltage modulation smoothed into gliding melodic lines.

### Famous Tracks & Iconic Usage
- **Donna Summer -- "I Feel Love" (1977, produced by Giorgio Moroder)**: Driven by a Moog Modular sequenced bassline and resonant ladder filter sweeps that invented modern electronic dance music.
- **Kraftwerk -- "Autobahn" (1974)**: Showcased the Minimoog's singing ladder filter lead and modulated oscillator sweeps.
- **Morton Subotnick -- "Silver Apples of the Moon" (1967)**: The landmark album recorded entirely on Don Buchla's West-Coast modular system, demonstrating organic vactrol percussion and folded timbres.
- **The Who -- "Won't Get Fooled Again" (1971)**: Classic ARP 2500 / 2600 modular sample-and-hold filter rhythmic gating.

### CLI & MCP Automation
```powershell
synthcraft-mcp update-oscillators --vco1 Saw --vco2 SuperSaw --detune-cents 12 --mix 0.55 --fm 0.15
synthcraft-mcp update-filter --mode LowPass24dB --cutoff-hz 1200 --resonance 0.65 --env-mod 0.45
synthcraft-mcp list-rack-presets
synthcraft-mcp load-rack-preset --id west_coast_vactrol
```

---

## Subpage 1.2: Factory Patch Bank Overview

### What It Does (Plain English)
This page is your studio preset browser and librarian. It displays all factory and custom user patches stored in SynthCraft's local SQLite database (`%LOCALAPPDATA%\synthcraft\synthcraft.db`), showing each patch's musical tempo (BPM), Camelot harmonic key (for DJ mixing), active synthesis engines, and quick audition buttons.

### Controls & Workflow
- **Instant Patch Recall**: Click any patch card -- or use the always-visible `< [Patch] >` arrows in the top studio header -- to switch patches immediately without interrupting playback.
- **A/B Patch Compare & 64-Deep Undo/Redo**: Experiment boldly with knobs, press `Ctrl+Z` / `Ctrl+Y` to step backward or forward up to 64 edits, or toggle A/B compare to audition your edited sound against the original preset.
- **Factory Reset**: Restore the built-in factory patches (`vienna_acid_303`, `goliath_reese_sub`, `danube_dub_chord`, `stephansdom_bell_fm`, `djcraft_riser_sweep`, `soundcraft_warm_pad`) at any time.

### Provenance, Hardware Lineage & History
- **Digital Patch Memory (Sequential Circuits Prophet-5, 1978)**: Before Dave Smith and John Bowen introduced the microprocessor-controlled Prophet-5 in 1978, musicians had to draw knob positions on paper patch sheets between songs. SynthCraft pairs that instant-recall convenience with full JSON serialization and transactional SQLite backups.
- **Reese Bass Heritage (`goliath_reese_sub`)**: Named after Kevin Saunderson's 1988 Detroit techno track "Just Want Another Chance" (released under the alias *Reese*), created by detuning two sawtooth oscillators on a Casio CZ-5000 / analog synth until their phases beat against each other into a thick, snarling bassline.

### Famous Tracks & Iconic Usage
- **Reese (Kevin Saunderson) -- "Just Want Another Chance" (1988)** & **Noisia / Chase & Status**: The blueprint for the detuned `goliath_reese_sub` bass heard across drum & bass, UK garage, and neurofunk.
- **Basic Channel -- "Phylyps Trak" (1993)**: The hypnotic filtered minor-chord dub techno sound behind `danube_dub_chord`.

### CLI & MCP Automation
```powershell
synthcraft-mcp list-patches --json
synthcraft-mcp load-patch --id vienna_acid_303
synthcraft-mcp reset-factory
```
