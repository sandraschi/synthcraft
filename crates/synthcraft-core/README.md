# `synthcraft-core` (L1 Pure-Rust Modular DSP Engine)

100% Safe Rust (`#![forbid(unsafe_code)]`) synthesizer, sequencer, and mastering engine for **SynthCraft**:

- Dual VCO (10 waveforms + wavetable), 4-pole ladder VCF, dual morphing Filter B (Formant/Comb/MS-20), 6-op thru-zero FM, 16-voice hyper-unison, 24-grain granular cloud, 64-partial spectral additive, 8x8 2D FDTD physical mesh, 8x oversampled WDF analog circuits, neural DDSP timbre transfer, and 16-band vocoder.
- 64-step acid sequencer, 8-voice TR-808/909 drum machine, 16-point MSEG, 8-route mod matrix, arpeggiator, Scala `.scl` microtuning, evolutionary Genopatch matcher, and mastering bus.
- Bundled SQLite persistence (`%LOCALAPPDATA%\synthcraft\synthcraft.db`), RIFF WAV / DJCraft stem forge, and agent file sandbox (`--automation-read-root` / `--automation-write-root`).
