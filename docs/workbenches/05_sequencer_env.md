# Workbench 5: Sequencer, Drums & ADSR/LFO

This workbench is your rhythm and movement command center. It combines a 4-bar (64-step) acid step sequencer, a complete 8-voice pure-Rust analog/FM drum machine inspired by the legendary Roland TR-808 and TR-909, and the envelope, LFO, modulation matrix, arpeggiator, and microtonal tuning controls that bring static tones to life.

---

## Subpage 5.1: 64-Step Acid Sequencer (4 Bars)

### What It Does (Plain English)
A step sequencer plays a repeating grid of musical notes locked tightly to your project tempo. While classic acid boxes stopped at 16 steps (one single bar), SynthCraft expands seamlessly to **16, 32, 48, or 64 steps (4 full bars)** with a clean `Bar A / B / C / D` pager -- so you can write evolving basslines, chord progressions, and melodic turnarounds without repeating every 2 seconds.

### Controls & Workflow
- **Length & Bar Pager**: Click `16`, `32`, `48`, or `64` steps to set the loop length, and flip between `Bar A` (steps 1-16), `Bar B` (17-32), `Bar C` (33-48), and `Bar D` (49-64).
- **Per-Step Acid Controls**: Every step has a toggle switch, MIDI pitch (`C2` to `C6`), velocity, gate length, **Accent** (which boosts both volume and filter bite), and **Slide** (which glides smoothly in pitch into the next note like a real TB-303).
- **Live Record (`Rec`)**: Arm `Rec` in the transport bar to overdub notes live from your USB MIDI keyboard or on-screen piano while the loop plays, or step-record one note at a time when stopped.

### Provenance, Hardware Lineage & History
- **Roland TB-303 Bass Line (Tadao Kikumoto, 1981)**: Originally designed to replace human bass guitarists for solo lounge musicians, the TB-303 was a commercial flop -- until Chicago producers like DJ Pierre (Phuture) bought them for \$50 in pawn shops, cranked the resonance and accent knobs, and accidentally invented **Acid House**. The secret to the 303's hypnotic "squelch" is the interaction between its accent circuit (which briefly narrows filter decay and punches the envelope) and its fixed-time pitch slide.

### Famous Tracks & Iconic Usage
- **Phuture -- "Acid Tracks" (1987)**: The ground-zero Chicago Acid House record built around a single 16-step TB-303 pattern twisted live for 12 minutes.
- **Daft Punk -- "Da Funk" (1995)**: An iconic snarling, accented acid riff that carried the TB-303 sound onto pop radio worldwide.
- **Josh Wink -- "Higher State of Consciousness" (1995)** & **Aphex Twin -- "Windowlicker" (1999)**: Legendary demonstrations of extreme accent/slide programming.

### CLI & MCP Automation
```powershell
synthcraft-mcp set-sequencer-length --steps 64
synthcraft-mcp set-step --step 0 --midi-note 36 --accent true --slide false --active true
```

---

## Subpage 5.2: TR-808/909 8-Voice Drum Machine

### What It Does (Plain English)
No samples required: this is an 8-voice percussion synthesizer that generates every drum hit mathematically in real time, allocation-free on the audio thread. It runs on its own 64-step (4-bar) pattern grid synchronized to the main sequencer, complete with realistic closed/open hi-hat choking, groove swing, Euclidean rhythm auto-fill, and 6 instant club kits.

### Controls & Workflow
- **8 Independent Synthesized Voices**:
  1. `BD` -- **Kick Drum**: Sine pitch-envelope sweep with click transient and tunable boom decay (from tight 909 punch to earth-shaking 808 sub).
  2. `SD` -- **Snare Drum**: Dual tuned body oscillators blended with snappy bandpass-filtered noise.
  3. `CP` -- **Handclap**: Classic triple-burst micro-delayed noise envelope plus reverberant tail.
  4. `CH` & `OH` -- **Closed & Open Hi-Hat**: 6-oscillator metallic square-wave cluster through a high-pass filter, with authentic **hi-hat choke** (a closed hat immediately silences an open hat).
  5. `LT` -- **Low Tom**: Pitch-dropping resonant membrane drum.
  6. `CB` -- **Cowbell**: Dual detuned square waves (`540 Hz` + `800 Hz`) through a sharp bandpass filter.
  7. `CY` -- **Crash / Ride Cymbal**: Bright shimmering metallic cluster with long decay.
- **6 Instant Club Kits**: Load `Classic 808`, `Classic 909`, `Berlin Techno`, `Electro Funk`, `Trap Sub`, or `Industrial FM` in one click.
- **Euclidean Auto-Fill & Swing**: Use the Euclidean fill button to distribute drum hits mathematically (like 5 hits across 16 steps for instant Latin/Afro/Techno polyrhythms) and dial in **Swing** (`50%` straight to `75%` shuffled groove).

### Provenance, Hardware Lineage & History
- **Roland TR-808 Rhythm Composer (1980)** & **Roland TR-909 (1983)**: Designed by Ikutaro Kakehashi and Tadao Kikumoto in Osaka. The 808 used pure analog transistor circuits (bridged-T resonators for the booming kick and cowbell, six Schmitt-trigger square oscillators for the metallic hi-hats), while the 909 combined punchy analog kicks/snares with crisp 6-bit metallic cymbals. Together, they formed the rhythmic backbone of hip-hop, electro, house, techno, and pop for four decades.
- **Euclidean Rhythms (Godfried Toussaint, McGill University, 2005)**: Proved that Euclid's ancient greatest-common-divisor algorithm naturally generates almost every traditional world music timeline (Cuban tresillo, bossa nova, West African bell patterns).

### Famous Tracks & Iconic Usage
- **Marvin Gaye -- "Sexual Healing" (1982)** & **Afrika Bambaataa -- "Planet Rock" (1982)**: Put the TR-808 kick, snare, clap, and cowbell at the center of R&B and hip-hop forever.
- **Phil Collins -- "In the Air Tonight" (1981)** & **Whitney Houston -- "I Wanna Dance with Somebody" (1987)**: Iconic 808/analog drum machine grooves.
- **Frankie Knuckles -- "Your Love" (1987)** & **Jeff Mills -- "The Bells" (1996)**: The punchy TR-909 kick and sizzling 909 hi-hats that defined Chicago house and Detroit/Berlin techno.

### CLI & MCP Automation
```powershell
synthcraft-mcp load-drum-preset --preset berlin_techno --enabled true --master-level 0.9 --swing 0.56
```

---

## Subpage 5.3: ADSR Envelope & Tempo LFO

### What It Does (Plain English)
A raw oscillator plays at full volume forever -- like a car horn. Envelopes and LFOs give sound shape and motion. The **ADSR Envelope** sculpts how a note starts, settles, holds, and fades (`Attack`, `Decay`, `Sustain`, `Release`). The **Tempo-Synced LFO** (Low-Frequency Oscillator) acts like an invisible hand rhythmicly turning a knob back and forth in time with your BPM. This page also houses the **8-Route Modulation Matrix**, **Real-Time Arpeggiator**, and **Scala `.scl` Microtonal Tuning** engine.

### Controls & Workflow
- **ADSR & Tempo LFO**: Dial in snappy plucks (short attack, short decay, low sustain) or lush slow-swelling pads (long attack, long release). Sync the LFO to musical divisions (`1/4`, `1/8`, `1/16`, dotted, triplet) for dubstep wobbles or rhythmic filter pulses.
- **8-Route Modulation Matrix**: Connect any of 6 modulation sources (`LFO`, `ADSR Env`, `Velocity`, `Mod Wheel`, `Sample & Hold`, `Ambient Drift`) to any of 9 destinations (`Cutoff`, `Resonance`, `FM Index`, `Pulse Width`, `VCO2 Detune`, `Wavefold`, `Stereo Pan`, `Extreme Morph`, `Reverb Mix`) with positive or negative depth.
- **Arpeggiator**: Hold a chord and let SynthCraft ripple through its notes (`Up`, `Down`, `Up/Down`, `Random`) across 1 to 4 octaves, with a **Latch** button so you can take your hands off the keys to tweak filters.
- **Scala `.scl` Microtuning**: Break free from standard 12-note piano tuning with 7 built-in scales (`12-TET`, `5-Limit Just Intonation`, `Pythagorean`, `19-TET`, `24-TET Quarter-Tone`, `31-TET`, `Bohlen-Pierce`) or load any `.scl` file from the massive Scala archive.

### Provenance, Hardware Lineage & History
- **ADSR Envelope (Robert Moog & Vladimir Ussachevsky, 1965)**: Columbia-Princeton Electronic Music Center composer Vladimir Ussachevsky proposed the 4-stage Attack-Decay-Sustain-Release shape to Bob Moog, creating the universal contour model used in virtually every synthesizer since.
- **Modulation Matrix (Oberheim Matrix-12, 1985)**: Replaced physical patch cables with a flexible digital routing grid capable of subtle velocity expression or wild cross-modulation.
- **Arpeggiator (Roland Jupiter-4 / Juno-6, 1978-1982)**: Turned simple triad chords into hypnotic, pulsing synth-pop and trance backdrops.
- **Scala `.scl` Format (Manuel Op de Coul, 1992)**: The global open standard for microtonal and historical tuning systems, embraced by modern microtonal artists like Aphex Twin, Wendy Carlos, and Sevish.

### Famous Tracks & Iconic Usage
- **Duran Duran -- "Save a Prayer" & "Rio" (1982)**: Classic Roland Jupiter-4 / Juno-6 latched arpeggiators.
- **Wendy Carlos -- "Beauty in the Beast" (1986)** & **Aphex Twin -- "Flim" / "Syro" (2014)**: Famous explorations of custom microtonal tunings and expressive modulation routing.

### CLI & MCP Automation
```powershell
synthcraft-mcp update-envelope-lfo --attack-ms 12 --decay-ms 280 --sustain 0.4 --release-ms 450 --lfo-rate-hz 4.0 --lfo-depth 0.35
synthcraft-mcp set-mod-route --slot 0 --source Velocity --dest Cutoff --amount 0.65
synthcraft-mcp set-arpeggiator --mode UpDown --octaves 2 --latch true
synthcraft-mcp set-tuning --scale 19-TET
```
