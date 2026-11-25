# Workbench 4: Demo Showcase & Audition

Think of this workbench as the showroom floor of a vintage-meets-futuristic synthesizer museum. Every card configures an entire sound design recipe -- oscillators, filter, PCM layer, Extreme DSP, modulation, and musical pattern -- and lets you hear and inspect it with a single click.

---

## Subpage 4.1: 1-Click PCM / Acid / West-Coast Demos

### What It Does (Plain English)
When you open SynthCraft for the first time (or want quick inspiration for a track), you do not need to dial in 40 knobs by hand. The Demo Showcase provides 6 curated one-click sound demonstrations spanning 1980s Chicago acid, 1990s Ibiza house, West-Coast Buchla percussion, physical plucked strings, modal marimbas, and modern drum & bass Reese subs. Clicking a card loads the patch, runs a short synthesis render, and displays live loudness and spectral metrics.

### Controls & Workflow
- **Audition Any Showcase**: Click a showcase card to immediately configure SynthCraft's engine and hear the characteristic sound in context.
- **Inspect Under the Hood**: After clicking a demo card, switch to Workbench 1 (Modular Rack), Workbench 2 (PCM & Extreme DSP), or Workbench 5 (Sequencer) to see exactly how the sound was built and start tweaking it into your own signature patch.
- **Spectral Feedback**: Each demo reports its RMS loudness, Peak dBFS, and detected fundamental frequency so you can compare headroom and tonal balance across genres.

### Provenance, Hardware Lineage & History
- **Hardware Demo Modes (Roland D-50, Korg M1, Yamaha DX7, 1983-1988)**: Classic flagship synthesizers always shipped with a dedicated "Demo" button so musicians walking into a music store could instantly hear the full range of the instrument -- from crystalline bells to punchy basslines and sweeping pads -- before touching a single edit menu.

### Famous Tracks & Iconic Usage
- **Chicago Acid House (`vienna_acid_303`)**: Echoes Josh Wink's *"Higher State of Consciousness"* (1995) and Fatboy Slim's *"Everybody Needs a 303"* (1996).
- **1990s Ibiza House Piano & Organ (`ibiza_rave_piano`)**: Channels Frankie Knuckles, Robert Miles (*"Children"*, 1996), and Robin S. (*"Show Me Love"*, 1993).
- **West-Coast Modular (`west_coast_wavefolder`)**: Pays tribute to Suzanne Ciani (*"Seven Waves"*, 1982) and Alessandro Cortini's Buchla performances.

### CLI & MCP Automation
```powershell
synthcraft-mcp run-demo --id vienna_acid_303 --json
synthcraft-mcp run-demo --id west_coast_buchla --json
```
