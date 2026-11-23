//! Hierarchical `SubApp` sidebar registry for SynthCraft's 6 workbenches.

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
        en: "Dual VCO & Ladder VCF",
        de: "Doppel-VCO & Kaskadenfilter",
    },
    SubPage {
        page: Page::ModularRack,
        sub: 1,
        en: "Patch Library & Presets",
        de: "Patch-Bibliothek & Presets",
    },
];

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

static WORKBENCHES: [CraftWorkbench; 6] = [
    CraftWorkbench {
        id: "modular_rack",
        en: "1. Modular Rack & VCO/VCF",
        de: "1. Modular-Rack & VCO/VCF",
        subpages: RACK_SUBS,
    },
    CraftWorkbench {
        id: "sequencer_env",
        en: "2. Sequencer & ADSR/LFO",
        de: "2. Sequenzer & ADSR/LFO",
        subpages: SEQ_SUBS,
    },
    CraftWorkbench {
        id: "scope_spectrum",
        en: "3. Oscilloscope & Spectrum",
        de: "3. Oszilloskop & Spektrum",
        subpages: SCOPE_SUBS,
    },
    CraftWorkbench {
        id: "clap_soundcraft",
        en: "4. CLAP & SoundCraft Bridge",
        de: "4. CLAP & SoundCraft-Bruecke",
        subpages: CLAP_SUBS,
    },
    CraftWorkbench {
        id: "djcraft_forge",
        en: "5. DJCraft Stem & WAV Forge",
        de: "5. DJCraft Stem & WAV-Schmiede",
        subpages: DJ_SUBS,
    },
    CraftWorkbench {
        id: "assistant",
        en: "6. Assistant & Help",
        de: "6. Assistent & Hilfe",
        subpages: ASSISTANT_SUBS,
    },
];

pub fn registry() -> Vec<&'static dyn SubApp> {
    WORKBENCHES.iter().map(|w| w as &dyn SubApp).collect()
}
