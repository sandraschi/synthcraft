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
        en: "1990 PCM workstation PCM Rompler",
        de: "1990 PCM workstation PCM-Rompler",
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
    en: "1-Click PCM / Acid / West-Coast Demos",
    de: "1-Klick PCM / Acid / West-Coast Demos",
}];

const SEQ_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::SequencerEnv,
        sub: 0,
        en: "64-Step Acid Sequencer (4 Bars)",
        de: "64-Step Acid-Sequenzer (4 Takte)",
    },
    SubPage {
        page: Page::SequencerEnv,
        sub: 2,
        en: "TR-808/909 8-Voice Drum Machine",
        de: "TR-808/909 8-Voice Drum-Machine",
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

const SETTINGS_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::Settings,
        sub: 0,
        en: "Audio Driver & DSP Oversampling",
        de: "Audio-Treiber & DSP-Oversampling",
    },
    SubPage {
        page: Page::Settings,
        sub: 1,
        en: "MIDI Hardware, Clock & MPE",
        de: "MIDI-Hardware, Clock & MPE",
    },
    SubPage {
        page: Page::Settings,
        sub: 2,
        en: "Skins, Layouts & Knob Physics",
        de: "Skins, Layouts & Regler-Physik",
    },
    SubPage {
        page: Page::Settings,
        sub: 3,
        en: "PCM workstation DAC & Extreme DSP Tuning",
        de: "PCM workstation DAC & Extreme-DSP-Kalibrierung",
    },
    SubPage {
        page: Page::Settings,
        sub: 4,
        en: "CLAP, SoundCraft & DJCraft Paths",
        de: "CLAP, SoundCraft & DJCraft-Pfade",
    },
    SubPage {
        page: Page::Settings,
        sub: 5,
        en: "AI Assistant, MCP & SQLite DB",
        de: "KI-Assistent, MCP & SQLite-DB",
    },
];

const NEXTGEN_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::NextGenLab,
        sub: 0,
        en: "MSEG & Dual Morph Filter B",
        de: "MSEG & Dual-Morph-Filter B",
    },
    SubPage {
        page: Page::NextGenLab,
        sub: 1,
        en: "Granular Cloud & 64-Partial Spectral",
        de: "Granular-Cloud & 64-Partial-Spektral",
    },
    SubPage {
        page: Page::NextGenLab,
        sub: 2,
        en: "Genopatch AI Audio-to-Patch",
        de: "Genopatch KI Audio-zu-Patch",
    },
];

const PRO_LAB_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::ProLab,
        sub: 0,
        en: "6-Op FM Matrix & 16-Voice Unison",
        de: "6-Op FM-Matrix & 16-Voice Unison",
    },
    SubPage {
        page: Page::ProLab,
        sub: 1,
        en: "Reorderable FX (OTT/IR/Shimmer/Tape)",
        de: "FX-Rack (OTT/IR/Shimmer/Tape)",
    },
    SubPage {
        page: Page::ProLab,
        sub: 2,
        en: "2D FDTD Mesh & 8x WDF Circuits",
        de: "2D-FDTD-Mesh & 8x WDF-Schaltkreise",
    },
];

static WORKBENCHES: [CraftWorkbench; 12] = [
    CraftWorkbench {
        id: "modular_rack",
        en: "1. VCV 2 Eurorack & VCO/VCF",
        de: "1. VCV 2 Eurorack & VCO/VCF",
        subpages: RACK_SUBS,
    },
    CraftWorkbench {
        id: "m1_extreme_dsp",
        en: "2. 1990-style PCM & Extreme DSP",
        de: "2. 1990-style PCM & Extreme-DSP",
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
    CraftWorkbench {
        id: "studio_settings",
        en: "10. Studio Settings & Prefs",
        de: "10. Studio-Einstellungen",
        subpages: SETTINGS_SUBS,
    },
    CraftWorkbench {
        id: "nextgen_lab",
        en: "11. Next-Gen Lab (MSEG/Granular/AI)",
        de: "11. Next-Gen-Labor (MSEG/Granular/KI)",
        subpages: NEXTGEN_SUBS,
    },
    CraftWorkbench {
        id: "pro_lab",
        en: "12. Pro Lab (6-Op FM/FX/2D Physics)",
        de: "12. Pro-Labor (6-Op FM/FX/2D-Physik)",
        subpages: PRO_LAB_SUBS,
    },
];

pub fn registry() -> Vec<&'static dyn SubApp> {
    WORKBENCHES.iter().map(|w| w as &dyn SubApp).collect()
}
