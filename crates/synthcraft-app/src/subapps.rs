//! Hierarchical `SubApp` sidebar registry for SynthCraft's 9 workbenches (17 subpages).

use crate::app::{AppLocale, Page};

#[derive(Debug, Clone, Copy)]
pub struct SubPage {
    pub page: Page,
    pub sub: usize,
    pub en: &'static str,
    pub de: &'static str,
}

impl SubPage {
    pub fn label(&self, locale: AppLocale) -> &'static str {
        match locale {
            AppLocale::En => self.en,
            AppLocale::De => self.de,
        }
    }
}

pub trait SubApp: Sync {
    fn id(&self) -> &'static str;
    fn title(&self, locale: AppLocale) -> &'static str;
    fn subpages(&self) -> &'static [SubPage];
}

struct CraftWorkbench {
    id: &'static str,
    en: &'static str,
    de: &'static str,
    subpages: &'static [SubPage],
}

impl SubApp for CraftWorkbench {
    fn id(&self) -> &'static str {
        self.id
    }

    fn title(&self, locale: AppLocale) -> &'static str {
        match locale {
            AppLocale::En => self.en,
            AppLocale::De => self.de,
        }
    }

    fn subpages(&self) -> &'static [SubPage] {
        self.subpages
    }
}

const RACK_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::ModularRack,
        sub: 0,
        en: "VCV 3U Eurorack & Patch Cables",
        de: "VCV 3U Eurorack & Patch-Kabel",
    },
    SubPage {
        page: Page::ModularRack,
        sub: 1,
        en: "Factory Patch Bank Overview",
        de: "Factory-Patch-Bank",
    },
];

const M1_DSP_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::M1ExtremeDsp,
        sub: 0,
        en: "1990 Korg M1 PCM Rompler",
        de: "1990 Korg M1 PCM-Rompler",
    },
    SubPage {
        page: Page::M1ExtremeDsp,
        sub: 1,
        en: "Extreme Modern DSP Engine",
        de: "Extreme-DSP-Synthese",
    },
];

const MIDI_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::MidiDepot,
        sub: 0,
        en: "MIDI Keyboard & 25-Key Piano",
        de: "MIDI-Keyboard & 25-Tasten-Piano",
    },
    SubPage {
        page: Page::MidiDepot,
        sub: 1,
        en: "MIDI Pattern Depot & .MID Export",
        de: "MIDI-Pattern-Depot & .MID-Export",
    },
];

const DEMO_SUBS: &[SubPage] = &[SubPage {
    page: Page::DemoShowcase,
    sub: 0,
    en: "1-Click M1 / Acid / Buchla Demos",
    de: "1-Klick M1 / Acid / Buchla Demos",
}];

const SEQ_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::SequencerEnv,
        sub: 0,
        en: "16-Step Acid Sequencer",
        de: "16-Step Acid-Sequenzer",
    },
    SubPage {
        page: Page::SequencerEnv,
        sub: 1,
        en: "ADSR Envelope & Tempo LFO",
        de: "ADSR-Huellkurve & Tempo-LFO",
    },
];

const SCOPE_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::ScopeSpectrum,
        sub: 0,
        en: "Waveform Oscilloscope",
        de: "Wellenform-Oszilloskop",
    },
    SubPage {
        page: Page::ScopeSpectrum,
        sub: 1,
        en: "32-Band DFT Spectrum & dBFS",
        de: "32-Band DFT-Spektrum & dBFS",
    },
];

const CLAP_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::ClapSoundcraft,
        sub: 0,
        en: "CLAP 1.2.2 Plugin Descriptor",
        de: "CLAP 1.2.2 Plugin-Deskriptor",
    },
    SubPage {
        page: Page::ClapSoundcraft,
        sub: 1,
        en: "SoundCraft Parameter Map",
        de: "SoundCraft Parameter-Tabelle",
    },
];

const DJ_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::DjcraftForge,
        sub: 0,
        en: "BPM & Camelot Loop Forge",
        de: "BPM & Camelot Loop-Schmiede",
    },
    SubPage {
        page: Page::DjcraftForge,
        sub: 1,
        en: "RIFF WAV & Stem Exporter",
        de: "RIFF WAV & Stem-Exporter",
    },
];

const ASSISTANT_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::Chat,
        sub: 0,
        en: "Chat",
        de: "Chat",
    },
    SubPage {
        page: Page::Help,
        sub: 0,
        en: "Help (F1)",
        de: "Hilfe (F1)",
    },
];

static WORKBENCHES: [CraftWorkbench; 9] = [
    CraftWorkbench {
        id: "modular_rack",
        en: "1. VCV 2 Eurorack & VCO/VCF",
        de: "1. VCV 2 Eurorack & VCO/VCF",
        subpages: RACK_SUBS,
    },
    CraftWorkbench {
        id: "m1_extreme_dsp",
        en: "2. Korg M1 (1990) & Extreme DSP",
        de: "2. Korg M1 (1990) & Extreme-DSP",
        subpages: M1_DSP_SUBS,
    },
    CraftWorkbench {
        id: "midi_depot",
        en: "3. MIDI Keyboard & MIDI Depot",
        de: "3. MIDI-Keyboard & MIDI-Depot",
        subpages: MIDI_SUBS,
    },
    CraftWorkbench {
        id: "demo_showcase",
        en: "4. Demo Showcase & Audition",
        de: "4. Demo-Showcase & Audition",
        subpages: DEMO_SUBS,
    },
    CraftWorkbench {
        id: "sequencer_env",
        en: "5. Sequencer & ADSR/LFO",
        de: "5. Sequenzer & ADSR/LFO",
        subpages: SEQ_SUBS,
    },
    CraftWorkbench {
        id: "scope_spectrum",
        en: "6. Oscilloscope & Spectrum",
        de: "6. Oszilloskop & Spektrum",
        subpages: SCOPE_SUBS,
    },
    CraftWorkbench {
        id: "clap_soundcraft",
        en: "7. CLAP & SoundCraft Bridge",
        de: "7. CLAP & SoundCraft-Bruecke",
        subpages: CLAP_SUBS,
    },
    CraftWorkbench {
        id: "djcraft_forge",
        en: "8. DJCraft Stem & WAV Forge",
        de: "8. DJCraft Stem & WAV-Schmiede",
        subpages: DJ_SUBS,
    },
    CraftWorkbench {
        id: "assistant",
        en: "9. Assistant & Help",
        de: "9. Assistent & Hilfe",
        subpages: ASSISTANT_SUBS,
    },
];

pub fn registry() -> Vec<&'static dyn SubApp> {
    WORKBENCHES.iter().map(|w| w as &dyn SubApp).collect()
}
