# Workbench 6: Oscilloscope & Spectrum

Synthesizer sound design becomes twice as fast when you can *see* what you are hearing. This workbench gives you classic laboratory-grade audio metering -- a real-time waveform oscilloscope and a 32-band frequency spectrum analyzer -- so you can check bass weight, spot harsh resonant peaks, and watch wavefolding or pulse-width modulation bend the audio waveform live.

---

## Subpage 6.1: Waveform Oscilloscope

### What It Does (Plain English)
An oscilloscope draws a zoomed-in snapshot of your sound wave over time (left to right = time, up and down = air pressure / speaker movement). Watching the wave lets you immediately understand synthesis concepts that sound abstract in words: a sine wave looks like a smooth rolling hill, a sawtooth looks like shark teeth, pulse-width modulation squeezes the wave horizontally, and wavefolding literally folds the peaks back toward the center line.

### Controls & Workflow
- **Live Triggered Waveform**: SynthCraft locks onto zero-crossings so repeating waveforms sit steady on screen instead of jittering wildly.
- **Compare Before & After**: Watch the scope while sweeping the 4-Pole Filter Cutoff (sharp sawtooth edges round off into a smooth sine wave) or adding distortion/wavefolding (smooth curves crease into jagged spikes).

### Provenance, Hardware Lineage & History
- **Cathode-Ray Oscilloscopes (Tektronix 1950s-1970s)**: Early electronic music studios like the BBC Radiophonic Workshop (where Delia Derbyshire created the *Doctor Who* theme in 1963) and WDR Cologne (Karlheinz Stockhausen) relied on glowing green laboratory oscilloscopes to tune oscillators and verify wave shapes before computer displays existed.
- **Korg Minilogue (2016)**: Reintroduced a built-in OLED real-time oscilloscope right on the front panel of an affordable analog synth, making waveform visualization an essential part of modern hardware and software synths.

### Famous Tracks & Iconic Usage
- **Delia Derbyshire / BBC Radiophonic Workshop -- "Doctor Who Theme" (1963)**: Sculpted using test-tone oscillators, tape loops, and laboratory oscilloscopes.
- **Jerobeam Fenderson -- "Oscilloscope Music" (2016)**: Turned XY oscilloscope synthesis itself into an audiovisual art form.

### CLI & MCP Automation
```powershell
synthcraft-mcp preview --duration-ms 250 --json
```

---

## Subpage 6.2: 32-Band DFT Spectrum & dBFS

### What It Does (Plain English)
While the oscilloscope shows *time*, the Spectrum Analyzer shows *frequency* -- from deep sub-bass rumble on the left (20 Hz) to airy treble sparkle on the right (20,000 Hz). It breaks your live sound into 32 frequency bands using a Discrete Fourier Transform (DFT), measures your exact **RMS loudness** (average energy), **Peak dBFS** (loudest instant, where `0 dBFS` is digital clipping), **crest factor** (punchiness), and tracks your fundamental musical pitch in Hertz.

### Controls & Workflow
- **Check Sub-Bass & Headroom**: Make sure your bassline sits cleanly around `40-90 Hz` without muddying the midrange (`250-500 Hz`), and verify your Peak reading stays safely below `0.0 dBFS` before exporting stems.
- **Spot Resonant Spikes**: When cranking filter resonance or comb filters, watch the exact harmonic bar jump upward so you can tune it musically to your track's key.

### Provenance, Hardware Lineage & History
- **Joseph Fourier (1822) & Cooley-Tukey FFT (1965)**: Fourier proved that any complex musical sound can be broken down into a recipe of pure sine waves at different frequencies. Real-time hardware spectrum analyzers (like the Klark Teknik DN60 in 1980s mastering rooms) made frequency balance visible to mix engineers.

### Famous Tracks & Iconic Usage
- **Modern Club Mastering (Berlin Techno, Drum & Bass, EDM)**: Club sound engineers rely on real-time spectrum analyzers and dBFS meters to ensure kicks and sub-basses translate cleanly onto massive festival subwoofers without distortion.

### CLI & MCP Automation
```powershell
synthcraft-mcp analyze-spectrum --duration-ms 500 --json
```
