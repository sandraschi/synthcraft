# Workbench 15: Cathedral Pipe Organ Console & Bach MIDI Depot

The pipe organ is the **original additive synthesizer** -- invented more than 2,000 years ago and perfected in the cathedrals of Germany, Austria, and France. This workbench brings a complete 3-division *Werkprinzip* pipe organ console (16 authentic stop ranks, couplers, Swell box shutters, Crescendo roller, wind-chest chiff, bellows wind sag, Tremulant, and 5 historical temperaments) together with a direct **GrandOrgue / J.S. Bach MIDI Depot** importer.

---

## Subpage 15.1: Cathedral Organ Console & Stop Jambs

### What It Does (Plain English)
Instead of oscillators and cutoff knobs, a pipe organ builds sound by pulling out **Stop Drawknobs** -- each stop adds an entire rank of pipes tuned to a specific footage (`16'` = one octave below written pitch, `8'` = unison pitch, `4'` = one octave up, `2 2/3'` = musical twelfth, `2'` = two octaves up, `Mixtur IV` = shimmering Baroque harmonic chorus). SynthCraft models 16 classic stops across three divisions:
- **I. Hauptwerk (Great Manual)**: `Bourdon 16'`, `Principal 8'`, `Rohrflote 8'`, `Oktave 4'`, `Quinte 2 2/3'`, `Superoktave 2'`, `Mixtur IV`, `Trompete 8'`.
- **II. Schwellwerk (Swell Manual -- behind wooden Venetian shutters)**: `Gedackt 8'`, `Salicional 8'`, `Voix Celeste 8'` (tuned `+13.5 cents` sharp for heavenly slow cathedral beating), `Blockflote 4'`, `Oboe 8'`.
- **Pedalwerk (Pedalboard)**: `Subbass 16'`, `Posaune 16'`, `Principalbass 8'`.

### Controls & Workflow
- **Pull Stops or Hit a Thumb Piston**: Click any ivory drawknob to pull/push individual ranks, or click one of the **8 Combination Thumb Pistons** (`pp Vespers Flute`, `p Romantic Voix Celeste`, `mf Baroque Chorale`, `f Silbermann Pleno`, `ff Cavaille-Coll Romantic`, `fff Bach Toccata Tutti`, `Solo Trompete Fanfare`, `General Cancel`).
- **Engage Stephansdom Cathedral IR**: Click `Engage Stephansdom Cathedral IR` to wrap your organ pipes in soaring gothic stone reverberation immediately.
- **Expression, Wind & Historical Temperament**: Sweep the **Swell Box Shutters** and **Crescendo Walze** (`pp -> Tutti`), dial **Wind-Chest Chiff** (initial air-edge transient when a pallet valve opens) and **Bellows Wind Pressure Sag**, engage the **Tremulant**, and switch between **Werckmeister III (1691 -- J.S. Bach's tuning)**, **Kirnberger III**, **Silbermann 1/6-Comma Meantone**, **Quarter-Comma Meantone**, and **12-TET**.

### Provenance, Hardware Lineage & History
- **Ctesibius of Alexandria (c. 250 BC)**: Invented the *Hydraulis* (water organ), the earliest keyboard instrument in human history.
- **Arp Schnitger (1648-1719) & Gottfried Silbermann (1683-1753)**: Defined the German Baroque *Werkprinzip* organ architecture (clear, singing Principal choruses, sparkling Mixtures, and articulate mechanical tracker wind-chests) that J.S. Bach composed for in Leipzig, Weimar, and Freiberg.
- **Aristide Cavaille-Coll (1811-1899)**: Created the French Romantic symphonic organ (Notre-Dame de Paris, Saint-Sulpice), introducing expressive enclosed Swell divisions, undulating *Voix Celeste* string ranks, and thunderous harmonic reeds.
- **GrandOrgue & Hauptwerk (2000s-2026)**: Brought virtual cathedral pipe organs to home studios and church organists worldwide.

### Famous Tracks & Iconic Usage
- **J.S. Bach -- "Toccata and Fugue in D minor" (BWV 565) & "Passacaglia and Fugue in C minor" (BWV 582)**: The towering peaks of the Baroque *Pleno* and *Tutti* pipe organ repertoire.
- **Charles-Marie Widor -- "Toccata" from Symphony No. 5 (1879)** & **Camille Saint-Saens -- Symphony No. 3 "Organ" (1886)**: Iconic showcases of the French Cavaille-Coll symphonic organ.
- **Hans Zimmer -- "Interstellar (Cornfield Chase / No Time for Caution)" (2014)**: Recorded on the 1926 Harrison & Harrison pipe organ at Temple Church, London -- proving the pipe organ remains the mightiest bass synthesizer in cinema.

### CLI & MCP Automation
```powershell
synthcraft-mcp load-patch --id organ_bach_toccata_tutti
```

---

## Subpage 15.2: GrandOrgue Bach MIDI Depot

### What It Does (Plain English)
Connects directly to your local **GrandOrgue / J.S. Bach `.mid` repertoire library** (automatically pointing to `D:\Dev\repos\grandorgue-mcp\midi_depot` when present, or any folder of `.mid` organ scores on your PC). Browse, filter by title (`jesu`, `prelud`, `fugue`, `air`, `chorale`), and import any piece straight into SynthCraft's 64-step sequencer with matched Cathedral Pipe Organ registration and Stephansdom reverb in one click.

### Controls & Workflow
- **Scan Your Organ Library**: Click **Scan Folder** to list every `.mid` file (automatically skipping macOS `._*` metadata files).
- **Filter & Import**: Type a keyword to narrow down Bach chorales, preludes, or fugues, then click **Import to Organ** to load the score, activate the Cathedral Pipe Organ stops, and engage Stephansdom Cathedral IR reverb.
- **Play Live Alongside GrandOrgue**: You can also export any SynthCraft organ pattern back out as a `.mid` file (Workbench 3) or route MIDI to standalone GrandOrgue via `grandorgue-mcp`.

### Provenance, Hardware Lineage & History
- **J.S. Bach's Orgelbuchlein, Clavier-Ubung III & Leipzig Chorales (1708-1750)**: The foundational body of pipe organ music -- and the primary repertoire preserved in open MIDI archives and `grandorgue-mcp`.

### Famous Tracks & Iconic Usage
- **J.S. Bach -- "Jesu, Joy of Man's Desiring" (BWV 147)**, **"Air on the G String" (BWV 1068)**, and **"Wachet auf, ruft uns die Stimme" (BWV 645)**: Timeless church organ masterworks ready to audition and study directly inside SynthCraft.

### CLI & MCP Automation
```powershell
synthcraft-mcp import-midi-file --path D:\Dev\repos\grandorgue-mcp\midi_depot\11jesu.mid
```
