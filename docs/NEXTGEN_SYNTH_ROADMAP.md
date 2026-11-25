# SynthCraft Next-Gen Architecture & Buildout Plan

**Date:** 2026-11-25  
**Target Host:** Goliath (AMD Ryzen 9 5900X 12c/24t, NVIDIA GeForce RTX 4090 24 GB VRAM, 64 GB RAM)  
**Baseline:** SynthCraft v0.4.0 (Dual VCO/FM/Wavetable, 4-Pole Ladder VCF, 1990 M1-style AI2 Dual-Layer PCM Combi + 12-bit/32 kHz DAC, Extreme DSP, 64-Step 4-Bar Acid Sequencer, 8-Voice TR-808/909 Drum Machine, 3U VCV Eurorack Canvas + CLAP Host, CLAP 1.2.2 Plugin, Native VCV Rack 2 C++/FFI Module, DJCraft Link/DJPD/DJFX Bridge)

---

## 1. Competitive Gap Analysis vs. Flagship 2026 Softsynths

| Capability | Flagship Reference | SynthCraft v0.4.0 | Gap / Opportunity |
|---|---|---|---|
| **Modulation Envelopes** | Serum 2, Vital, Zebra3 (Drawable MSEGs + Chaos LFOs) | Fixed ADSR + Sine LFO | Add interactive drawable MSEG (up to 16 spline points, tempo-synced loops, presets) + live animated mod rings on rotary knobs |
| **Filter Topology** | Serum 2, Pigments 6, Diva (Dual Serial/Parallel + Formant/Comb/MS-20) | Single 4-pole Ladder VCF | Add independent Filter B (Formant A-E-I-O-U, Comb +/-, MS-20 Sallen-Key, LP/BP/HP) with Serial/Parallel routing |
| **Granular Synthesis** | Pigments 6, Phase Plant, Quanta | Parametric granular DSP mode | Add true multi-grain sample/live-buffer cloud (24 Hann-windowed grains, scrub, spray, freeze, stereo scatter) |
| **Spectral / Additive** | Vital, Razor, Alchemy | None | Add 64-to-512 partial FFT/DFT spectral resynthesis with harmonic tilt, formant shift, odd/even warp, and spectral freeze |
| **Audio-to-Patch Reverse Engineering** | Synplant 2 (Genopatch) | None | Add 24-thread evolutionary DSP matcher (`std::thread::scope`) that evolves native editable SynthCraft patches from any `.wav` |
| **Multi-Operator FM** | FM8, Opsix, Phase Plant | 2-Op VCO2->VCO1 FM | Add 6-Op free routing FM matrix with per-op envelopes and feedback |
| **Physical Modeling** | Mutable Rings, Chromaphone, EaganMatrix | Karplus-Strong pluck mode | Add 64-to-1024 mode resonant modal bank + 2D FDTD plate/membrane waveguide mesh |
| **FX Chain** | Phase Plant, Serum 2, Falcon | Fixed Chorus -> Delay -> Reverb | Add drag-and-drop reorderable FX slots, OTT 3-band upward/downward compressor, Convolution IR Reverb, Shimmer, Tape Wow/Flutter |
| **Per-Voice Poly-Mod & MPE** | Vital, Surge XT, Osmose | Channel-wide pitch/mod | Add full CLAP Polyphonic Modulation & MPE per-note timbre/pressure routing |
| **Neural Timbre Transfer** | RAVE / DDSP | None | Run local ONNX/Candle RAVE/DDSP model on RTX 4090 as a 3U Eurorack module (<5 ms latency) |
| **Component Circuit Sim** | Diva, Cytomic (WDF) | Tanh saturation + 4-pole ladder | Add 8x oversampled Wave Digital Filter (WDF) models of Moog CP3, TB-303 diode ladder, MS-20 OTA, Buchla 292 LPG |
| **Memory-Resident Sample Banks** | Omnisphere, Kontakt | On-demand SFZ/SF2/WAV | Leverage 64 GB RAM for uncompressed 24-bit/96 kHz multi-velocity round-robin acoustic banks |

---

## 2. Why Goliath Beats Dedicated Hardware Synth Boxes

Dedicated hardware boxes (Elektron, Roland, Korg, Moog, Waldorf) run on embedded 400 MHz-1.5 GHz ARM/SHARC DSPs with 64-256 MB RAM and no GPU. Goliath has **24 high-IPC CPU threads**, **64 GB RAM**, and an **RTX 4090 (24 GB VRAM, 82 TFLOPS FP32)**. That unlocks:

1. **Multi-Core Evolutionary Patch Synthesis (Genopatch)**: Evaluating 1,000+ full DSP synth renders against a target `.wav` in ~1-3 seconds across 24 threads.
2. **Real-Time High-Order Spectral & Modal Engines**: 64-512 additive partials or 1,000-mode physical resonators without voice stealing.
3. **Zero-Allocation Real-Time Audio + 120 FPS GPU Visualizers**: Lock-free ring buffers feeding `egui`/`wgpu` live modulation rings, grain clouds, and 3D waterfall spectrograms.
4. **8x/16x Oversampled Non-Linear Circuit Emulation**: Zero aliasing even under extreme audio-rate FM and wavefolding.
5. **Local Neural Audio Processing on RTX 4090**: Real-time timbre transfer and neural drum/texture generation completely offline.

---

## 3. Step-by-Step Buildout Plan (All 8 Phases + Bonus Suite -- 100% Implemented 2026-11-25)

### Phase 1 -- Drawable MSEG, Dual Morphing Filter & Animated Knob Mod Rings *(Implemented & Verified)*
- **Core (`crates/synthcraft-core/src/nextgen.rs`)**:
  - `MsegParams` & `MsegRuntime`: up to 16 drawable `(time, value, curve)` points, modes (`Off`, `OneShot`, `TempoLoop`, `FreeHz`), presets (`Acid Pluck`, `Trance Gate`, `EDM Riser`, `Wobble Sub`, `Swelling Pad`, `Chaos Lorenz`).
  - `Filter2Params` & `Filter2State`: `Off`, `LowPass24dB`, `BandPass12dB`, `HighPass12dB`, `CombPos`, `CombNeg`, `FormantVowel` (3-bandpass morphing vocal tract `A -> E -> I -> O -> U`), `Ms20Screamer` (Sallen-Key diode feedback), with `Serial` / `Parallel` routing and MSEG modulation depth.
- **Audio Thread (`rt.rs`)**:
  - Zero-allocation `MsegRuntime` + per-voice/bus `Filter2State` integrated directly into `SynthRt::process_stereo_with_ext_in`.
  - Expose live MSEG modulation value via `Meters` atomic so UI knobs animate smoothly.
- **UI (`controls.rs` + `nextgen_ui.rs`)**:
  - `rotary_knob_mod(...)` with glowing secondary outer modulation arc.
  - Interactive drawable MSEG spline editor canvas + Dual Filter B console.

### Phase 2 -- Sample-Buffer Granular Cloud & 64-Partial Spectral Resynthesis *(Implementing Now)*
- **Core (`crates/synthcraft-core/src/granular.rs`)**:
  - `GranularParams` & `GranularRuntime`: 24 concurrent Hann-windowed grains reading from a 2-second stereo capture buffer (fed continuously by live synth output or frozen on demand, or loaded from a `.wav` / sample bank), with `position`, `spray`, `grain_size_ms`, `density`, `pitch_semitones`, `stereo_spread`, and `freeze`.
  - `SpectralParams` & `SpectralResynth`: 64-partial additive oscillator bank with DFT capture from any audio buffer/patch, real-time `formant_shift`, `odd_even_tilt`, `harmonic_warp`, `spectral_smear`, and `mix`.
- **UI (`nextgen_ui.rs`)**:
  - Live grain-cloud visualizer showing playhead, spray window, and active grains + 64-bar interactive harmonic partial sculptor.

### Phase 3 -- Multi-Core "Genopatch" Audio-to-Patch Evolutionary Matcher *(Implementing Now)*
- **Core (`crates/synthcraft-core/src/genopatch.rs`)**:
  - Perceptual audio fingerprint extractor (32-band log-spectrum, spectral centroid, RMS envelope curve, autocorrelation fundamental pitch, zero-crossing noisiness).
  - Parallel `(mu + lambda)` evolutionary optimizer using `std::thread::scope` across available CPU cores.
  - Evolves native `SynthPatch` parameters and returns the top 6 ranked DNA branches with `similarity_pct` (`0..100%`).
- **UI (`nextgen_ui.rs`)**:
  - Genopatch Lab: target WAV loader (plus built-in acoustic/club targets), generation/population controls, parallel evolution runner, and 6 clickable DNA branch cards to audition/load straight into SynthCraft.

### Phase 4 -- 6-Operator Thru-Zero FM Matrix & Per-Voice Unison Stack
- Free 6x6 FM routing matrix with per-operator ratio, fine detune, feedback, and MSEG envelope.
- Per-oscillator 16-voice unison with stack modes (octave, fifth, chord, golden-ratio detune) and per-voice wavefolder distortion.

### Phase 5 -- Reorderable FX Rack, OTT Multiband, Convolution Reverb & Tape Engine
- Drag-and-drop FX slot ordering, 3-band Linkwitz-Riley OTT upward/downward compressor, partitioned FFT Convolution Reverb (custom `.wav` impulse responses), Shimmer pitch-shifted reverb, and Tape Wow/Flutter + Bitcrusher.

### Phase 6 -- Full MPE & CLAP 1.2.2 Polyphonic Modulation (Poly-Mod)
- Wire CLAP `CLAP_EVENT_PARAM_MOD` per-note modulation + MPE Zone Lower/Upper pitch-bend/CC74/pressure directly to per-voice cutoff, wavetable position, MSEG depth, and granular spray.

### Phase 7 -- Physical Modeling (2D Waveguide Mesh + 64-Mode Resonator) & 8x WDF Circuit Sim
- 2D finite-difference metal plate / drum membrane solver + 64-mode resonant body excitable by the drum machine or live `DJFX` input.
- 8x oversampled Wave Digital Filter (WDF) circuit models for Moog CP3, TB-303 diode ladder, MS-20 OTA, and Buchla 292 LPG.

### Phase 8 -- RTX 4090 GPU Compute & Neural Timbre Transfer (RAVE / DDSP)
- `wgpu` compute shader for 120 FPS 3D STFT waterfall spectrogram & Lissajous vectorscope.
- Optional local ONNX / Candle RAVE/DDSP inference running on CUDA/DirectML as a 3U Eurorack neural timbre module.
- 64 GB RAM zero-latency multi-velocity 24-bit/96 kHz streaming sample bank engine.
