# Samples, the M1 question and licensing

Status: 2026-11-23. Read this before adding audio to the repository or promising M1 sounds.

## 1. Where things stand

- The sampler loads WAV files, folders of WAVs named by note, SFZ instruments and SF2 soundfonts (`crates/synthcraft-core/src/sampler.rs`). Zones have key and velocity ranges, root key, tuning, gain and loop points. Playback uses cubic Hermite interpolation.
- A zone that is marked as looping but has no loop points gets an automatic sustain loop: the sampler searches the sustain section for a stretch of at least 0.4 s that repeats well, then bakes an equal-power crossfade into the join so the wrap does not click. If nothing repeats well enough it falls back to looping the whole sample, as the SFZ specification says.
- The built-in "PCM workstation" programs (`PcmProgram`, ids such as `m1_01_piano16`) are synthesized. They are not sampled ROM data and they are not Korg sounds. They keep the old ids so saved databases still load.
- Nothing licensed or third-party is stored in the repository. `banks/manifest.json` lists downloadable packs by pinned commit and SHA-256 and is the only thing committed.

## 2. Getting real sounds in: three routes

| Route | Sounds like | Cost | Can be committed or redistributed |
|---|---|---|---|
| Free CC0 packs via `fetch-bank` | Orchestral strings today (VSCO 2 CE violin and cello section sustains) | free | Manifest only; the pack itself is CC0 |
| Your own SF2 or SFZ files dropped into the Banks folder | Whatever you own | depends | No, unless the licence says so |
| Licensed KORG Collection M1 bounced to WAV, for personal use | The real M1 | see section 4 | Never |

Banks folder: `Documents\SynthCraft\Banks` (the rack panel has a "Create folder" button and lists what it finds).

### 2.1 Free packs

```
synthcraft-mcp list-banks
synthcraft-mcp fetch-bank vsco2ce-cello-section-sus
```

The app has the same thing as a "Free sample banks (download)" section in the sample bank panel. Each file is verified against its SHA-256 before it is kept; a mismatch writes nothing for that file. The command writes `<id>.sfz` (with `loop_mode=loop_continuous`) and `ATTRIBUTION.txt` next to the samples and prints the `.sfz` path. Paste that path into the Bank field if you did not use the button.

### 2.2 Adding a pack to the manifest

1. Check the licence file of the exact source and note the SPDX id. Do not rely on a summary: some collections mix CC0, CC-BY and GPL parts.
2. Pin a commit (never a branch name). Raw URLs are built from `repo`, `commit` and `path`.
3. List only the files you need, each with its `note` (for example `C3` or `F#3`), `bytes` and `sha256`. Keep packs small; sustained one-velocity sets are enough for pads.
4. Add the attribution line to `ATTRIBUTION.md`; CI checks that assets are attributed.
5. Run `cargo xtask ci`, then `fetch-bank <id>` on a clean folder, then the real-bank test: `SYNTHCRAFT_TEST_BANK=<path to .sfz> cargo test -p synthcraft-core --test real_banks`.

Candidates, licence to be verified per pack before use: more VSCO 2 CE instruments (CC0), FreePats (CC0 and CC-BY mix), Salamander Grand Piano (CC-BY, needs attribution), University of Iowa Musical Instrument Samples (free with attribution), GeneralUser GS (its own licence, read it).

## 3. The M1 sound and why single looped samples sound flat

The original M1 builds many sounds from a short, non-looping attack transient followed by a looped sustain, then layers or splits programs and runs them through its filter, envelope and effects. A convincing M1-style set therefore needs both an attack layer and a sustain layer per key range. One looped sample per key range sounds flat because the transient is missing.

Current gap: `SampleBank::find` returns a single zone for a note and velocity. Overlapping zones (attack plus sustain) are not played together yet. The work to close it:

1. Let `find` return all zones that match, and give each voice a small list of zone playheads.
2. Support one-shot zones (`loop_mode=one_shot`, no loop) that play to their end regardless of key release, alongside looped zones.
3. Respect SFZ `group` and `off_by` only if a bank needs it; not required for attack plus sustain.
4. Tests: a bank with a 50 ms transient zone and a looped sustain zone must produce more energy in the first 50 ms than the sustain zone alone, and must not click at the transition.

Do this after you have heard the real M1 and decided you want it sampled into SynthCraft.

## 4. The licensed M1 (KORG Collection M1 V2)

Facts found on 2026-11-23, with sources at the end:

- KORG Collection M1 V2 contains the PCM data of the original M1 and all 3,300 programs, including those from the expansion ROM cards.
- It is the only licensed source of the real data. A free demo exists; a KORG ID account is required for it.
- Activation goes through KORG Software Pass: install it, create or use a KORG ID, confirm the email, log in, then activate. This is Korg's own scheme, not iLok or a dongle. It needs a KORG ID and an internet connection to activate. Offline activation for some products is possible through the KORG ID Pass according to a forum thread, but the thread did not show how. Reviewers call the activation manager slightly clunky; one user could not install an update of Software Pass at all (Collection 3 era).
- What exactly Software Pass installs on Windows (services, drivers, startup items) was not documented in the sources found. If it matters, install it on a test machine and inspect services and startup entries afterwards.
- Price: list $99.99 for the M1 alone on Plugin Boutique; KORG Collection 6 (21 instruments) is $199 instead of $399 in the Black Friday sale. Korg's sale runs 12 to 30 November 2026. A claimed $49.99 price for the M1 alone was not confirmed. In Korg's 2022 summer sale the M1 was $49.99.
- Unconfirmed: whether a standalone version exists next to the plugin formats, and the demo's session limit and saving restrictions. Check the product page before buying.

Rules for using it with SynthCraft:

1. Bounce from the plugin to WAV for your own music. A plugin licence normally covers using the sounds in your music, not redistributing the sample data. Read the end user licence agreement you accept.
2. Keep bounced audio outside the repository (it is also covered by `.gitignore` only for demo files, so do not rely on that). Bundling it would make the `MIT OR Apache-2.0` claim false.
3. Put the bounces in `Documents\SynthCraft\Banks\<name>\` and write an SFZ for them. Name files by note and layer, for example `choir_C3_attack.wav` and `choir_C3_sustain.wav`.

### 4.1 Bounce procedure for a pad bank

- One program at a time, no effects (or record dry and let SynthCraft add chorus and reverb).
- Every minor third across the useful range (about 12 to 16 notes per program), one or two velocities.
- Hold each note long enough for the sustain to settle (6 to 10 s), then let the release ring out.
- Export 24-bit or 16-bit WAV at the project rate, mono or stereo (stereo is mixed down to mono today).
- Trim the file start; leave the tail.
- SFZ per program: one region per file with `lokey`, `hikey`, `pitch_keycenter`, `loop_mode=loop_continuous`. Leave out loop points to use the automatic loop, or write `loop_start` and `loop_end` by hand for a better join.

## 5. Redistribution checklist for anything audio

- Do I have the licence text for these exact files? Is it CC0, CC-BY, or something else?
- If CC-BY: is the attribution in `ATTRIBUTION.md` and in the pack's `ATTRIBUTION.txt`?
- Is it small enough to fetch on demand instead of committing it? (Always prefer the manifest.)
- Did a plugin or hardware vendor's licence produce this audio? Then it stays local.

This is practical guidance, not legal advice.

## Sources

- KORG Collection M1 V2 product page: https://www.korg.com/us/products/software/kc_m1/index.php
- KORG 2026 Black Friday news: https://www.korg.com/us/news/2026/1112/
- Plugin Boutique KORG install guide: https://help.pluginboutique.com/hc/en-us/articles/47832643148180-KORG
- Plugin Boutique M1 listing: https://www.pluginboutique.com/products/11198
- Korg forum thread on offline activation: https://korgforums.com/forum/phpBB3/viewtopic.php?p=837042
- VSCO 2 Community Edition (CC0): https://github.com/sgossner/VSCO-2-CE

## Layers, one-shots, stereo and ambient mode

Added after the first plug-in release; all of it is in `synthcraft-core` and so reaches the app, the CLAP plug-in and the Rack module at once.

- **Layered zones.** A note now plays every zone that matches its key and velocity, up to 4 (`MAX_LAYERS`), instead of only the first. An SFZ or SF2 bank with an attack transient zone over a looped sustain zone sounds as the author built it. Layers add up, so overlapping zones of one bank are louder than a single zone; `volume` in SFZ and the SF2 attenuation set the balance. When nothing matches, the nearest zone is used as before.
- **One-shot zones.** SFZ `loop_mode=one_shot` (without a loop) marks a zone that plays to its end and ignores note-off, like a drum hit. The voice frees itself when the last one-shot ends. SF2 sample modes are not read yet, so SF2 zones are never one-shot.
- **Per-zone pan.** SFZ `pan` (-100 to 100) is read. A voice has one filter, so a voice with several layers is panned by the mean of its layers' pans, not each layer separately.
- **Stereo engine.** `SynthRt::process_stereo` is the one synthesis path; `process` is its mono average. Each voice slot has a fixed position in the field scaled by `stereo_spread` (default 0.3; 0 is exactly mono), chorus taps go to opposite channels, and reverb is a decorrelated pair of tanks. The desktop app, CLAP, the C ABI and Rack all output true stereo. Offline WAV export and DJCraft stems are still mono (the average). This changes the sound of existing patches slightly because the default spread is not 0.
- **Ambient mode.** `ambient_depth` (0 is off) adds a slow seeded drift to the filter cutoff (up to 1.5 octaves), the X-morph (up to 0.35) and the reverb send (up to 0.25); `ambient_rate_hz` sets the speed and `ambient_seed` (patch JSON only, not a host parameter) the pattern. Same seed, same drift. The drift needs `reverb_mix` above 0 to move the reverb send, and `extreme_dsp` on to move the morph.
- **Fixed on the way:** pitch bend was applied twice to sampler playback.
