//! Dynamic 3U Eurorack module catalog, multi-row rack state, and modular DSP routing parameters.

use crate::rompler::{RackCable, RackRouting};
use serde::{Deserialize, Serialize};

/// Category for the interactive Module Browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RackCategory {
    Oscillators,
    Filters,
    Modulators,
    Effects,
    Sequencers,
    Utilities,
}

impl RackCategory {
    pub const ALL: [Self; 6] = [
        Self::Oscillators,
        Self::Filters,
        Self::Modulators,
        Self::Effects,
        Self::Sequencers,
        Self::Utilities,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Oscillators => "Oscillators",
            Self::Filters => "Filters",
            Self::Modulators => "Modulators",
            Self::Effects => "Effects",
            Self::Sequencers => "Sequencers",
            Self::Utilities => "Utilities",
        }
    }
}

/// All available 3U Eurorack module types in SynthCraft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RackModuleKind {
    Vco1,
    Vco2Xdsp,
    PcmWorkstation,
    VcfLadder,
    LfoAdsr,
    VcaMaster,
    WavefolderLpg,
    SampleHoldSlew,
    EuclideanClock,
    StereoDelay,
    Attenuverter,
    InlineScope,
}

impl RackModuleKind {
    pub const ALL: [Self; 12] = [
        Self::Vco1,
        Self::Vco2Xdsp,
        Self::PcmWorkstation,
        Self::VcfLadder,
        Self::LfoAdsr,
        Self::VcaMaster,
        Self::WavefolderLpg,
        Self::SampleHoldSlew,
        Self::EuclideanClock,
        Self::StereoDelay,
        Self::Attenuverter,
        Self::InlineScope,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vco1 => "vco1",
            Self::Vco2Xdsp => "vco2_xdsp",
            Self::PcmWorkstation => "pcm_workstation",
            Self::VcfLadder => "vcf_ladder",
            Self::LfoAdsr => "lfo_adsr",
            Self::VcaMaster => "vca_master",
            Self::WavefolderLpg => "wavefolder_lpg",
            Self::SampleHoldSlew => "sample_hold_slew",
            Self::EuclideanClock => "euclidean_clock",
            Self::StereoDelay => "stereo_delay",
            Self::Attenuverter => "attenuverter",
            Self::InlineScope => "inline_scope",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "vco1" | "vco-1" => Some(Self::Vco1),
            "vco2_xdsp" | "vco2" | "xdsp" => Some(Self::Vco2Xdsp),
            "pcm_workstation" | "pcm" | "rompler" => Some(Self::PcmWorkstation),
            "vcf_ladder" | "vcf" | "filter" => Some(Self::VcfLadder),
            "lfo_adsr" | "lfo" | "adsr" => Some(Self::LfoAdsr),
            "vca_master" | "vca" => Some(Self::VcaMaster),
            "wavefolder_lpg" | "wavefolder" | "lpg" => Some(Self::WavefolderLpg),
            "sample_hold_slew" | "sh" | "sample_hold" | "slew" => Some(Self::SampleHoldSlew),
            "euclidean_clock" | "euclidean" | "clock" => Some(Self::EuclideanClock),
            "stereo_delay" | "delay" | "echo" => Some(Self::StereoDelay),
            "attenuverter" | "atten" | "mixer" => Some(Self::Attenuverter),
            "inline_scope" | "scope" | "oscilloscope" => Some(Self::InlineScope),
            _ => None,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Vco1 => "VCO-1",
            Self::Vco2Xdsp => "VCO-2 / X-DSP",
            Self::PcmWorkstation => "PCM-WORKSTATION",
            Self::VcfLadder => "VCF-LADDER",
            Self::LfoAdsr => "LFO / ADSR",
            Self::VcaMaster => "VCA-MASTER",
            Self::WavefolderLpg => "WAVEFOLDER + LPG",
            Self::SampleHoldSlew => "S&H + SLEW",
            Self::EuclideanClock => "EUCLIDEAN CLK",
            Self::StereoDelay => "STEREO DELAY",
            Self::Attenuverter => "ATTENUVERTER",
            Self::InlineScope => "3U SCOPE",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Vco1 => "Primary PolyBLEP analog oscillator with PWM and FM input",
            Self::Vco2Xdsp => "Detuned secondary oscillator + Extreme Modern DSP layer",
            Self::PcmWorkstation => "1990-style synthesized PCM rompler layer",
            Self::VcfLadder => "4-pole zero-delay feedback resonant ladder filter",
            Self::LfoAdsr => "Tempo-synced LFO and ADSR envelope generator",
            Self::VcaMaster => "Master VCA with saturation drive and output gain",
            Self::WavefolderLpg => "West-Coast trigonometric harmonic wavefolder & symmetry bias",
            Self::SampleHoldSlew => "Stepped random Sample & Hold with smooth slew limiter CV",
            Self::EuclideanClock => "Bjorklund Euclidean rhythm & polyrhythmic gate generator",
            Self::StereoDelay => "Ping-pong stereo tape delay with LP-damped feedback",
            Self::Attenuverter => "Dual bipolar CV/signal attenuverter (-100% to +100%)",
            Self::InlineScope => "Live 3U waveform oscilloscope & peak meter faceplate",
        }
    }

    pub fn category(self) -> RackCategory {
        match self {
            Self::Vco1 | Self::Vco2Xdsp | Self::PcmWorkstation => RackCategory::Oscillators,
            Self::VcfLadder => RackCategory::Filters,
            Self::LfoAdsr | Self::SampleHoldSlew => RackCategory::Modulators,
            Self::WavefolderLpg | Self::StereoDelay => RackCategory::Effects,
            Self::EuclideanClock => RackCategory::Sequencers,
            Self::VcaMaster | Self::Attenuverter | Self::InlineScope => RackCategory::Utilities,
        }
    }

    /// Faceplate width in Eurorack HP units (1 HP ~= 12.5 logical px).
    pub fn hp(self) -> u16 {
        match self {
            Self::Attenuverter => 10,
            Self::VcfLadder | Self::StereoDelay | Self::InlineScope => 14,
            _ => 12,
        }
    }

    /// Jack definitions as `(jack_name, is_output)`.
    pub fn jacks(self) -> &'static [(&'static str, bool)] {
        match self {
            Self::Vco1 => &[("FM CV", false), ("SAW OUT", true)],
            Self::Vco2Xdsp => &[("SYNC IN", false), ("DSP OUT", true)],
            Self::PcmWorkstation => &[("GATE IN", false), ("AI2 ROM OUT", true)],
            Self::VcfLadder => &[
                ("AUDIO IN", false),
                ("AUX IN", false),
                ("CUTOFF CV", false),
                ("LP24 OUT", true),
            ],
            Self::LfoAdsr => &[("SINE OUT", true), ("ENV OUT", true)],
            Self::VcaMaster => &[("SIGNAL IN", false), ("GAIN CV", false)],
            Self::WavefolderLpg => &[("AUDIO IN", false), ("FOLD CV", false), ("FOLD OUT", true)],
            Self::SampleHoldSlew => &[("TRIG IN", false), ("S&H OUT", true)],
            Self::EuclideanClock => &[("CLK OUT", true), ("TRIG OUT", true)],
            Self::StereoDelay => &[("AUDIO IN", false), ("DLY OUT", true)],
            Self::Attenuverter => &[
                ("IN 1", false),
                ("OUT 1", true),
                ("IN 2", false),
                ("OUT 2", true),
            ],
            Self::InlineScope => &[("SIG IN", false), ("THRU", true)],
        }
    }

    pub fn default_params(self) -> (f32, f32) {
        match self {
            Self::WavefolderLpg => (0.0, 0.5),
            Self::SampleHoldSlew => (0.35, 0.25),
            Self::EuclideanClock => (0.5, 0.35),
            Self::StereoDelay => (0.38, 0.35),
            Self::Attenuverter => (0.75, 0.5),
            _ => (0.5, 0.5),
        }
    }
}

/// One placed 3U module instance in the multi-row rack.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RackModuleInstance {
    pub id: String,
    pub kind: RackModuleKind,
    pub row: usize,
    #[serde(default)]
    pub bypassed: bool,
    #[serde(default = "default_half")]
    pub param_a: f32,
    #[serde(default = "default_half")]
    pub param_b: f32,
}

fn default_half() -> f32 {
    0.5
}

/// Default 2-row 3U Eurorack skiff: Row 0 has the 6 classic synthesis modules (matching
/// `default_rack_cables()`), and Row 1 has 3 playable expansion modules ready to patch.
pub fn default_rack_modules() -> Vec<RackModuleInstance> {
    vec![
        RackModuleInstance {
            id: "VCO-1".into(),
            kind: RackModuleKind::Vco1,
            row: 0,
            bypassed: false,
            param_a: 0.5,
            param_b: 0.5,
        },
        RackModuleInstance {
            id: "VCO-2 / X-DSP".into(),
            kind: RackModuleKind::Vco2Xdsp,
            row: 0,
            bypassed: false,
            param_a: 0.5,
            param_b: 0.5,
        },
        RackModuleInstance {
            id: "PCM-WORKSTATION".into(),
            kind: RackModuleKind::PcmWorkstation,
            row: 0,
            bypassed: false,
            param_a: 0.5,
            param_b: 0.5,
        },
        RackModuleInstance {
            id: "VCF-LADDER".into(),
            kind: RackModuleKind::VcfLadder,
            row: 0,
            bypassed: false,
            param_a: 0.5,
            param_b: 0.5,
        },
        RackModuleInstance {
            id: "LFO / ADSR".into(),
            kind: RackModuleKind::LfoAdsr,
            row: 0,
            bypassed: false,
            param_a: 0.5,
            param_b: 0.5,
        },
        RackModuleInstance {
            id: "VCA-MASTER".into(),
            kind: RackModuleKind::VcaMaster,
            row: 0,
            bypassed: false,
            param_a: 0.5,
            param_b: 0.5,
        },
        // Row 1 starter expansion modules
        RackModuleInstance {
            id: "WAVEFOLDER-1".into(),
            kind: RackModuleKind::WavefolderLpg,
            row: 1,
            bypassed: false,
            param_a: 0.0,
            param_b: 0.5,
        },
        RackModuleInstance {
            id: "DELAY-1".into(),
            kind: RackModuleKind::StereoDelay,
            row: 1,
            bypassed: false,
            param_a: 0.38,
            param_b: 0.35,
        },
        RackModuleInstance {
            id: "SCOPE-1".into(),
            kind: RackModuleKind::InlineScope,
            row: 1,
            bypassed: false,
            param_a: 0.5,
            param_b: 0.5,
        },
    ]
}

/// Compute a Euclidean rhythm mask (`steps` in 2..=16, `pulses` in 1..=steps).
pub fn euclidean_pattern(steps: usize, pulses: usize) -> [bool; 16] {
    let mut out = [false; 16];
    let n = steps.clamp(2, 16);
    let k = pulses.clamp(1, n);
    for (i, slot) in out.iter_mut().take(n).enumerate() {
        *slot = ((i * k) % n) < k;
    }
    out
}

/// Compute live `RackRouting` from both placed modules (respecting bypass & custom params)
/// and connected patch cables.
pub fn compute_rack_routing(modules: &[RackModuleInstance], cables: &[RackCable]) -> RackRouting {
    let mut r = RackRouting::from_cables(cables);

    let find_active = |kind: RackModuleKind| -> Option<&RackModuleInstance> {
        modules.iter().find(|m| m.kind == kind && !m.bypassed)
    };

    if let Some(wf) = find_active(RackModuleKind::WavefolderLpg) {
        r.fold_amount = wf.param_a.clamp(0.0, 1.0);
        r.fold_symmetry = (wf.param_b.clamp(0.0, 1.0) - 0.5) * 1.4;
    }
    if let Some(dly) = find_active(RackModuleKind::StereoDelay) {
        // Only engage delay when feedback/mix is turned up or a cable connects into it
        let patched = cables
            .iter()
            .any(|c| c.to_module == dly.id || c.from_module == dly.id);
        let time_s = 0.06 + dly.param_a.clamp(0.0, 1.0) * 0.69;
        let fdbk = dly.param_b.clamp(0.0, 0.88);
        r.delay_time_s = time_s;
        r.delay_feedback = fdbk;
        r.delay_mix = if patched {
            0.45
        } else if fdbk > 0.4 {
            (fdbk - 0.35) * 0.7
        } else {
            0.0
        };
    }
    if let Some(sh) = find_active(RackModuleKind::SampleHoldSlew) {
        r.sh_rate_hz = 0.5 + sh.param_a.clamp(0.0, 1.0) * 19.5;
        r.sh_slew = sh.param_b.clamp(0.0, 1.0);
        for c in cables {
            if c.from_module == sh.id && c.from_jack == "S&H OUT" {
                match (c.to_module.as_str(), c.to_jack.as_str()) {
                    ("VCF-LADDER", "CUTOFF CV") => {
                        r.sh_to_cutoff = (r.sh_to_cutoff + c.gain.clamp(0.0, 2.0)).clamp(0.0, 2.0);
                    }
                    ("VCO-1", "FM CV") => {
                        r.sh_to_fm = (r.sh_to_fm + c.gain.clamp(0.0, 2.0)).clamp(0.0, 2.0);
                    }
                    _ => {}
                }
            }
        }
    }
    if let Some(att) = find_active(RackModuleKind::Attenuverter) {
        // Bipolar scale (-1.0..+1.0) applied to LFO cutoff modulation when attenuverter is present
        let scale = (att.param_a.clamp(0.0, 1.0) - 0.5) * 2.0;
        r.lfo_to_cutoff *= scale;
    }

    r
}
