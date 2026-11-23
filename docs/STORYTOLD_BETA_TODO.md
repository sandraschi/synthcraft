# Storytold ArtCraft Beta Submission Checklist -- SynthCraft

- [x] 3-tier Pure-Rust workspace (`synthcraft-core`, `synthcraft-mcp`, `synthcraft-app`)
- [x] `#![forbid(unsafe_code)]` and `clippy::deny(unwrap_used, expect_used, panic, todo, unimplemented)`
- [x] 6 integrated workbenches in hierarchical `SubApp` sidebar
- [x] CLAP 1.2.2 plugin bridge for SoundCraft + BPM/Camelot WAV stem exporter for DJCraft
- [x] Bundled SQLite persistence (`rusqlite` 0.40)
- [x] 14 MCP `2026-07-28` stdio tools + 14 CLI subcommands
- [x] Headless `egui_kittest` verification across every sub-app workbench and help tab
- [ ] Attach Dark/Light studio screenshots before public submission to `https://getartcraft.com/beta`
