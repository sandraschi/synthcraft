# Building SynthCraft By Hand: A 2024 Post-Mortem for Mortals

> *"We just wanted a simple softsynth. One oscillator and a filter."*
> -- Famous last words, right before `kitchen_sink.rs` compiled with 0 clippy warnings.

If you are considering building a comparable multi-engine modular synthesizer, native CLAP `1.2.2` DAW plugin, native VCV Rack 2 module, WebAssembly AudioWorklet, 16-stop Cathedral Pipe Organ, 60-tool CLI/MCP server, and 39-subpage handbook **by hand in 2024** -- pouring coffee, trawling 1971 Wave Digital Filter papers, and fighting MinGW/Win32 HWND lifecycles -- here is the exact measured bill.

---

## 1. The Measured Inventory (`v0.5.0`, Measured `2026-11-25`)

No vendored third-party crates live inside this repository; all 523 dependencies resolve cleanly via `Cargo.lock`. Every line below is first-party SynthCraft code:

| Crate / Area | Files | Lines | What You Have to Write |
|---|---:|---:|---|
| `synthcraft-core` | 42 | **18,057** | Pure-Rust DSP (`#![forbid(unsafe_code)]`): Dual VCO, ZDF Ladder, 64-Step Acid Seq, 8-Voice TR-808/909 Drums, 16-Point MSEG, Dual Morph Filter B, 24-Grain Cloud, 64-Partial Spectral, Multi-Core Genopatch, 6-Op Thru-Zero FM, 16-Voice Unison, OTT/IR/Shimmer/Tape FX, 8x8 2D FDTD Mesh, 8x WDF Circuits, Neural DDSP, 16-Band Vocoder, Chord Memory, 8-Scene XY Morph, Mastering Bus, 16-Stop Cathedral Pipe Organ + 5 Historical Temperaments, 129 Audio-Verified Presets, SF2/SFZ/SMF/WAV/SQLite |
| `synthcraft-app` | 21 | **10,692** | 15 Workbenches / 39 Subpages, GPU rotary knobs with mod rings, drooping 3U Eurorack catenary cables, 3D Waterfall Spectrogram, Lissajous Vectorscope, MPE Touch Pad, Pro Preset Browser, Drag-and-Drop Mod, Wavetable Frame Editor, Cathedral Organ Console, GrandOrgue Bach MIDI Importer, Contextual `? Guide` + `F1` Encyclopedia, 5 Skins, 4 Layouts, `egui_kittest` WCAG accessibility suite |
| `synthcraft-mcp` | 3 | **2,613** | 60 CLI subcommands + sandboxed MCP `2026-07-28` JSON-RPC 2.0 stdio server |
| `synthcraft-clap` | 6 | **2,567** | Native CLAP `1.2.2` instrument (`SynthCraft.clap`), embedded Win32/wgpu `CLAP_EXT_GUI`, MPE & Per-Note Poly-Mod (`clap-validator` green, live-verified in SoundCraft & REAPER) |
| `synthcraft-clap-host` | 1 | **778** | 16 HP 3U Eurorack module that hosts *external* `.clap` plugins with live CV parameter modulation |
| `synthcraft-ffi` | 2 | **522** | Stable C ABI (`libsynthcraft_ffi.a` + `include/synthcraft.h`) |
| `synthcraft-wasm` | 1 | **212** | ~190 KB WebAssembly AudioWorklet linear-memory engine |
| **Total First-Party Rust** | **76** | **35,441** | **0 clippy warnings (`-D warnings`), 108 passing tests** |
| `rack/` (VCV Rack 2 C++) | -- | **441** | Native VCV Rack 2.6.6 `plugin.dll` with context menus (live-loaded in VCV Rack Free 2.6.6) |
| Documentation (`*.md`) | -- | **1,489** | README, HELP, 15 human-readable workbench sub-READMEs (+ 115 KB interactive HTML handbook) |
| **Cargo Dependency Graph** | -- | **523 crates** | `eframe`/`egui`/`wgpu`, `cpal`, `midir`, `rusqlite`, `clap`, `serde`, `clap-sys`, etc. |

---

## 2. Realistic 2024 Human Team Estimate (By Hand)

Assume **three genuinely domain-competent senior engineers** working full-time in 2024 without agentic coding:
1. **Senior Audio DSP Engineer** -- oscillators, ZDF/WDF filters, 2D FDTD wave equations, FM, granular/spectral, DDSP, vocoder, pipe organ acoustics, lock-free real-time audio.
2. **Plugin / Systems / Platform Engineer** -- CLAP `1.2.2` C ABI, Win32/wgpu embedded plugin GUI, external CLAP hosting, VCV Rack 2 C++/MinGW/Rust `staticlib` cross-linking, WASM AudioWorklet, `cpal`/`midir`, SQLite migrations, NSIS + MCPB release engineering.
3. **UI/UX + Sound Design + Docs Engineer** -- custom GPU widgets, 15 workbenches / 39 subpages, 5 skins, 129 presets + browser, CLI/MCP, automated headless GUI testing, 15 workbench manuals, and interactive HTML site.

### The Calendar Bill
- **Working, tested, released `v0.5.0` equivalent**: **10 to 16 months calendar time** (**2.5 to 4.0 person-years**).
- **Vienna / Berlin Engineering Payroll (2024)**: roughly **EUR 350,000 -- EUR 500,000** (fully loaded salaries + overhead).
- **Where the months actually evaporate**:
  - **DSP literature & math prototyping (3--5 months)**: Nobody memorizes Fettweis's 1971 Wave Digital Filter scattering matrices, 2D FDTD Courant stability bounds, Werckmeister III cent offsets, or SF2/SFZ spec edge cases.
  - **Windows plugin ABI & toolchain purgatory (3--4 months)**: Embedding a `wgpu` surface inside a DAW's Win32 `HWND` without deadlocking the host UI thread; linking Rust `x86_64-pc-windows-gnu` staticlibs against VCV Rack SDK 2.6.6; satisfying every assertion in `clap-validator`.
  - **UI, presets, tests & documentation (3--5 months)**: 39 subpages, WCAG accessibility checks, 129 verified presets, 60 CLI/MCP tools, 108 automated tests, 15 manuals, and a self-contained HTML handbook.
  - **The 20-iteration integration tax (2--3 months)**: Hunting audio clicks, DC offsets, denormals, state migration bugs, and DAW quirks.

### One Honest Caveat
A 3-person human team spending 14 months would also spend a huge slice of that time doing golden-ear blind A/B listening tests against vintage hardware and QA across 10+ commercial DAWs on macOS, Linux, and Windows. On pure architecture, working DSP breadth, live CLAP/VCV/WASM/MCP integration, automated test coverage, and documentation, what took days on Goliath with an AI collaborator is legitimately a **1-year, 3-engineer studio project**.

*(Still want to type it all by hand? Better put the kettle on.)*
