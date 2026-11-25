# Workbench 10: Studio Settings & Prefs

Press `Ctrl+,` from anywhere in SynthCraft to configure your studio environment across 6 clean, organized preference tabs -- from low-latency audio drivers and MPE keyboards to hardware faceplates, vintage DAC emulation, DAW plugin paths, and SQLite database hygiene.

---

## Subpage 10.1: Audio Driver & DSP Oversampling

### What It Does (Plain English)
Controls how SynthCraft sends sound to your speakers or studio audio interface (`cpal` / Windows WASAPI), including sample rate (`44.1 kHz` or `48 kHz`), buffer size (lower buffers mean faster keyboard response; larger buffers mean rock-solid stability under heavy CPU load), and internal DSP oversampling for ultra-clean filter saturation and wavefolding without harsh digital aliasing.

### Controls & Workflow
- **Audio Output Device**: Choose your studio interface, headphones, or speakers.
- **Sample Rate & Buffer**: `48,000 Hz` is recommended for modern DAW, SoundCraft, and DJCraft workflows.
- **Oversampling**: Increase oversampling when pushing analog filter drive, Wave Digital Filter circuits, or extreme FM modulation.

### Provenance, Hardware Lineage & History
- **Oversampling & Anti-Aliasing in Virtual Analog (Clavia Nord Lead 1995, Access Virus 1997, u-he Diva 2011)**: Early digital synths suffered from metallic "aliasing" artifacts when oscillators or distortion folded frequencies above half the sample rate. Modern CPUs let SynthCraft run internal analog circuit models at up to **8x oversampling** (`384 kHz` internal rate) for velvet-smooth analog highs.

### Famous Tracks & Iconic Usage
- **Daft Punk -- "Discovery" (2001)** & **The Prodigy -- "The Fat of the Land" (1997)**: Celebrated the punch of early virtual-analog engines pushed hard into saturation.

### CLI & MCP Automation
```powershell
synthcraft-mcp status --json
```

---

## Subpage 10.2: MIDI Hardware, Clock & MPE

### What It Does (Plain English)
Configures how SynthCraft listens to external hardware keyboards, drum pads, master clocks, and expressive MPE controllers (like the Expressive E Osmose, ROLI Seaboard, or LinnStrument).

### Controls & Workflow
- **MIDI Input Port & Channel**: Select a specific USB/DIN MIDI port or leave on Omni.
- **MIDI Clock Sync**: Choose Internal clock, External MIDI Clock, or `DJLink` sync from DJCraft.
- **MPE Zone Configuration**: Enable MIDI Polyphonic Expression so per-finger pitch bends, vertical slide (`CC 74`), and channel pressure map independently to every held voice.

### Provenance, Hardware Lineage & History
- **MPE -- MIDI Polyphonic Expression (Roger Linn, ROLI, Apple, 2018)**: Extended 1983's MIDI 1.0 by giving each held finger its own MIDI channel, unlocking violin-like vibrato, polyphonic pitch bends, and independent per-note timbre swells on modern keyboards.

### Famous Tracks & Iconic Usage
- **Vangelis -- "Blade Runner Blues" (1982)** (Yamaha CS-80 polyphonic aftertouch -- the spiritual grandfather of MPE) & **Jordan Rudess / A.R. Rahman** (modern MPE Seaboard & Osmose performances).

### CLI & MCP Automation
```powershell
synthcraft-mcp set-patch-param --param bpm --value 128
```

---

## Subpage 10.3: Skins, Layouts & Knob Physics

### What It Does (Plain English)
Tailors how SynthCraft looks and feels under your mouse and eyes. Switch between **5 hardware skins** (including high-contrast **Daylight Studio** light mode and **Obsidian VCV** dark mode), **4 studio layouts**, and realistic rotary knob drag physics.

### Controls & Workflow
- **5 Hardware Skins**: Choose between dark Eurorack obsidian, warm vintage analog panels, and crisp WCAG AAA Daylight Studio light mode.
- **Knob Drag Sensitivity & `Ctrl+MouseWheel` Zoom**: Fine-tune vertical drag speed and zoom the entire interface smoothly to fit laptops or 4K studio monitors.
- **Language Toggle (`EN` / `DE`)**: Switch all workbench and subpage labels cleanly between English and German.

### Provenance, Hardware Lineage & History
- **Hardware Industrial Design (Dieter Rams / Braun, Teenage Engineering, Moog)**: Thoughtful panel contrast, clear knob rings, and uncluttered grouping reduce ear and eye fatigue during long late-night studio sessions.

### Famous Tracks & Iconic Usage
- **Studio Ergonomics Across Every Era**: From the walnut wood cheeks of the Minimoog to the clean anodized aluminum of Eurorack skiffs, tactile visual clarity directly inspires bolder musical choices.

### CLI & MCP Automation
*(UI preference persisted locally in studio settings.)*

---

## Subpage 10.4: PCM Workstation DAC & Extreme DSP Tuning

### What It Does (Plain English)
Fine-tunes the vintage digital character of Workbench 2's 1990 PCM Workstation (toggle 1988 32 kHz / 12-bit DAC warmth, sample interpolation quality, and bank directory) as well as CPU/quality parameters for the 6 Extreme DSP algorithms.

### Controls & Workflow
- **DAC Emulation**: Switch between pristine 48 kHz floating-point playback and gritty 1988 32 kHz / 12-bit converter character.
- **Sample Bank Folder**: Points to `Documents/SynthCraft/Banks` for your `.sfz`, `.sf2`, and `.wav` packs.

### Provenance, Hardware Lineage & History
- **12-Bit / 32 kHz Samplers (E-mu SP-1200 1987, Akai MPC60 1988, Korg M1 1988)**: Golden-era hip-hop and house producers noticed that pitched-down 12-bit samples had a crunchy, mid-forward punch that sounded better in a club than sterile hi-fi recordings.

### Famous Tracks & Iconic Usage
- **Pete Rock & CL Smooth -- "They Reminisce Over You (T.R.O.Y.)" (1992)** & **Wu-Tang Clan -- "C.R.E.A.M." (1993)**: Immortalized the warm crunch of 12-bit vintage sampler converters.

### CLI & MCP Automation
```powershell
synthcraft-mcp list-banks
```

---

## Subpage 10.5: CLAP, SoundCraft & DJCraft Paths

### What It Does (Plain English)
Manages the filesystem paths that connect SynthCraft to the rest of your studio: your system CLAP plugin folder (`%LOCALAPPDATA%\Programs\Common\CLAP`), external CLAP plugin scan paths for the 3U CLAP Host module, SoundCraft session folders, and DJCraft stem export targets.

### Controls & Workflow
- **Scan External CLAP Plugins**: Discover third-party `.clap` synths and effects installed on your PC so you can load them straight into Workbench 1's 3U `CLAP HOST` module.
- **Verify `SynthCraft.clap` & VCV Rack 2**: Check install locations for both `SynthCraft.clap` and the native VCV Rack 2 `plugin.dll`.

### Provenance, Hardware Lineage & History
- **Open Studio Interoperability**: Keeps all plugin and stem paths transparent, user-controlled, and free from proprietary cloud lock-in.

### Famous Tracks & Iconic Usage
- **Modern Hybrid DAW + Modular Workflows**: Lets producers bounce seamlessly between standalone sketching, VCV Rack modular patching, SoundCraft/REAPER mixing, and DJCraft live performance.

### CLI & MCP Automation
```powershell
synthcraft-mcp scan-clap-plugins --json
synthcraft-mcp get-clap-host-status --json
```

---

## Subpage 10.6: AI Assistant, MCP & SQLite DB

### What It Does (Plain English)
Displays the active SQLite database path (`%LOCALAPPDATA%\synthcraft\synthcraft.db`), schema version, automatic pre-migration backup status, and the security sandbox rules (`--automation-read-root` and `--automation-write-root`) that keep AI MCP tools safely confined to approved folders.

### Controls & Workflow
- **Inspect Database Health**: All patch edits save off the UI thread so audio never glitches. Corrupt rows are quarantined automatically rather than crashing startup.
- **MCP Sandbox Verification**: Verify which read/write folders are exposed to external AI agents.

### Provenance, Hardware Lineage & History
- **SQLite (D. Richard Hipp, 2000)**: The world's most reliable embedded single-file database -- ensuring your patch library is fast, portable, ACID-safe, and readable forever without cloud servers.

### Famous Tracks & Iconic Usage
- **Every Working Studio's Most Important Feature**: Never losing a 3 AM patch breakthrough to a crash or format change.

### CLI & MCP Automation
```powershell
synthcraft-mcp status --json
synthcraft-mcp export-wasm-bundle --out-dir D:\AudioOut\wasm_bundle
```
