# Plugins: CLAP and VCV Rack

Status: 2026-11-23. SynthCraft runs as a CLAP instrument plug-in and as a VCV Rack module. Both drive the same engine as the desktop app (`SynthRt`), so a patch sounds the same everywhere.

## What exists

| Piece | Where | What it is | Verified |
|---|---|---|---|
| Plug-in engine | `crates/synthcraft-core/src/plugin.rs` | Safe `PluginEngine`: notes, parameters by id, block processing, JSON state, bank loading | unit tests |
| Parameter table | `crates/synthcraft-core/src/params.rs` | 25 parameters with stable ids, ranges, defaults; shared by every front end | unit tests |
| C ABI | `crates/synthcraft-ffi` | `libsynthcraft_ffi` (static, shared, rlib) and `include/synthcraft.h` | Rust ABI tests, C smoke test with gcc |
| CLAP plug-in | `crates/synthcraft-clap` | `synthcraft_clap.dll` to install as `SynthCraft.clap` | in-crate host simulation tests and `clap-validator` 0.4.1 (34 passed, 0 failed, 10 skipped) |
| VCV Rack module | `rack/` | C++ module over the C ABI | compiled and linked against the Rack SDK 2.6.4 on Linux; not yet loaded in a running Rack |

The CLAP plug-in calls the Rust `PluginEngine` directly. The Rack module goes through the C ABI. Both sit on the same `PluginEngine` and `params` table, so they cannot drift apart.

## Parameters

All 25 parameters are automatable in CLAP and have a knob in Rack. Ids never change; new parameters get new ids at the end of the table.

| Id | Key | Range | Default |
|---|---|---|---|
| 0x1001 | `vco2_detune_cents` | -100 to 100 | 12 |
| 0x1002 | `vco_mix` | 0 to 1 | 0.5 |
| 0x1003 | `fm_index` | 0 to 4 | 0.2 |
| 0x1004 | `cutoff_hz` | 35 to 16000 | 1200 |
| 0x1005 | `resonance_q` | 0.5 to 12 | 3.5 |
| 0x1006 | `env_mod_depth` | 0 to 1 | 0.65 |
| 0x1007 | `attack_ms` | 1 to 2000 | 8 |
| 0x1008 | `decay_ms` | 5 to 4000 | 180 |
| 0x1009 | `sustain_level` | 0 to 1 | 0.5 |
| 0x100A | `lfo_rate_hz` | 0.1 to 30 | 2 |
| 0x100B | `release_ms` | 5 to 8000 | 220 |
| 0x100C | `lfo_depth` | 0 to 1 | 0.2 |
| 0x100D | `drive_saturation` | 0 to 3 | 0 |
| 0x100E | `master_gain` | 0 to 1 | 0.7 |
| 0x100F | `pulse_width` | 0.05 to 0.95 | 0.5 |
| 0x1010 | `m1_layer_mix` | 0 to 1 | 0.55 |
| 0x1011 | `extreme_morph` | 0 to 1 | 0.55 |
| 0x1012 | `sampler_mix` | 0 to 1 | 0.8 |
| 0x1013 | `chorus_mix` | 0 to 1 | 0 |
| 0x1014 | `reverb_mix` | 0 to 1 | 0 |
| 0x1015 | `reverb_decay_s` | 0.2 to 20 | 3 |
| 0x1016 | `reverb_damping` | 0 to 0.95 | 0.5 |
| 0x1017 | `stereo_spread` | 0 to 1 | 0.3 |
| 0x1018 | `ambient_depth` | 0 to 1 | 0 |
| 0x1019 | `ambient_rate_hz` | 0.01 to 0.5 | 0.05 |

The source of truth is `params.rs`. Waveform choice, filter mode, the PCM program and the sequencer steps are part of the saved patch but have no host parameter yet.

## CLAP plug-in

### Build and install (Windows)

```
cargo build --release -p synthcraft-clap
just clap-install
```

`just clap-install` copies `target\release\synthcraft_clap.dll` to `%LOCALAPPDATA%\Programs\Common\CLAP\SynthCraft.clap`, a per-user folder CLAP hosts scan. To install for all users copy the file to `%COMMONPROGRAMFILES%\CLAP\` from an elevated shell. A `.clap` file is the DLL under a different name; no installer is needed.

### What it supports

- Instrument with one stereo output and one note input (CLAP note events and raw MIDI: note on and off, mod wheel CC1, pitch bend of plus or minus 2 semitones, all notes off).
- Sample-accurate event handling: the block is split at each event time.
- 25 automatable parameters with value text and parsing.
- State as the full patch JSON (a few KB), including the sample bank path. Loading a state tells the host to rescan parameter values, as the CLAP specification requires.
- Safe on the audio thread: `process` never blocks. If the main thread holds the engine (during a state load) the block is silent.

### Sample banks in the plug-in

There is no GUI yet, so there is no file dialog. Two ways to get a bank in:

1. Set the environment variable `SYNTHCRAFT_BANK` to an `.sfz`, `.sf2`, `.wav` or folder path before starting the host. It loads when the plug-in is activated, unless the restored state already has a bank.
2. Build the sound in the desktop app, then copy the bank path into a patch; the state keeps it.

Fetch free banks with `synthcraft-mcp fetch-bank <id>` (see `docs/SAMPLES_AND_M1.md`).

### Verified, and what is not

`clap-validator` 0.4.1 reports 44 tests run: 34 passed, 0 failed, 10 skipped. The skipped tests are for features the plug-in does not have (GUI, preset discovery, tail, voice info and similar). Its first run found one real defect, a missing parameter-rescan request after a state load; that is fixed.

Not verified: loading in a real DAW. `clap-validator` is strict about the ABI but it is not Bitwig or Reaper. Do that once and report problems.

Known limits: mono engine (both channels carry the same signal), no GUI, no MPE, no per-note modulation, no preset discovery, no sequencer sync to host transport (the internal sequencer is not exposed to the host).

## VCV Rack module

### Build

Needs the Rack SDK matching the Rack version you run (a free Rack 2 download and its SDK from vcvrack.com), a C++ toolchain, `jq`, and Rust.

```
cargo build --release -p synthcraft-ffi
cd rack
make RACK_DIR=/path/to/Rack-SDK
make install RACK_DIR=/path/to/Rack-SDK
```

On Linux this is verified (SDK 2.6.4, GCC, `plugin.so` links with no undefined `sc_` symbols and exports `init`). macOS and Windows use the same Makefile with the system libraries Rust needs listed per platform; they are untested. On Windows build the engine for the MinGW target Rack's toolchain uses and pass its path as `FFI_LIB`. `rack/plugin.json` carries the plug-in manifest.

### Module

Name `SynthCraft`, 20 HP. 25 knobs (one per parameter) and these ports:

| Port | Type | Meaning |
|---|---|---|
| V/OCT | input, poly | pitch, 0 V = C4 (MIDI 60), 1 V per octave, rounded to the nearest note |
| GATE | input, poly | gate, high at 2 V, low below 0.1 V; each channel is one voice |
| VEL | input, poly | velocity 0 to 10 V; unconnected means 0.8 |
| CUTOFF | input | cutoff CV, 1 V per octave, channel 1 |
| RESO, MORPH, PWM | input | 0 to 10 V spans the whole parameter range, channel 1 |
| L, R | output | audio at plus or minus 5 V; identical today |

Right-click menu: maximum voices (1, 2, 4, 8, 16), load a sample bank (`.sfz`, `.sf2`, `.wav`), remove the bank. The patch (including the bank path) is stored in the Rack patch, so a saved Rack patch is self-contained apart from the bank files.

### Behaviour worth knowing

- The engine renders in blocks of 16 samples, so a gate can start the sound up to 16 samples late (about 0.3 ms at 48 kHz).
- A pitch change while a gate stays high retriggers the voice on the new note (this is how a sequencer with a slide-less pitch bus behaves).
- Loading a bank or state happens on the UI thread and holds the engine, so the module is silent for that moment (hundreds of milliseconds for a large bank).
- Changing Rack's sample rate cuts sounding notes.
- The internal sequencer, waveform choice and filter mode are not on the panel yet.

### Licence

SynthCraft is `MIT OR Apache-2.0`. Rack is GPLv3 with a non-commercial plug-in exception: a plug-in offered free of charge may use any licence. Selling it needs a commercial licence from VCV. `plugin.json` says `MIT`. The Rack ABI follows the Rack major version, so rebuild against the SDK of the Rack you run.

Submitting to the VCV Library has its own requirements (panel review, naming); none of that is done.

## C ABI

`crates/synthcraft-ffi/include/synthcraft.h` is the contract; `SC_ABI_VERSION` (currently 1) changes with any incompatible change. Rules:

- One engine is used from one thread at a time.
- Real-time safe (no allocation, no locks): `sc_process`, `sc_note_on`, `sc_note_off`, `sc_all_notes_off`, `sc_pitch_bend`, `sc_mod_wheel`, `sc_set_param`, `sc_get_param`, `sc_set_max_voices`, `sc_active_voices`.
- Not real-time safe (allocate or read files): `sc_create`, `sc_destroy`, `sc_set_sample_rate`, `sc_load_bank`, `sc_clear_bank`, `sc_state_json`, `sc_load_state_json`. Never call them while another thread is inside `sc_process` on the same engine.
- Strings are UTF-8 and NUL-terminated. Free `sc_state_json` results with `sc_free_string`. `sc_last_error` belongs to the engine.
- Null handles and buffers are ignored, and a panic never crosses the boundary.

Tests: `crates/synthcraft-ffi/tests/abi.rs` (also checks the header lists every function) and `tests/c/smoke.c` (plain C; the build line is in the file).

## Unsafe code policy

The workspace forbids `unsafe_code`. Two crates opt out because a C ABI cannot be written without it: `synthcraft-ffi` and `synthcraft-clap`. Both deny undocumented unsafe blocks and missing safety docs, so every block has a `// SAFETY:` comment. The engine, `PluginEngine` and the parameter table stay `forbid(unsafe_code)`.

## What to do next

1. Load `SynthCraft.clap` in a real host and a built `plugin.so`/`.dll` in a real Rack and report anything odd. These two checks are the biggest remaining unknowns.
2. Ship `SynthCraft.clap` as a plain release asset beside the exes (no zip), and the Rack plugin as a `.vcvplugin` per platform from CI.
3. A GUI for the CLAP plug-in (bank picker, patch browser) or a host-side bank parameter.
4. Stereo engine output (the chorus and reverb are mono today), then MPE and a modulation matrix.
