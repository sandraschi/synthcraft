# Storytold ArtCraft Beta Submission Checklist -- SynthCraft

- [x] 3-tier Pure-Rust workspace (`synthcraft-core`, `synthcraft-mcp`, `synthcraft-app`)
- [x] `#![forbid(unsafe_code)]` and `clippy::deny(unwrap_used, expect_used, panic, todo, unimplemented)`
- [x] 6 integrated workbenches in hierarchical `SubApp` sidebar
- [x] CLAP 1.2.2 plugin bridge for SoundCraft + BPM/Camelot WAV stem exporter for DJCraft
- [x] Bundled SQLite persistence (`rusqlite` 0.41, schema v2, quarantine and backup)
- [x] 23 MCP `2026-07-28` stdio tools + 26 CLI subcommands, with a required read/write sandbox for agents
- [x] Headless `egui_kittest` verification across every sub-app workbench and help tab
- [ ] Attach Dark/Light studio screenshots before public submission to `https://getartcraft.com/beta`
- [x] Live audio (cpal) and MIDI (midir), sampler (WAV/SFZ/SF2), pad oscillator, chorus and reverb
- [ ] CLAP plugin: only the descriptor JSON exists; a loadable CLAP binary (separate cdylib crate) is not built
- [ ] Route every panel edit through `Command` (rule 14)
