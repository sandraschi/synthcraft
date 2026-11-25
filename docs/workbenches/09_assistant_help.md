# Workbench 9: Assistant & Help

SynthCraft is designed to be explored both by hand and alongside an AI sound-design collaborator. This workbench pairs a local-first Patch Co-Designer chat panel with the full, searchable **37-Subpage Interactive Encyclopedia** you are reading right now.

---

## Subpage 9.1: Chat

### What It Does (Plain English)
A local-first studio assistant panel for brainstorming sound design recipes, checking which MCP tools control a specific engine, and coordinating patch edits with external AI coding/music agents connected to SynthCraft's `60`-tool MCP `2026-07-28` server.

### Controls & Workflow
- **Ask About Patches or Routing**: Type natural-language sound design questions ("How do I dial in a warm 1980s Juno chord pad with subtle chorus?" or "Which MCP tool loads the Berlin Techno 808/909 kit?").
- **Agent Automation via MCP**: Connect your IDE or AI agent to `synthcraft-mcp --mcp --automation-read-root <DIR> --automation-write-root <DIR>` to let the agent sculpt patches, run Genopatch audio matching, render `.wav` previews, and inspect spectrum data safely inside your sandboxed folders.

### Provenance, Hardware Lineage & History
- **Model Context Protocol (MCP, 2024-2026)**: Standardized how AI assistants safely inspect and control local desktop tools using structured JSON-RPC schemas and explicit filesystem sandboxes -- turning SynthCraft into a first-class collaborative instrument for both human hands and AI co-producers.

### Famous Tracks & Iconic Usage
- **Holly Herndon -- "PROTO" (2019) & Brian Eno -- "Generative Music 1" (1996)**: Landmark milestones in human-plus-machine musical collaboration, where algorithms act as creative studio partners rather than replacements for the artist.

### CLI & MCP Automation
```powershell
synthcraft-mcp --mcp --automation-read-root D:\AudioIn --automation-write-root D:\AudioOut
```

---

## Subpage 9.2: Help (F1)

### What It Does (Plain English)
Press **`F1`** from anywhere in SynthCraft -- or click the **`? Subpage Guide`** button in the top studio bar -- to open human-readable documentation tailored directly to the exact workbench and subpage you are using. No hunting through external PDFs or decoding walls of acronyms: every engine explains what it does in plain language, how to use its controls, which vintage hardware or modern synth inspired it, and which famous records made that sound legendary.

### Controls & Workflow
- **Contextual `? Guide` Button**: On any of the 37 subpages, click `? Guide` (or press `F1`) to jump straight to that subpage's dedicated handbook.
- **Live Search**: Filter across all 14 workbenches and 37 subpages by keyword (try searching `303`, `Daft Punk`, `Buchla`, `vocoder`, `OTT`, or `MPE`).
- **Quick Reference Tabs**: Switch between the **37-Subpage Encyclopedia**, **Get Started**, **60-Tool CLI & MCP Reference**, and **Technical Architecture**.
- **Keyboard Shortcuts**: `F1` (Help), `Ctrl+K` (Command Palette), `Ctrl+,` (Settings), `Ctrl+Z` / `Ctrl+Y` (64-deep Undo/Redo), `Ctrl+MouseWheel` (UI Zoom).

### Provenance, Hardware Lineage & History
- **Printed Hardware Owner's Manuals (Roland, Moog, ARP 1970s-1980s)**: Classic Roland and ARP manuals were famous for teaching synthesis from first principles with clear diagrams and musical patch sheets rather than dry engineering specs. SynthCraft brings that same welcoming spirit directly inside the app and as a browsable interactive HTML website (`docs/site/index.html`).

### Famous Tracks & Iconic Usage
- **Every Bedroom Producer Who Learned From a Synth Manual**: From Aphex Twin modifying his own circuits to Daft Punk reading Roland manuals cover-to-cover, clear documentation turns intimidating hardware into lifelong instruments.

### CLI & MCP Automation
```powershell
synthcraft-mcp status --json
synthcraft-mcp summary --json
```
