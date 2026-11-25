# Workbench 14: Performance & Mastering Suite

The finishing workbench turns SynthCraft into a complete live stage rig and radio-ready mastering desk. Trigger lush jazz and house chords with one finger, talk through a classic 16-band analog vocoder, morph seamlessly across 8 full synthesizer scenes with a crossfader or 4-corner XY pad, and polish your final stereo output through a true-peak studio mastering chain.

---

## Subpage 14.1: Chord Memory & Scale Lock

### What It Does (Plain English)
You do not need ten years of conservatory piano training to write rich, emotional chord progressions. **Chord Memory** turns every single key you press into a full polyphonic chord chosen from **10 classic musical voicings** (Major, Minor, Minor 7th, Major 7th, Minor 9th, Sus4, Power 5th, House Stab, Dub Techno Minor 9th, and Neo-Soul 11th). Pair it with **Scale Lock** (Major, Natural Minor, Dorian, Phrygian, Harmonic Minor, Pentatonic, Blues) so every note you play or sequence snaps cleanly into key, plus a built-in **Riff Generator** for instant melodic ideas.

### Controls & Workflow
- **1-Finger Chords**: Enable **Chord Memory** and select a voicing (try `DubMinor9` with a resonant low-pass filter and ping-pong delay for instant Berlin dub techno, or `HouseStab` with the M1 Piano/Organ for classic 90s house).
- **Scale Lock**: Choose your root note (`C` through `B`) and scale -- zero wrong notes during live improvisation.
- **Riff Generator**: Click **Generate Riff** to populate the sequencer with a musical, scale-locked groove in one click.

### Provenance, Hardware Lineage & History
- **Roland Juno-60 / Polysix / Korg Mono/Poly Chord Memory (1981-1982)**: Early 1980s polyphonic synths included a "Chord Memory" button that latched a chord shape and transposed it in parallel up and down the keyboard. That exact parallel minor-7th/minor-9th movement -- technically "forbidden" in classical music theory -- accidentally became the harmonic DNA of **Detroit Techno**, **Chicago Deep House**, and **Berlin Dub Techno**.

### Famous Tracks & Iconic Usage
- **Inner City -- "Good Life" (1988)** & **Derrick May -- "Strings of Life" (1987)**: Classic Detroit house/techno parallel synth chord stabs.
- **Basic Channel -- "Quadrant Dub" (1994)** & **Larry Heard (Mr. Fingers) -- "Can You Feel It" (1986)**: Hypnotic, echoing minor-7th and minor-9th Chord Memory stabs.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-perf-master --chord-enabled true --chord-mode DubMinor9 --scale-lock Minor --root-note 0
```

---

## Subpage 14.2: 16-Band Analog Vocoder

### What It Does (Plain English)
A **Vocoder** ("Voice Encoder") makes your synthesizer literally *talk* or *sing*. It splits a voice signal (the **Modulator** -- either SynthCraft's built-in robotic speech formant generator or live vocal audio coming in from `DJFX`) into **16 frequency bands**, measures how loud each band is, and applies those exact 16 volume envelopes onto your synthesizer's oscillators (the **Carrier**). Play a thick sawtooth chord and hear your synth speak like a cybernetic choir!

### Controls & Workflow
- **Carrier + Modulator**: Play any bright, harmonically rich patch (sawtooth, supersaw, or 6-Op FM works best -- pure sine waves lack the upper harmonics needed for intelligible speech!).
- **Sculpt the Voice**: Toggle **Enable**, adjust **Formant Shift** (shift downward for booming giant robots or upward for bright chipmunk/cyber-pop vocals), **Unvoiced Sibilance** (adds crisp "s" and "t" consonants), and **Wet/Dry Mix**.

### Provenance, Hardware Lineage & History
- **Homer Dudley / Bell Labs Voder & Vocoder (1939)**: Originally invented to compress telephone speech across transatlantic cables and later used for encrypted WWII communications (SIGSALY).
- **Sennheiser VSM-201 (1977), Roland VP-330 Vocoder Plus (1979) & EMS Vocoder 2000 (1977)**: Musicians hijacked the telecom technology to invent the iconic robotic voice of synth-pop, electro, funk, and French house.

### Famous Tracks & Iconic Usage
- **Kraftwerk -- "The Robots" (1978)** & **Herbie Hancock -- "I Thought It Was You" (1978)**: Iconic Sennheiser VSM-201 robotic vocals.
- **Daft Punk -- "Around the World" (1997)** & **"Harder, Better, Faster, Stronger" (2001)**: Made the vocoder / talkbox one of the most beloved hooks in pop and dance history.
- **Laurie Anderson -- "O Superman" (1981)** & **Imogen Heap -- "Hide and Seek" (2005)**: Breathtaking, hauntingly emotional vocal harmony vocoding.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-perf-master --vocoder-enabled true --vocoder-mix 0.85 --vocoder-formant-shift 0.15
```

---

## Subpage 14.3: 8-Scene Morph & XY Pad

### What It Does (Plain English)
Live electronic music thrives on dramatic transitions -- building from a whisper-quiet filtered verse into a roaring, detuned, reverb-drenched festival drop with a single sweep of one hand. The **8-Scene Morph & XY Pad** lets you snapshot up to **8 complete macro scenes** (`Scene 1..8`) and glide smoothly between them using either a horizontal **Live Crossfader (`Scene A <-> Scene B`)** or a **4-Corner XY Morph Pad** that interpolates four scenes simultaneously in two dimensions.

### Controls & Workflow
- **Capture & Recall Scenes**: Click any of the 8 scene buttons to recall or store a snapshot of cutoff, resonance, drive, FX depth, and timbre macros.
- **Crossfader (`A <-> B`)**: Assign any scene to Slot A and Slot B, then slide the crossfader smoothly to morph every parameter together without a single click or audio pop.
- **4-Corner XY Pad**: Drag the glowing puck anywhere inside the square to blend between Corners 1, 2, 3, and 4 continuously.

### Provenance, Hardware Lineage & History
- **Elektron Octatrack Crossfader (2011)**: Beloved by live techno and IDM performers for its silky optical crossfader that morphs dozens of parameters between Scene A and Scene B in one fluid gesture.
- **Prophet VS (1986), Korg Wavestation (1990) & Korg Kaoss Pad (1999)**: Pioneered 2D joystick/touchpad vector morphing between four sonic corners.

### Famous Tracks & Iconic Usage
- **Elektron Live Techno Sets (Surgeon, Blawan, Karenn)** & **Radiohead / Muse (Korg Kaoss Pad)**: Iconic real-time scene morphing and 2D touch-pad performance sweeps.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-perf-master --scene-crossfade 0.5 --xy-x 0.75 --xy-y 0.35
```

---

## Subpage 14.4: Studio Mastering Bus

### What It Does (Plain English)
Before your audio leaves SynthCraft for your speakers, a club PA, or a `.wav` stem export, it passes through a dedicated 4-stage **Studio Mastering Bus** so your patches sound finished, wide, punchy, and clip-free right out of the box:
1. **4-Band Mastering EQ**: Gentle musical shelving and bell bands (`Low`, `Low-Mid`, `High-Mid`, `High Air`) to polish tonal balance.
2. **Sub-Bass Mono Maker (`< 120 Hz`)**: Forces deep sub-bass frequencies below 120 Hz strictly to the center mono channel while leaving mids and highs wide -- preventing muddy stereo phase cancellation on club subwoofers and vinyl records.
3. **Mid/Side (M/S) Stereo Width**: Expand stereo pads past `100%` for huge widescreen immersion, or tighten leads toward mono for laser-focused punch.
4. **True-Peak Brickwall Limiter**: Prevents digital clipping (`0.0 dBFS`) cleanly while raising perceived loudness to competitive streaming and club levels.

### Controls & Workflow
- **Enable Mastering Bus**: Toggle the master chain on and dial **Low/High EQ**, **Mono Bass (<120 Hz)**, **M/S Stereo Width (`0%` to `200%`)**, and **Limiter Ceiling / Drive**.
- **Verify on Meters**: Keep an eye on the gain-reduction meter and Workbench 6 / Workbench 13 scopes so you add punch without squashing dynamics flat.

### Provenance, Hardware Lineage & History
- **Vinyl Lathe Elliptical EQ (Neumann VMS-70, 1960s-1970s)**: Cutting engineers discovered that stereo out-of-phase sub-bass literally threw the vinyl cutting stylus out of the groove, inventing the classic **elliptical mono-bass filter** still essential in every modern club mastering chain.
- **Alan Blumlein's Mid/Side Stereo (1931) & Modern Lookahead Brickwall Limiters (Waves L1 1994, FabFilter Pro-L)**: Defined the clean, wide, loud polish of modern electronic and pop masters.

### Famous Tracks & Iconic Usage
- **Daft Punk -- "Random Access Memories" (2013, mastered by Bob Ludwig)** & **Deadmau5 -- "4x4=12" (2010)**: Benchmarks of crystal-clear sub-bass mono discipline, lush M/S stereo width, and punchy brickwall mastering.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-perf-master --master-enabled true --master-mono-bass true --master-stereo-width 1.25 --master-limiter-drive 0.35
```
