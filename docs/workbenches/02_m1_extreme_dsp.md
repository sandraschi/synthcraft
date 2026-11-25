# Workbench 2: 1990-Style PCM & Extreme DSP

This workbench pairs two wildly different eras of sound design side by side: the unmistakable punchy digital workstation keyboards that defined 1988-1995 club and pop music, and six cutting-edge mathematical synthesis engines that create plucked strings, crystalline clouds, metallic bars, and West-Coast timbres from scratch.

---

## Subpage 2.1: 1990 PCM Workstation Rompler

### What It Does (Plain English)
In the late 1980s, a new generation of keyboard workstations replaced analog oscillators with short digital recordings ("PCM samples") stored in ROM chips, played back through gritty 12-to-16-bit digital-to-analog converters at 32 kHz. Because memory was tiny, engineers combined a bright attack transient (like a piano hammer strike or organ key-click) with a short looped body -- producing sounds that cut through club sound systems like nothing before or since. SynthCraft recreates this dual-layer "Combi" workstation architecture with 8 classic programs plus loadable CC0 sample banks (including harpsichords/cembalos and cathedral pipe organs).

### Controls & Workflow
- **Choose a Program**: Pick from the 8 classic workstation programs (`Universe`, `Piano 16'`, `Organ 2`, `Fretless Bass`, `Slap Bass`, `Pan Flute`, `Ooh/Aah Choir`, `Orchestra Hit`) and blend them with your analog oscillators using the **PCM Mix** knob.
- **Dual-Layer AI2 Combi Stacking**: Layer two contrasting programs together (for example, a bright percussive attack layer under a lush choir pad) with independent volume and detune.
- **1988 32 kHz / 12-Bit DAC Mode**: Toggle the vintage converter emulation to add subtle aliasing sparkle and mid-range grit that makes house pianos and basslines jump out of a dense mix.
- **Load Custom Sample Banks**: Drop in your own `.wav`, `.sfz`, or `.sf2` SoundFont libraries, or fetch checksum-verified CC0 packs (`vsco2ce-cello-section-sus`, cembalos, pipe organs) with one click.

### Provenance, Hardware Lineage & History
- **Korg M1 (1988)**: The best-selling hardware synthesizer of its era. Its "AI Synthesis" combined 4 MB of 16-bit PCM samples with onboard digital effects and a sequencer. Program 00 (*Universe*), Program 01 (*Piano 16'*), and Program 17 (*Organ 2*) became the defining sonic signatures of 1990s house, rave, and pop.
- **Fairlight CMI (1979) & E-mu Emulator II (1984)**: Pioneered the "Orchestra Hit" (originally sampled from Stravinsky's *The Firebird*) and breathy pan-flute / vocal choir patches.

### Famous Tracks & Iconic Usage
- **Robin S. -- "Show Me Love" (1993)**: One of the most recognizable basslines in dance music history -- played straight off the Korg M1's **Organ 2** preset (`m1_17_organ2`).
- **Madonna -- "Vogue" (1990)** & **Black Box -- "Ride on Time" (1989)**: Driven by the punchy, bright **M1 House Piano (`m1_01_piano16`)** cutting through club mixes.
- **Enigma -- "Sadeness (Part I)" (1990)** & **Peter Gabriel -- "Sledgehammer" (1986)**: Iconic use of breathy **Pan Flute** (`m1_24_panflute`) and **Ooh/Aah Choir** (`m1_36_choir`) textures.
- **Yes -- "Owner of a Lonely Heart" (1983)** & **Afrika Bambaataa -- "Planet Rock" (1982)**: Made the **Orchestra Hit** (`m1_48_orchhit`) an eternal hip-hop and pop staple.

### CLI & MCP Automation
```powershell
synthcraft-mcp load-m1 --program m1_17_organ2 --mix 0.85
synthcraft-mcp list-banks
synthcraft-mcp fetch-bank vsco2ce-cello-section-sus
```

---

## Subpage 2.2: Extreme Modern DSP Engine

### What It Does (Plain English)
Instead of relying on traditional oscillators or recorded samples, the Extreme DSP engine generates audio directly from physics and non-linear mathematics. One single **Morph / Harmonics** macro lets you twist each algorithm from delicate acoustic realism into futuristic electronic textures.

### Controls & Workflow
Select one of 6 algorithms and sweep the **Extreme Morph** and **Mix** controls:
- **Karplus-Strong Pluck**: Models a vibrating guitar, harp, or koto string by sending a tiny burst of noise into a tuned feedback delay line with realistic high-frequency damping.
- **Granular Cloud**: Chops audio into overlapping micro-grains windowed with smooth bell curves, adding +1 octave and +5th shimmer harmonics.
- **Wavetable Morph**: Smoothly glides through a 3D stack of single-cycle waves (from pure Sine to Vocal Formant, Digital Fold, and HyperSaw).
- **West-Coast Wavefolder**: Pushes a smooth sine wave through a 5-stage trigonometric folder paired with a vactrol Low-Pass Gate for woody, rubbery, percussive tones.
- **Phase Distortion (CZ-Style)**: Warps the playback speed of a cosine wave within each single cycle to mimic resonant analog filter sweeps without using a filter at all.
- **Modal Struck Bar**: Resonates a bank of mathematical bar modes (`1.0, 2.756, 5.404, 8.933`) derived from the Euler-Bernoulli beam equation to synthesize marimbas, glockenspiels, vibraphones, and glass bells.

### Provenance, Hardware Lineage & History
- **Karplus-Strong Algorithm (Kevin Karplus & Alex Strong, Stanford CCRMA, 1983)**: A breakthrough in physical modeling that proved a simple filtered delay loop could sound astonishingly like a plucked acoustic string.
- **Casio CZ-101 Phase Distortion (1984)**: Casio's ingenious digital alternative to Yamaha's patented FM synthesis -- beloved for warm, hollow basses and glassy pads.
- **PPG Wave 2.2 (1982) & Waldorf Microwave (1989)**: Wolfgang Palm invented wavetable synthesis in Germany, allowing digital oscillators to sweep continuously between radically different timbres.
- **Mutable Instruments Rings / Plaits (Emilie Gillet, 2015-2018)**: Brought modal resonators and compact West-Coast wavefolders into modern Eurorack studios worldwide.

### Famous Tracks & Iconic Usage
- **Depeche Mode -- "Just Can't Get Enough" / A-ha & Tears for Fears (1980s)**: PPG Wave wavetable pads and crisp digital sweeps.
- **Vince Clarke (Erasure) & early UK Techno**: Heavy users of the Casio CZ-101's snappy Phase Distortion basses.
- **Four Tet & Jon Hopkins**: Modern electronic producers celebrated for shimmering granular clouds, Karplus-Strong plucks, and Mutable-style modal percussion.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-extreme-dsp --mode karplus_strong_pluck --morph 0.6 --mix 0.75
synthcraft-mcp set-extreme-dsp --mode modal_struck_bar --morph 0.45 --mix 0.8
```
