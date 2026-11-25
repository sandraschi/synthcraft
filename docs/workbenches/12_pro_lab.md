# Workbench 12: Pro Lab (6-Op FM / Unison / Reorderable FX / 2D Physics & 8x WDF)

This workbench unleashes serious studio horsepower: a full 6-operator Thru-Zero FM matrix, a massive 16-voice Hyper-Unison stack, a reorderable pro studio effects rack, a real-time 2D Finite-Difference Time-Domain vibrating plate simulator, and 8x oversampled Wave Digital Filter models of legendary analog circuits.

---

## Subpage 12.1: 6-Op FM Matrix & 16-Voice Unison

### What It Does (Plain English)
- **6-Operator Thru-Zero FM Matrix**: Instead of fixed vintage algorithms, you get a complete **6x6 modulation grid** where any of the 6 sine operators can frequency-modulate any other operator (or itself for gritty feedback). Because it uses **Thru-Zero FM** (allowing oscillator phase to smoothly reverse through zero), even extreme modulation stays rock-solid in musical pitch -- ideal for crystalline electric pianos, metallic bells, woody marimbas, and ripping modern basslines.
- **16-Voice Hyper-Unison**: Stacks up to **16 detuned copies** of your sound spread wide across the stereo panorama with customizable detune curves and stereo width -- turning a single humble sawtooth into a towering festival trance lead or cavernous cinematic pad.

### Controls & Workflow
- **FM Matrix**: Enable the 6-Op engine, set operator harmonic **Ratios** (`0.5`, `1.0`, `2.0`, `3.5`, `7.0`...), tweak individual modulation depths in the 6x6 matrix, or load a preset algorithm (Electric Piano, Tubular Bell, Slap FM Bass, Metallic Perc).
- **Hyper-Unison**: Dial **Voices** (`1` to `16`), **Detune (cents)**, and **Stereo Spread (`0%` mono to `100%` ultra-wide)**.

### Provenance, Hardware Lineage & History
- **FM Synthesis (John Chowning, Stanford 1967 -> Yamaha DX7, 1983)**: John Chowning discovered that modulating one sine wave's frequency at audio rate with another creates extraordinarily rich, bell-like and brassy harmonics. Yamaha licensed his patent and released the **DX7** in 1983, which instantly reshaped 1980s pop, R&B, and film music. Later, **Native Instruments FM8 (2006)** replaced fixed DX7 algorithms with a freeform 6x6 routing matrix, and modular designers introduced **Thru-Zero FM** so deep modulation never drifted flat or sharp.
- **SuperSaw & Hyper-Unison (Roland JP-8000, 1996 -> Access Virus TI HyperSaw, 2005)**: Roland's JP-8000 stacked 7 detuned sawtooth waves on a single key, single-handedly defining the euphoric trance anthem sound of the late 1990s and 2000s.

### Famous Tracks & Iconic Usage
- **A-ha -- "Take On Me" (1985)** & **Whitney Houston -- "Greatest Love of All" (1985)**: Iconic Yamaha DX7 crystalline electric piano (`E.PIANO 1`) and punchy FM synth bass.
- **Kenny Loggins -- "Danger Zone" (1986)** & **Phil Collins -- "Sussudio" (1985)**: Classic DX7 metallic brass and bass punch.
- **Skrillex -- "Bangarang" (2011)**: Modern extreme FM growl basses built on multi-operator modulation matrices.
- **Darude -- "Sandstorm" (1999)** & **Tiesto -- "Adagio for Strings" (2004)**: Towering Roland JP-8000 SuperSaw / Hyper-Unison trance leads.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-fm6-unison --fm-enabled true --fm-mix 0.7 --unison-voices 12 --unison-detune-cents 28 --unison-spread 0.85
```

---

## Subpage 12.2: Reorderable FX (OTT/IR/Shimmer/Tape)

### What It Does (Plain English)
Raw synthesizer sounds come alive inside a great studio effects chain -- and the *order* of your effects changes the sound dramatically (putting Reverb *before* OTT compression creates huge roaring shoegaze walls; putting Tape saturation *after* Delay glues echoes together warmly). SynthCraft gives you **4 boutique studio effects** that you can drag or click to reorder freely:
1. **OTT 3-Band Multiband Compressor**: Splits audio into Low, Mid, and High bands and applies aggressive upward + downward compression ("Over The Top"), bringing quiet details forward and giving leads and basses that unmistakable modern crisp loudness.
2. **Convolution IR Reverb**: Places your synth inside a real acoustic space (cathedral, tiled hall, vintage spring tank, studio plate) by convolving your audio with an Impulse Response.
3. **Shimmer Reverb**: Feeds a lush reverb tail through an octave-up (`+12 semitones`) pitch shifter for glowing, angelic ambient halos.
4. **Analog Tape & Bitcrusher**: Adds warm magnetic tape saturation, gentle wow/flutter pitch drift, and optional lo-fi sample-rate/bit-depth crunch.

### Controls & Workflow
- **Reorder Slots**: Use the up/down arrows on each rack unit to change the exact signal flow order on the fly.
- **Dial Mix & Depth**: Each unit has an independent enable toggle, wet/dry mix, and character controls.

### Provenance, Hardware Lineage & History
- **OTT Multiband Compression (Ableton Live Multiband Dynamics preset -> Steve Duda's Xfer OTT)**: Originally a hidden preset inside Ableton Live, EDM and hyperpop producers fell in love with its hyper-bright upward compression, turning "OTT" into a verb in modern production.
- **Shimmer Reverb (Brian Eno & Daniel Lanois, 1984)**: Invented in the studio by patching an Eventide H949/H3000 harmonizer into the feedback loop of an AMS RMX16 / Lexicon 224 reverb.
- **Analog Tape Saturation (Roland RE-201 Space Echo 1974, Studer A800)**: Magnetic tape naturally softens harsh digital highs and thickens low-mid punch.

### Famous Tracks & Iconic Usage
- **U2 -- "The Unforgettable Fire" (1984) & Brian Eno -- "Apollo" (1983)**: The birth of **Shimmer Reverb**.
- **Boards of Canada -- "Roygbiv" (1998)**: Iconic warm, wobbly **Analog Tape** wow/flutter nostalgia.
- **Porter Robinson, Madeon, Deadmau5 & Skrillex (2010s-2020s)**: Signature bright, larger-than-life **OTT** multiband synth gloss.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-studio-fx --ott-enabled true --ott-depth 0.55 --ir-enabled true --ir-mix 0.35 --shimmer-enabled true --shimmer-mix 0.3 --tape-enabled true --tape-drive 0.4
```

---

## Subpage 12.3: 2D FDTD Mesh & 8x WDF Circuits

### What It Does (Plain English)
Two flagship "Goliath CPU" physics engines that would melt a tiny hardware synth chip:
- **8x8 2D FDTD Physical Modeling Mesh**: Simulates an actual vibrating two-dimensional physical membrane, steel plate, or glass pane in real time using the 2D wave equation (Finite-Difference Time-Domain). Strike or excite it at different coordinates to synthesize living snare skins, Tibetan singing bowls, bowed metallic plates, and organic percussion that never sound identical twice.
- **8x Oversampled Wave Digital Filter (WDF) Circuits**: Rather than approximating analog warmth with a cheap curve, WDF solves the actual electrical Kirchhoff voltage and current equations of real vintage components at **8x internal sample rate**:
  - **Moog CP3 Mixer Overdrive**: Legendary asymmetric discrete-transistor clipping that thickens basslines with warm even harmonics.
  - **Buchla 292 Vactrol Low-Pass Gate**: Models the sluggish light-dependent resistor (cadmium-sulfide cell) that gives West-Coast synths their organic, woody decay.
  - **TB-303 Diode Ladder Filter**: Models the quirky 18 dB/octave diode clipping and bass-thinning resonance of Roland's iconic acid box.

### Controls & Workflow
- **2D FDTD Mesh**: Enable the mesh, choose a material (`Membrane`, `SteelPlate`, `GlassPane`, `WoodMarimba`), and adjust **Tension**, **Damping**, and **Strike Position**.
- **8x WDF Circuit**: Enable the circuit stage, choose `MoogCp3`, `Buchla292Lpg`, or `Tb303Diode`, and push **Drive** and **Mix** to hear authentic component-level analog saturation.

### Provenance, Hardware Lineage & History
- **Wave Digital Filters (Alfred Fettweis, Ruhr University Bochum, 1971 -> Kurt Werner / Stanford CCRMA, 2015)**: Mathematical framework that converts real analog circuit schematics (resistors, capacitors, diodes, transistors) directly into stable, zero-delay digital audio filters.
- **2D Waveguide & FDTD Acoustic Simulation (Stefan Bilbao, University of Edinburgh, *Numerical Sound Synthesis*, 2009)**: Brought true 2D vibrating-plate physics to real-time musical synthesis.

### Famous Tracks & Iconic Usage
- **Moog Modular CP3 Overdrive (Wendy Carlos, Keith Emerson, Tangerine Dream 1970s)**: The unmistakable fat, crunchy analog overdrive heard when hot oscillator signals slam into a Moog CP3 mixer.
- **Buchla 292 Vactrol LPG (Morton Subotnick, Kaitlyn Aurelia Smith)**: Organic, breathing electronic percussion that blurs the line between acoustic instruments and electricity.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-physical-wdf --mesh-enabled true --mesh-material SteelPlate --mesh-tension 0.6 --mesh-damping 0.25 --wdf-enabled true --wdf-mode MoogCp3 --wdf-drive 0.65 --wdf-mix 0.8
```
