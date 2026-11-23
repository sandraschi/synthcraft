# SynthCraft roadmap

Status: 2026-11-23. Estimates assume AI-assisted development (days, not weeks).

## Done

Live audio and MIDI, one shared `SynthRt` engine for live and offline, sampler (WAV, SFZ, SF2, loops, automatic sustain loops), pad oscillator, chorus, reverb, persistence v2, agent sandbox, free bank downloader, panel edits as `Command`s, 23 MCP tools, CI with cargo-deny.

## Done since

FFI shim (`synthcraft-ffi`), CLAP plug-in (`synthcraft-clap`, passes `clap-validator`) and a VCV Rack module (`rack/`, builds and links against the Rack SDK). Details and what is still unverified: `docs/PLUGINS.md`.

## Next, in the order I would do it

1. **Try both plug-ins in real hosts (hours).** A DAW for the CLAP, Rack for the module. Fix whatever turns up.
2. **AI2-style layering in the sampler (about 1 to 2 days).** Overlapping zones, one-shot attack zones plus looped sustain; see `docs/SAMPLES_AND_M1.md`.
3. **Release assets.** `SynthCraft.clap` as a plain file beside the exes, and a `.vcvplugin` per platform built in CI.
4. **Stereo engine output**, then the feature work below.

## VCV Rack module (design notes, built)

Feasible, and now built (see `docs/PLUGINS.md`). Rack plugins are C++ built against the Rack SDK; the plugin tutorial says they are written in C++11 and the SDK has a helper script that scaffolds a module. So the module is a small C++ wrapper that calls the Rust engine through the C ABI from the shim crate.

Design:

- Inputs: V/OCT (polyphonic, 1 V per octave, 0 V = C4 so MIDI note = 60 + 12 x volts), GATE (polyphonic), VEL (optional), CUTOFF CV, RESONANCE CV, MORPH CV, PWM CV.
- Outputs: LEFT, RIGHT (or one polyphonic out). Voices map to cable channels; a gate edge becomes note on, falling edge note off.
- Parameters: the same ones as `Command::SetPatchParam` and the patch commands (cutoff, resonance, ADSR, LFO, drive, master, PCM mix, X-morph, sampler mix, chorus, reverb).
- Sample rate: Rack can change it at any time; rebuild the engine on a sample rate change.
- Block size: Rack calls `process` per sample. Run the engine in blocks of 16 frames (about 0.3 ms at 48 kHz) and hand out samples from a small buffer.
- Banks: loading happens off the audio thread (as in the app), by path from the module's context menu.
- State: store the patch as JSON in the module's `dataToJson` so a Rack patch is self-contained, apart from sample bank paths.
- Tests: an offline harness that drives the C ABI with a recorded V/OCT and gate sequence and compares against `synthesize_audio_buffer`.

Licensing: Rack is GPLv3 with a non-commercial plugin exception. VCV's licensing page recommends GPLv3+ for free software, and a plugin that is free of charge may use any licence under the exception. SynthCraft is `MIT OR Apache-2.0`, which is GPL-compatible, so a free plugin is fine. Selling a Rack plugin needs a commercial licence from VCV. The Rack ABI is tied to the Rack major version, so build against the SDK of the Rack version you run. Not yet verified here: the exact Windows toolchain for the SDK (the Rack manual's build instructions apply) and whether VCV Library submission needs anything extra.

## Feature ideas (value for effort, highest first)

1. **Undo and redo.** Every edit is now a `Command`, so a command log gives undo almost for free. Plus A/B compare of two patch states.
2. **Preset browser with tags and search**, including your ambient pads, with a "randomize within a tag" button.
3. **Evolving ambient mode.** Slow random walks on filter, morph, pan and reverb send, seeded and reproducible; this is the quickest route to the M1 ambient feel.
4. **Modulation matrix.** Sources (LFOs, envelopes, velocity, mod wheel, random) to any `SetPatchParam` target with depth. Fits the command model.
5. **Polyphony and voice management UI.** Voice count, stealing mode, unison and detune spread, mono legato with glide.
6. **Arpeggiator and chord memory**, plus pattern chaining and a song mode for the sequencer.
7. **MIDI learn** and MPE (per-note pitch bend and pressure).
8. **More effects.** Stereo delay, a better chorus or ensemble, a drive stage before the filter, and a shimmer reverb for pads.
9. **Wavetable oscillator** (load single-cycle WAVs, morph between frames); the sampler's WAV loader already does most of the loading work.
10. **Multitrack stem export and real-time bounce.** Render each patch layer to its own WAV; capture live playing to WAV.
11. **MIDI file import and live record into the sequencer.**
12. **Microtonal tuning files (Scala `.scl`)** for the pitch table.
13. **Browser build.** The core crate has no UI or audio dependency, so a wasm build of the engine with a web front end, as moneycraft did, is possible; the CI wasm check currently finds no wasm crate to build, so this would need a small wasm wrapper crate.
14. **Gaps against the Craft rules:** translated UI text (rule 16), accessibility roles for knobs and pads (rule 21), stronger test assertions (rules 27 and 28).

## Open decisions for the owner

- Build the CLAP plugin and the VCV Rack module on one shared FFI shim (recommended), or only one of them first.
- Whether the real KORG M1 is purchased, and whether to invest in AI2 layering for bounced banks.
- Whether SynthCraft should also be offered as a free Rack plugin on the VCV Library.
