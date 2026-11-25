# Workbench 11: Next-Gen Lab (MSEG / Granular / Spectral / Genopatch)

This is where modern desktop computing power (multi-core CPUs, gigabytes of RAM, and GPU-accelerated visuals) lets SynthCraft leap far beyond what dedicated hardware synth boxes can do. Sculpt custom drawable modulation curves, morph between formant vowels, freeze live audio into 24-grain clouds, paint individual overtones in a 64-partial additive engine, or let a multi-core evolutionary algorithm reverse-engineer any audio sample into a playable SynthCraft patch.

---

## Subpage 11.1: MSEG & Dual Morph Filter B

### What It Does (Plain English)
Standard ADSR envelopes only have four straight segments. A **Multi-Segment Envelope Generator (MSEG)** lets you draw up to **16 custom breakpoints** with adjustable tension curves -- turning modulation into bouncing balls, stuttering rhythmic gates, complex risers, or evolving ambient swells. Alongside it sits **Dual Morphing Filter B**, a second independent filter you can wire in **Series** (Filter A feeds into Filter B) or **Parallel** (both filters blend side-by-side), featuring **Vocal Formant (A-E-I-O-U)**, **Comb +/-** (metallic flanging/resonator), and an aggressive **Korg MS-20 Screamer** mode. Every modulated knob displays a live glowing **Modulation Ring** showing exactly how far the MSEG is moving it in real time.

### Controls & Workflow
- **Draw Your Curve**: Click and drag breakpoints on the 16-point spline canvas, choose `OneShot`, `Loop`, or `TempoSync` mode, and dial in **Rate** and **Depth**.
- **Shape Dual Filter B**: Pick `FormantVowel` (morph smoothly from *Ah* to *Ee* to *Ooh* for talking basses and choirs), `CombPos` / `CombNeg` (for robotic, string-like metallic resonances), or `Ms20Screamer` (for snarling, self-oscillating distortion). Blend between `Series` and `Parallel` routing.

### Provenance, Hardware Lineage & History
- **Drawable MSEGs (Xfer Serum 2014, Matt Tytel's Vital 2020, Kilohearts Phase Plant 2019)**: Steve Duda's *Serum* revolutionized modern electronic production by making modulation visual -- letting producers draw custom LFO/MSEG shapes and see animated rings dancing around every modulated knob.
- **Korg MS-20 Korg-35 Filter (1978)**: Famous for its raw, screaming dual high-pass/low-pass resonant filter that snarls aggressively instead of staying polite like a Moog ladder.
- **Formant & Comb Filters (E-mu Morpheus Z-Plane 1993, Waldorf Q 1999)**: Unlocked talking "growl" basses and metallic resonant sweeps impossible on simple low-pass filters.

### Famous Tracks & Iconic Usage
- **Daft Punk -- "Da Funk" (1995)** & **The Chemical Brothers -- "Block Rockin' Beats" (1997)**: Legendary screaming Korg MS-20 filter leads and basslines.
- **Skrillex -- "Scary Monsters and Nice Sprites" (2010)** & **Noisia -- "Stigma" (2009)**: Defined modern bass music with talking vocal formant filters and comb-filter metallic growls.
- **Flume & Disclosure (2013-2020)**: Signature choppy, bouncing MSEG-drawn synth chord stabs.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-mseg-filter2 --mseg-enabled true --mseg-mode Loop --mseg-rate-hz 2.0 --mseg-depth 0.7 --filter2-enabled true --filter2-mode FormantVowel --filter2-cutoff-hz 1100 --filter2-resonance 0.65 --filter2-morph 0.4 --filter2-mix 0.75
```

---

## Subpage 11.2: Granular Cloud & 64-Partial Spectral

### What It Does (Plain English)
Two radically different ways to turn sound into liquid sculpture:
- **24-Grain Live-Capture Granular Cloud**: Continuously records your live synthesizer output (or `DJFX` audio) into a circular buffer, then sprays up to **24 tiny overlapping audio droplets ("grains")** back out -- letting you freeze a moment in time, scatter grains across the stereo field, pitch-shift them up an octave for angelic shimmer, or turn a short pluck into an infinite cinematic pad.
- **64-Partial Spectral Additive Sculptor**: Builds sound directly from **64 individual harmonic sine waves (partials)**, just like a 64-drawbar super-organ. Tilt the spectrum brighter or darker, shift the odd/even harmonic balance (hollow clarinet vs. rich brass), or stretch the partials out of tune (`Inharmonicity`) to morph a warm organ into shimmering church bells or alien glass chimes.

### Controls & Workflow
- **Granular Cloud**: Toggle **Enable**, hit **Freeze** to lock the live audio buffer in place, and sweep **Grain Density**, **Grain Size (ms)**, **Position**, **Spray (randomness)**, and **Pitch Shift (semitones)**.
- **64-Partial Spectral**: Enable the spectral bank, sculpt **Tilt**, **Odd/Even Balance**, **Comb/Stretch Inharmonicity**, and watch the 64 partial bars react live.

### Provenance, Hardware Lineage & History
- **Granular Synthesis (Iannis Xenakis 1960s, Curtis Roads 1974, Arturia Pigments & Mutable Instruments Clouds 2015)**: Mathematician-composer Iannis Xenakis envisioned sound as clouds of microscopic acoustic particles. Modern desktop CPUs allow SynthCraft to run 24 smooth Hann-windowed stereo grains live with zero clicks.
- **Additive Synthesis (Telharmonium 1897, Hammond Organ 1935, Kawai K5000 1996, Image-Line Harmor 2011)**: Literally builds any timbre from Fourier's recipe of individual harmonic partials.

### Famous Tracks & Iconic Usage
- **Jon Hopkins -- "Open Eye Signal" & "Emerald Rush" (2013-2018)**: Lush, frozen granular clouds and evolving crystalline textures.
- **Radiohead -- "Everything In Its Right Place" (2000)** & **Brian Eno -- "Apollo" (1983)**: Iconic ambient/granular textural manipulation.
- **Hammond B3 / Kawai K5000 Classics (Deep Purple, Aphex Twin)**: Pure harmonic partial sculpting from warm drawbars to glassy additive bells.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-granular-spectral --granular-enabled true --granular-freeze false --granular-density 18 --granular-size-ms 95 --granular-mix 0.55 --spectral-enabled true --spectral-tilt -0.2 --spectral-odd-even 0.6 --spectral-mix 0.45
```

---

## Subpage 11.3: Genopatch AI Audio-to-Patch

### What It Does (Plain English)
Ever heard a cool bass, bell, or lead sound and wished your synthesizer could automatically figure out the knob settings to recreate it? **Genopatch** is a multi-core evolutionary synthesizer matcher (powered by Rust's `std::thread::scope` across all your CPU cores). Feed it a target audio profile -- or click one of the built-in target archetypes -- and watch a population of candidate synth patches evolve across generations, mutating and cross-breeding until their 32-band spectral fingerprint matches the target sound.

### Controls & Workflow
- **Pick a Target & Run Evolution**: Choose a target archetype (or target spectrum), set the number of **Generations** and **Population Size**, and click **Evolve Patch**.
- **Apply Best Candidate**: Watch the spectral fitness score climb in real time across your CPU cores, inspect the top evolved parameters, and click **Apply to Synth** to play the newly discovered patch immediately.

### Provenance, Hardware Lineage & History
- **Sonic Charge Synplant 2 "Genopatch" (Magnus Lidstrom, 2023)**: Stunned the synthesizer world by letting musicians drop any short audio recording onto a seed and letting an AI/evolutionary optimizer grow a playable synth patch that mimics it -- plus a whole garden of musical "mutations" around it. SynthCraft brings a pure-Rust, multi-core, offline-capable evolutionary matcher directly into your studio with zero cloud APIs required.

### Famous Tracks & Iconic Usage
- **Modern Film & Game Sound Design (2023-2026)**: Producers use evolutionary patch matching to turn field recordings (a slammed car door, a violin bow, a subway screech) into fully playable polyphonic synthesizer instruments.

### CLI & MCP Automation
```powershell
synthcraft-mcp run-genopatch --target-preset acid_squelch --generations 12 --population 24 --apply true --json
```
