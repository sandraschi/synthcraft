# Attribution & Third-Party Notices

SynthCraft is dual-licensed under `MIT OR Apache-2.0` and authored by Sandra Schipal (`sandraschi`).

## Architectural Lineage

- **Storytold ArtCraft Pattern** (`storytold/artcraft` by Brandon Thomas): 3-tier Pure-Rust architecture (`L1 core` -> `L2 CLI + MCP stdio` -> `L3 eframe/egui desktop studio`). All code in SynthCraft is original and independently written.
- **CLAP (Clever Audio Plug-in ABI)**: Open-source MIT audio plugin specification by u-he and Bitwig (`https://github.com/free-audio/clap`).

## Rust Crates

- `egui` / `eframe` / `egui_kittest` 0.37 -- Emil Ernerfeldt & Embark Studios (`MIT OR Apache-2.0`)
- `rusqlite` 0.41 (`bundled` SQLite) -- The rusqlite Developers (`MIT`)
- `clap-sys` 0.5 -- Micah Johnston and Robbert van der Helm (`MIT OR Apache-2.0`), raw bindings for the CLAP plug-in API
- `clap` 4.5 -- clap Contributors (`MIT OR Apache-2.0`)
- `serde` / `serde_json` 1.0 -- Erick Tryzelaar & David Tolnay (`MIT OR Apache-2.0`)
- `thiserror` 2.0 -- David Tolnay (`MIT OR Apache-2.0`)

## Bundled Assets

- `assets/synth.svg` -- Original vector modular synthesizer waveform & filter icon authored for SynthCraft (`CC0-1.0 / MIT`).
- `rack/res/SynthCraft.svg` -- Original plain panel background for the VCV Rack module, authored for SynthCraft by Sandra Schipal (`MIT OR Apache-2.0`).

## Downloadable Sample Banks (not stored in this repository)

`synthcraft-mcp fetch-bank <id>` downloads packs listed in `banks/manifest.json` (URL pinned to a commit, SHA-256 per file) into the user's Banks folder and writes an `ATTRIBUTION.txt` beside the samples.

- `vsco2ce-violin-section-sus`, `vsco2ce-cello-section-sus`, `vsco2ce-concert-harp`, `vsco2ce-glockenspiel`, `vsco2ce-marimba`, `vsco2ce-flute-susvib` -- VSCO 2 Community Edition by Versilian Studios LLC (`https://github.com/sgossner/VSCO-2-CE`, `CC0-1.0`).
- `vcsl-cembalo-harpsichord`, `vcsl-cembalo-lute-stop`, `vcsl-pipe-organ-loud`, `vcsl-pipe-organ-pedal` -- Versilian Community Sample Library (VCSL) by Versilian Studios LLC (`https://github.com/sgossner/VCSL`, `CC0-1.0`). English Cembalo / Harpsichord (Normal & Lute stops) and Cathedral Pipe Organ (Full Principal/Mixture & 16' Pedal).
