# Workbench 13: Visuals, MPE & Neural DDSP

See your sound in three dimensions, bend and breathe into individual notes inside a chord with modern MPE touch expression, and transform raw synthesizer waveforms into acoustic violin, flute, saxophone, or cello timbres using a real-time Neural DDSP (Differentiable Digital Signal Processing) harmonic-plus-noise decoder.

---

## Subpage 13.1: 3D Waterfall & Lissajous Scope

### What It Does (Plain English)
Two mesmerizing, high-frame-rate studio analyzers built right into the workspace:
- **24-Slice 3D Perspective Waterfall Spectrogram**: Draws your audio spectrum flowing backward into 3D space over time -- like a glowing mountain range of harmonics. Watch filter sweeps carve valleys, shimmer reverbs light up high mountain peaks, and 808 kicks thunder down the low-frequency canyon.
- **Lissajous Stereo Vectorscope & Phase Correlation Meter**: Plots Left vs. Right audio channels as a glowing X/Y cloud so you can instantly verify stereo width and club-system mono compatibility. A vertical line means pure mono; a wide balanced sphere means lush stereo; a flat horizontal line warns of phase cancellation (check the **Phase Correlation** bar from `-1.0` out-of-phase to `+1.0` mono-safe).

### Controls & Workflow
- **Watch Live While Playing**: Play notes or run the 64-step sequencer to see the 24-slice waterfall scroll smoothly in real time.
- **Check Club Mono Safety**: Keep the Phase Correlation readout above `+0.2` on basslines and leads so your mix never vanishes when played on a mono club PA or phone speaker.

### Provenance, Hardware Lineage & History
- **RTW / DK-Audio Stereo Phase Scopes (1980s-1990s)** & **iZotope Insight / FabFilter Pro-Q**: Broadcast and mastering studios mounted CRT Lissajous vectorscopes next to mixing consoles so engineers could catch stereo phase problems at a glance.
- **3D Waterfall Spectrograms (Sonar & Spectral Lab Tools -> modern GPU synths)**: Turned frequency-over-time analysis into an intuitive visual compass for sound designers.

### Famous Tracks & Iconic Usage
- **Pink Floyd -- "The Dark Side of the Moon" (1973)** & **Tipper / Amon Tobin**: Masterclasses in wide, phase-coherent 3D stereo imaging that look like living geometric art on a Lissajous vectorscope.

### CLI & MCP Automation
```powershell
synthcraft-mcp analyze-spectrum --duration-ms 500 --json
```

---

## Subpage 13.2: MPE & CLAP Poly-Mod Touch Pad

### What It Does (Plain English)
Traditional keyboards treat every finger in a chord identically: if you move the pitch wheel or filter knob, *all* notes move together. **MPE (MIDI Polyphonic Expression)** and **CLAP `1.2.2` Poly-Mod** break that limitation. Every single held note gets its own independent **Pitch Bend (X)**, **Timbre Slide (Y)**, and **Pressure (Z)** -- and you can test and perform it right on screen with SynthCraft's interactive **2D MPE Expression Touch Pad**, even if you do not own an MPE hardware controller yet.

### Controls & Workflow
- **Perform on the 2D Touch Pad**: Click and drag horizontally to bend pitch smoothly on the active voice, and drag vertically to open the filter, sweep wavetable position, or add vibrato.
- **Hardware MPE & DAW Poly-Mod**: Plug in an Expressive E Osmose, ROLI Seaboard, or LinnStrument, or load `SynthCraft.clap` in SoundCraft / Bitwig Studio to send live `CLAP_EVENT_NOTE_EXPRESSION` and `CLAP_EVENT_PARAM_MOD` automation per individual note.

### Provenance, Hardware Lineage & History
- **Yamaha CS-80 (1977)**: The legendary 220-pound analog monolith with polyphonic aftertouch and a ribbon controller -- the first synth to feel as expressive under the fingers as a cello or trumpet.
- **Expressive E Osmose (2022) & ROLI Seaboard (2013)**: Brought three-dimensional per-finger expression into modern studios via the MPE standard.

### Famous Tracks & Iconic Usage
- **Vangelis -- "Blade Runner (End Titles)" & "Blade Runner Blues" (1982)**: The single most famous demonstration of polyphonic per-note expression in synthesizer history, performed live on the Yamaha CS-80.
- **Toto -- "Africa" (1982)** & **Peter Gabriel -- "Red Rain" (1986)**: Lush, breathing polyphonic-aftertouch synth brass and pads.

### CLI & MCP Automation
*(Live per-note expression runs automatically over MIDI MPE and CLAP `1.2.2` `CLAP_EVENT_NOTE_EXPRESSION` / `CLAP_EVENT_PARAM_MOD`.)*

---

## Subpage 13.3: Neural DDSP Timbre Transfer

### What It Does (Plain English)
**DDSP (Differentiable Digital Signal Processing)** is a breakthrough in musical AI: instead of generating muddy, glitchy raw audio samples with a giant black-box neural network, a lightweight neural decoder predicts clean, interpretable **32-harmonic additive amplitudes + filtered breath/bow noise** directly from your live played pitch and loudness. That means you can play any melody on your synthesizer and smoothly morph its timbre into a **Solo Violin**, **Concert Flute**, **Tenor Saxophone**, **Warm Cello**, **Trumpet**, or **Ethereal Choir** in real time on your CPU with zero latency spikes.

### Controls & Workflow
- **Pick a Latent Timbre Embedding**: Select from 6 acoustic-neural embeddings (`Violin`, `Flute`, `Saxophone`, `Cello`, `Trumpet`, `Choir`).
- **Sculpt Harmonic vs. Noise Balance**: Dial **Harmonic Gain** (the singing pitched body), **Noise Gain** (bow rosin, flute breath chiff, or sax reed air), **Brightness**, and **Wet/Dry Mix** to blend acoustic realism with pure electronic synthesis.

### Provenance, Hardware Lineage & History
- **Google Magenta DDSP (Jesse Engel, Lamtharn Hantrakul, Chenjie Gu & Adam Roberts, ICLR 2020)**: Demonstrated that combining classic DSP oscillators and filtered noise with compact neural control achieves lifelike acoustic timbre transfer using a tiny fraction of the compute of raw waveform neural audio models.

### Famous Tracks & Iconic Usage
- **Google Magenta Tone Transfer & Modern Experimental Electronic Pop (2020-2026)**: Producers hum a melody into a microphone or play a simple synth line and transform it into hyper-expressive hybrid violin/saxophone/synth hybrids that bend physics in ways no physical instrument can.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-ddsp-timbre --enabled true --preset Violin --harmonic-gain 0.85 --noise-gain 0.25 --mix 0.7
```
