# Attribution & Third-Party Notices

SynthCraft is dual-licensed under `MIT OR Apache-2.0` and authored by Sandra Schipal (`sandraschi`).

## Architectural Lineage

- **Storytold ArtCraft Pattern** (`storytold/artcraft` by Brandon Thomas): 3-tier Pure-Rust architecture (`L1 core` -> `L2 CLI + MCP stdio` -> `L3 eframe/egui desktop studio`). All code in SynthCraft is original and independently written.
- **CLAP (Clever Audio Plug-in ABI)**: Open-source MIT audio plugin specification by u-he and Bitwig (`https://github.com/free-audio/clap`).

## Rust Crates

- `egui` / `eframe` / `egui_kittest` 0.37 -- Emil Ernerfeldt & Embark Studios (`MIT OR Apache-2.0`)
- `rusqlite` 0.40 (`bundled` SQLite) -- The rusqlite Developers (`MIT`)
- `clap` 4.5 -- clap Contributors (`MIT OR Apache-2.0`)
- `serde` / `serde_json` 1.0 -- Erick Tryzelaar & David Tolnay (`MIT OR Apache-2.0`)
- `thiserror` 2.0 -- David Tolnay (`MIT OR Apache-2.0`)

## Bundled Assets

- `assets/synth.svg` -- Original vector modular synthesizer waveform & filter icon authored for SynthCraft (`CC0-1.0 / MIT`).
