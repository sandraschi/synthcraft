## SynthCraft v0.3.0 (2026-11-25)

### Added
- **Multi-row scrollable 3U Eurorack skiff & Module Browser:** dynamic multi-row rack persisted to SQLite, categorized `+ Add Module` browser, right-click move/bypass/remove, interactive rotary knobs, clickable mode labels, and cross-row draggable catenary patch cables wired into live `SynthRt` DSP routing.
- **6 new playable 3U DSP modules:** Wavefolder + LPG, Sample & Hold + Slew, Stereo Ping-Pong Delay, Euclidean Clock, Dual Attenuverter, and Inline 3U Oscilloscope.
- **12 curated pre-patched modular Rack Rigs:** 1-click `Rack Rig` picker in the 3U toolbar (Berlin School Echo, West-Coast Wavefolder Pluck, Acid Squelch 303, Random S&H Computer Lab, Drifting Ambient Cathedral, Industrial Filter Bypass, Dub Techno Chord Echo, FM Metallic Bells, Sub-Bass Reese & Attenuverter, 1990 PCM Shimmer, Polyrhythmic Glitch Lab, and Clean Studio Reference).
- **Native CLAP 1.2.2 instrument plug-in (`SynthCraft.clap`) with `CLAP_EXT_GUI`:** Windows `egui-wgpu` embedded hardware UI with 25 automatable rotary knobs (DAW automation gestures), live stereo peak meters and oscilloscope, `< [Patch] >` selector, VCO1/VCF selectors, Sample Bank picker, and 1-octave audition keyboard.
- **VCV Rack 2 module (`rack/`) & C ABI (`synthcraft-ffi`):** polyphonic V/Oct, gate, velocity, 4 CV inputs, 25 knobs, bank loading, and Rack patch serialization.
- **Real-time audio (`cpal`) & MIDI (`midir`):** single `SynthRt` engine shared across live playback, offline WAV rendering, CLAP, and VCV Rack; true stereo output (voice spread, stereo chorus, decorrelated stereo reverb) and seeded ambient drift mode.
- **Sampler & CC0 bank downloader:** WAV, SFZ, and SF2 loading with layered/one-shot zones, Hermite interpolation, automatic crossfaded sustain loops, and checksum-verified `fetch-bank` / `list-banks`.
- **35 MCP `2026-07-28` stdio tools & 38 CLI subcommands** with mandatory `--automation-read-root` / `--automation-write-root` sandbox.

### Changed
- SQLite persistence rewritten (schema v2, backup before migration, quarantine for corrupt rows, debounced off-UI-thread saves, `rusqlite` 0.41).
- Always-visible `< [Patch] >` selector added across all workbenches in the desktop studio.
- Offline render limit raised to 5 minutes; trademarked product and song names removed.

### Fixed
- `SubOctave` DC offset and double pitch bend on sampler playback.
