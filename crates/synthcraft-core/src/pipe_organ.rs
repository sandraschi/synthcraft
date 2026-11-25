//! Cathedral Pipe Organ Console & Physical/Additive DSP Engine (`#![forbid(unsafe_code)]`).
//!
//! Models a 3-division *Werkprinzip* pipe organ (Hauptwerk / Schwellwerk / Pedalwerk) with:
//! - 16 authentic stop ranks (`Bourdon 16'`, `Principal 8'`, `Rohrflote 8'`, `Oktave 4'`,
//!   `Quinte 2 2/3'`, `Superoktave 2'`, `Mixtur IV`, `Trompete 8'`, `Gedackt 8'`,
//!   `Salicional 8'`, `Voix Celeste 8'`, `Blockflote 4'`, `Oboe 8'`, `Subbass 16'`,
//!   `Posaune 16'`, `Principalbass 8'`)
//! - Manual/Pedal Couplers (`SW->HW`, `HW->PED`, `SW->PED`, `Sub 16'`, `Super 4'`)
//! - Swell Box Venetian shutter filter, Crescendo Walze, Wind-Chest Chiff, Bellows Wind Sag,
//!   Tremulant, 5 Historical Organ Temperaments (incl. Werckmeister III & Silbermann Meantone),
//!   8 Combination Pistons, and GrandOrgue `.mid` Depot Scanner.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
use std::path::{Path, PathBuf};

pub const ORGAN_STOP_COUNT: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganDivision {
    Hauptwerk,
    Schwellwerk,
    Pedalwerk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganTemperament {
    Equal12,
    WerckmeisterIII,
    KirnbergerIII,
    SilbermannMeantone,
    QuarterCommaMeantone,
}

impl OrganTemperament {
    pub const ALL: [Self; 5] = [
        Self::Equal12,
        Self::WerckmeisterIII,
        Self::KirnbergerIII,
        Self::SilbermannMeantone,
        Self::QuarterCommaMeantone,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Equal12 => "Equal Temperament (12-TET)",
            Self::WerckmeisterIII => "Werckmeister III (1691, Bach)",
            Self::KirnbergerIII => "Kirnberger III (1779)",
            Self::SilbermannMeantone => "Silbermann 1/6-Comma Meantone",
            Self::QuarterCommaMeantone => "Quarter-Comma Meantone (Renaissance)",
        }
    }

    /// Pitch offset in cents for pitch class `0..12` (C=0 .. B=11).
    pub fn cents_offset(self, pc: usize) -> f32 {
        const WERCK3: [f32; 12] = [
            0.0, -9.8, -7.8, -5.9, -9.8, -2.0, -11.7, -3.9, -7.8, -11.7, -3.9, -7.8,
        ];
        const KIRN3: [f32; 12] = [
            0.0, -9.8, -6.8, -5.9, -13.7, -2.0, -11.7, -3.4, -7.8, -10.3, -3.9, -11.7,
        ];
        const SILB: [f32; 12] = [
            0.0, -14.0, -4.0, 2.0, -8.0, 2.0, -12.0, -2.0, -10.0, -6.0, 4.0, -10.0,
        ];
        const QC_MEAN: [f32; 12] = [
            0.0, -24.0, -6.8, 10.3, -13.7, 3.4, -20.5, -3.4, -27.4, -10.3, 6.8, -17.1,
        ];
        let idx = pc % 12;
        match self {
            Self::Equal12 => 0.0,
            Self::WerckmeisterIII => WERCK3[idx],
            Self::KirnbergerIII => KIRN3[idx],
            Self::SilbermannMeantone => SILB[idx],
            Self::QuarterCommaMeantone => QC_MEAN[idx],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OrganStopSpec {
    pub index: usize,
    pub name: &'static str,
    pub footage: &'static str,
    pub division: OrganDivision,
    pub pitch_mult: f32,
    pub detune_cents: f32,
    pub harmonic_profile: [f32; 6], // amplitudes of partials 1..=6
    pub is_reed: bool,
    pub is_mixture: bool,
}

pub const ORGAN_STOPS: [OrganStopSpec; ORGAN_STOP_COUNT] = [
    // Hauptwerk (Great Manual I): 0..8
    OrganStopSpec {
        index: 0,
        name: "Bourdon 16'",
        footage: "16'",
        division: OrganDivision::Hauptwerk,
        pitch_mult: 0.5,
        detune_cents: -1.5,
        harmonic_profile: [1.0, 0.08, 0.36, 0.04, 0.14, 0.02],
        is_reed: false,
        is_mixture: false,
    },
    OrganStopSpec {
        index: 1,
        name: "Principal 8'",
        footage: "8'",
        division: OrganDivision::Hauptwerk,
        pitch_mult: 1.0,
        detune_cents: 0.0,
        harmonic_profile: [1.0, 0.58, 0.34, 0.22, 0.12, 0.07],
        is_reed: false,
        is_mixture: false,
    },
    OrganStopSpec {
        index: 2,
        name: "Rohrflote 8'",
        footage: "8'",
        division: OrganDivision::Hauptwerk,
        pitch_mult: 1.0,
        detune_cents: 1.2,
        harmonic_profile: [1.0, 0.18, 0.42, 0.06, 0.18, 0.03],
        is_reed: false,
        is_mixture: false,
    },
    OrganStopSpec {
        index: 3,
        name: "Oktave 4'",
        footage: "4'",
        division: OrganDivision::Hauptwerk,
        pitch_mult: 2.0,
        detune_cents: -0.8,
        harmonic_profile: [0.95, 0.52, 0.30, 0.18, 0.09, 0.04],
        is_reed: false,
        is_mixture: false,
    },
    OrganStopSpec {
        index: 4,
        name: "Quinte 2 2/3'",
        footage: "2 2/3'",
        division: OrganDivision::Hauptwerk,
        pitch_mult: 3.0,
        detune_cents: 1.5,
        harmonic_profile: [0.85, 0.32, 0.18, 0.08, 0.04, 0.02],
        is_reed: false,
        is_mixture: false,
    },
    OrganStopSpec {
        index: 5,
        name: "Superoktave 2'",
        footage: "2'",
        division: OrganDivision::Hauptwerk,
        pitch_mult: 4.0,
        detune_cents: -1.2,
        harmonic_profile: [0.88, 0.42, 0.22, 0.10, 0.05, 0.02],
        is_reed: false,
        is_mixture: false,
    },
    OrganStopSpec {
        index: 6,
        name: "Mixtur IV",
        footage: "1 1/3'",
        division: OrganDivision::Hauptwerk,
        pitch_mult: 6.0,
        detune_cents: 2.0,
        harmonic_profile: [0.75, 0.65, 0.52, 0.40, 0.25, 0.15],
        is_reed: false,
        is_mixture: true,
    },
    OrganStopSpec {
        index: 7,
        name: "Trompete 8'",
        footage: "8'",
        division: OrganDivision::Hauptwerk,
        pitch_mult: 1.0,
        detune_cents: 2.4,
        harmonic_profile: [0.82, 0.78, 0.72, 0.60, 0.48, 0.36],
        is_reed: true,
        is_mixture: false,
    },
    // Schwellwerk (Swell Manual II): 8..13
    OrganStopSpec {
        index: 8,
        name: "Gedackt 8'",
        footage: "8'",
        division: OrganDivision::Schwellwerk,
        pitch_mult: 1.0,
        detune_cents: -1.0,
        harmonic_profile: [1.0, 0.06, 0.32, 0.03, 0.11, 0.01],
        is_reed: false,
        is_mixture: false,
    },
    OrganStopSpec {
        index: 9,
        name: "Salicional 8'",
        footage: "8'",
        division: OrganDivision::Schwellwerk,
        pitch_mult: 1.0,
        detune_cents: 0.5,
        harmonic_profile: [0.72, 0.65, 0.50, 0.35, 0.22, 0.14],
        is_reed: false,
        is_mixture: false,
    },
    OrganStopSpec {
        index: 10,
        name: "Voix Celeste 8'",
        footage: "8'",
        division: OrganDivision::Schwellwerk,
        pitch_mult: 1.0,
        detune_cents: 13.5,
        harmonic_profile: [0.70, 0.62, 0.46, 0.30, 0.18, 0.10],
        is_reed: false,
        is_mixture: false,
    },
    OrganStopSpec {
        index: 11,
        name: "Blockflote 4'",
        footage: "4'",
        division: OrganDivision::Schwellwerk,
        pitch_mult: 2.0,
        detune_cents: 0.9,
        harmonic_profile: [1.0, 0.22, 0.14, 0.05, 0.02, 0.01],
        is_reed: false,
        is_mixture: false,
    },
    OrganStopSpec {
        index: 12,
        name: "Oboe 8'",
        footage: "8'",
        division: OrganDivision::Schwellwerk,
        pitch_mult: 1.0,
        detune_cents: -1.8,
        harmonic_profile: [0.65, 0.75, 0.85, 0.52, 0.30, 0.16],
        is_reed: true,
        is_mixture: false,
    },
    // Pedalwerk (Pedalboard): 13..16
    OrganStopSpec {
        index: 13,
        name: "Subbass 16'",
        footage: "16'",
        division: OrganDivision::Pedalwerk,
        pitch_mult: 0.5,
        detune_cents: 0.0,
        harmonic_profile: [1.0, 0.12, 0.28, 0.04, 0.08, 0.01],
        is_reed: false,
        is_mixture: false,
    },
    OrganStopSpec {
        index: 14,
        name: "Posaune 16'",
        footage: "16'",
        division: OrganDivision::Pedalwerk,
        pitch_mult: 0.5,
        detune_cents: 2.2,
        harmonic_profile: [0.90, 0.80, 0.68, 0.52, 0.36, 0.24],
        is_reed: true,
        is_mixture: false,
    },
    OrganStopSpec {
        index: 15,
        name: "Principalbass 8'",
        footage: "8'",
        division: OrganDivision::Pedalwerk,
        pitch_mult: 1.0,
        detune_cents: -0.6,
        harmonic_profile: [0.95, 0.50, 0.28, 0.14, 0.06, 0.03],
        is_reed: false,
        is_mixture: false,
    },
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PipeOrganParams {
    pub enabled: bool,
    pub mix: f32,
    pub stops: [bool; ORGAN_STOP_COUNT],
    // Couplers
    pub coupler_sw_to_hw: bool,
    pub coupler_hw_to_ped: bool,
    pub coupler_sw_to_ped: bool,
    pub coupler_sub_16: bool,
    pub coupler_super_4: bool,
    // Expression & Mechanical Wind Physics
    pub swell_pedal: f32,     // 0.0 (closed shutters) .. 1.0 (wide open)
    pub crescendo_pedal: f32, // 0.0 (off) .. 1.0 (full Tutti Walze)
    pub chiff_amount: f32,    // 0.0 .. 1.0 wind-chest edge-tone transient
    pub wind_sag: f32,        // 0.0 .. 1.0 mechanical bellows pressure dip
    pub tremulant_enabled: bool,
    pub tremulant_rate_hz: f32,
    pub tremulant_depth: f32,
    pub temperament: OrganTemperament,
    pub piston_name: String,
}

impl Default for PipeOrganParams {
    fn default() -> Self {
        let mut p = Self {
            enabled: false,
            mix: 0.85,
            stops: [false; ORGAN_STOP_COUNT],
            coupler_sw_to_hw: true,
            coupler_hw_to_ped: true,
            coupler_sw_to_ped: false,
            coupler_sub_16: false,
            coupler_super_4: false,
            swell_pedal: 0.82,
            crescendo_pedal: 0.0,
            chiff_amount: 0.38,
            wind_sag: 0.22,
            tremulant_enabled: false,
            tremulant_rate_hz: 5.4,
            tremulant_depth: 0.28,
            temperament: OrganTemperament::WerckmeisterIII,
            piston_name: "Silbermann Baroque Pleno".into(),
        };
        p.load_piston("silbermann_pleno");
        p
    }
}

impl PipeOrganParams {
    pub fn piston_list() -> &'static [(&'static str, &'static str)] {
        &[
            (
                "vespers_pp",
                "1: pp Vespers Flute (Gedackt 8' + Subbass 16')",
            ),
            (
                "celeste_p",
                "2: p Romantic Celeste (Salicional + Voix Celeste 8')",
            ),
            (
                "baroque_mf",
                "3: mf Baroque Chorale (Principal 8' + Rohrflote + Oktave 4')",
            ),
            (
                "silbermann_pleno",
                "4: f Silbermann Pleno (Principal 8'+4'+2'+Mixtur IV)",
            ),
            (
                "cavaille_coll_ff",
                "5: ff Cavaille-Coll Romantic (Bourdon 16' + Reeds + Swell)",
            ),
            (
                "bach_toccata_tutti",
                "6: fff Bach Toccata Tutti (Full Cathedral Organ)",
            ),
            (
                "trompete_fanfare",
                "7: Solo Trompete 8' + Posaune 16' Fanfare",
            ),
            ("general_cancel", "0: General Cancel (All Stops Off)"),
        ]
    }

    pub fn load_piston(&mut self, id: &str) {
        self.stops = [false; ORGAN_STOP_COUNT];
        match id {
            "vespers_pp" => {
                self.stops[8] = true; // Gedackt 8'
                self.stops[13] = true; // Subbass 16'
                self.swell_pedal = 0.55;
                self.tremulant_enabled = false;
                self.piston_name = "pp Vespers Flute".into();
            }
            "celeste_p" => {
                self.stops[9] = true; // Salicional 8'
                self.stops[10] = true; // Voix Celeste 8'
                self.stops[13] = true; // Subbass 16'
                self.swell_pedal = 0.68;
                self.tremulant_enabled = true;
                self.piston_name = "p Romantic Voix Celeste".into();
            }
            "baroque_mf" => {
                self.stops[1] = true; // Principal 8'
                self.stops[2] = true; // Rohrflote 8'
                self.stops[3] = true; // Oktave 4'
                self.stops[11] = true; // Blockflote 4'
                self.stops[13] = true; // Subbass 16'
                self.stops[15] = true; // Principalbass 8'
                self.swell_pedal = 0.80;
                self.tremulant_enabled = false;
                self.piston_name = "mf Baroque Chorale".into();
            }
            "silbermann_pleno" => {
                self.stops[1] = true; // Principal 8'
                self.stops[3] = true; // Oktave 4'
                self.stops[4] = true; // Quinte 2 2/3'
                self.stops[5] = true; // Superoktave 2'
                self.stops[6] = true; // Mixtur IV
                self.stops[13] = true; // Subbass 16'
                self.stops[15] = true; // Principalbass 8'
                self.swell_pedal = 0.90;
                self.tremulant_enabled = false;
                self.piston_name = "f Silbermann Pleno".into();
            }
            "cavaille_coll_ff" => {
                self.stops[0] = true; // Bourdon 16'
                self.stops[1] = true; // Principal 8'
                self.stops[3] = true; // Oktave 4'
                self.stops[7] = true; // Trompete 8'
                self.stops[9] = true; // Salicional 8'
                self.stops[10] = true; // Voix Celeste 8'
                self.stops[12] = true; // Oboe 8'
                self.stops[13] = true; // Subbass 16'
                self.stops[14] = true; // Posaune 16'
                self.swell_pedal = 0.85;
                self.coupler_sw_to_hw = true;
                self.piston_name = "ff Cavaille-Coll Romantic".into();
            }
            "bach_toccata_tutti" => {
                self.stops = [true; ORGAN_STOP_COUNT];
                self.stops[10] = false; // keep Celeste off in pure Tutti
                self.coupler_sw_to_hw = true;
                self.coupler_hw_to_ped = true;
                self.coupler_super_4 = true;
                self.swell_pedal = 1.0;
                self.tremulant_enabled = false;
                self.piston_name = "fff Bach Toccata Tutti".into();
            }
            "trompete_fanfare" => {
                self.stops[1] = true; // Principal 8'
                self.stops[7] = true; // Trompete 8'
                self.stops[12] = true; // Oboe 8'
                self.stops[14] = true; // Posaune 16'
                self.swell_pedal = 1.0;
                self.piston_name = "Solo Trompete Fanfare".into();
            }
            _ => {
                self.piston_name = "General Cancel".into();
            }
        }
    }

    pub fn active_stop_count(&self) -> usize {
        self.stops.iter().filter(|&&on| on).count()
    }
}

/// Real-time allocation-free Cathedral Pipe Organ DSP runtime.
#[derive(Debug, Clone)]
pub struct PipeOrganRuntime {
    stop_phases: [f32; ORGAN_STOP_COUNT],
    trem_phase: f32,
    swell_lp_l: f32,
    swell_lp_r: f32,
    chiff_env: f32,
    last_freq: f32,
    rng: u32,
    wind_pressure: f32,
}

impl Default for PipeOrganRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl PipeOrganRuntime {
    pub fn new() -> Self {
        Self {
            stop_phases: [0.0; ORGAN_STOP_COUNT],
            trem_phase: 0.0,
            swell_lp_l: 0.0,
            swell_lp_r: 0.0,
            chiff_env: 0.0,
            last_freq: 0.0,
            rng: 0x0B4C_4565, // BWV 565 seed!
            wind_pressure: 1.0,
        }
    }

    #[inline]
    fn next_noise(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        ((x as f32) / (u32::MAX as f32)) * 2.0 - 1.0
    }

    /// Synthesize one stereo frame of Cathedral Pipe Organ audio conditioned on played `freq_hz`
    /// and envelope `env_amp`, blended with incoming stereo `(in_l, in_r)`. Allocation-free.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_stereo(
        &mut self,
        in_l: f32,
        in_r: f32,
        freq_hz: f32,
        midi_note: u8,
        env_amp: f32,
        params: &PipeOrganParams,
        sr: f32,
    ) -> (f32, f32) {
        if !params.enabled || params.mix <= 0.001 || freq_hz < 16.0 || env_amp <= 0.0005 {
            return (in_l, in_r);
        }

        // Trigger wind-chest chiff burst when note changes
        if (freq_hz - self.last_freq).abs() > 1.0 {
            self.last_freq = freq_hz;
            self.chiff_env = 1.0;
        }
        self.chiff_env *= (-1.0 / (0.022 * sr.max(8000.0))).exp();

        // Historical organ temperament adjustment
        let temp_cents = params.temperament.cents_offset((midi_note % 12) as usize);
        let temp_ratio = 2.0_f32.powf(temp_cents / 1200.0);

        // Tremulant bellows modulation (~5.4 Hz)
        let mut trem_amp = 1.0_f32;
        let mut trem_pitch = 1.0_f32;
        if params.tremulant_enabled {
            self.trem_phase += params.tremulant_rate_hz.clamp(2.0, 9.0) / sr.max(8000.0);
            if self.trem_phase >= 1.0 {
                self.trem_phase -= 1.0;
            }
            let s = (TAU * self.trem_phase).sin();
            let d = params.tremulant_depth.clamp(0.0, 1.0);
            trem_amp = 1.0 + s * 0.14 * d;
            trem_pitch = 2.0_f32.powf((s * 9.0 * d) / 1200.0);
        }

        // Bellows wind-pressure sag under dense registration
        let active_n = params.active_stop_count() as f32;
        let target_wind =
            1.0 - (params.wind_sag.clamp(0.0, 1.0) * (active_n / 16.0) * 0.004 * env_amp);
        self.wind_pressure += (target_wind - self.wind_pressure) * (15.0 / sr.max(8000.0));

        let base_f = freq_hz * temp_ratio * trem_pitch * self.wind_pressure;
        let cresc = params.crescendo_pedal.clamp(0.0, 1.0);
        let swell_open = params.swell_pedal.clamp(0.05, 1.0);

        let mut hw_l = 0.0_f32;
        let mut hw_r = 0.0_f32;
        let mut sw_l = 0.0_f32;
        let mut sw_r = 0.0_f32;
        let mut active_weight = 0.0_f32;

        // Progressive crescendo order (softest to loudest)
        const CRESC_ORDER: [usize; ORGAN_STOP_COUNT] =
            [8, 13, 9, 2, 11, 1, 15, 3, 12, 0, 4, 5, 6, 7, 14, 10];

        for (rank_pos, &idx) in CRESC_ORDER.iter().enumerate() {
            let stop = &ORGAN_STOPS[idx];
            let cresc_thresh = (rank_pos as f32 + 0.5) / (ORGAN_STOP_COUNT as f32);
            let on = params.stops[idx] || (cresc > 0.02 && cresc >= cresc_thresh);
            if !on {
                continue;
            }
            // Skip Pedalwerk stops on high manual notes unless coupler enabled
            if stop.division == OrganDivision::Pedalwerk
                && midi_note > 64
                && !params.coupler_hw_to_ped
            {
                continue;
            }

            let mut mult = stop.pitch_mult;
            if params.coupler_sub_16 && stop.division == OrganDivision::Hauptwerk {
                mult *= 0.5;
            } else if params.coupler_super_4 && stop.division == OrganDivision::Hauptwerk {
                mult *= 2.0;
            }

            let det = 2.0_f32.powf(stop.detune_cents / 1200.0);
            let f = base_f * mult * det;
            if f > sr * 0.44 {
                continue;
            }

            self.stop_phases[idx] += f / sr.max(8000.0);
            if self.stop_phases[idx] >= 1.0 {
                self.stop_phases[idx] -= 1.0;
            }
            let ph = self.stop_phases[idx] * TAU;

            let mut s = 0.0_f32;
            for (h_i, &amp) in stop.harmonic_profile.iter().enumerate() {
                if amp <= 0.001 {
                    continue;
                }
                let k = (h_i + 1) as f32;
                if f * k < sr * 0.45 {
                    s += amp * (ph * k).sin();
                }
            }
            if stop.is_mixture && f * 2.0 < sr * 0.45 {
                s += 0.35 * (ph * 1.5).sin();
            }

            // Spatialize pipes across the church facade (C-side left, C#-side right!)
            let pan = if (midi_note + idx as u8).is_multiple_of(2) {
                -0.35
            } else {
                0.35
            };
            let gl = 0.5 * (1.0 - pan);
            let gr = 0.5 * (1.0 + pan);

            match stop.division {
                OrganDivision::Schwellwerk => {
                    sw_l += s * gl;
                    sw_r += s * gr;
                }
                OrganDivision::Hauptwerk | OrganDivision::Pedalwerk => {
                    hw_l += s * gl;
                    hw_r += s * gr;
                }
            }
            active_weight += 1.0;
        }

        // Swell box Venetian shutter low-pass & attenuation
        let sw_alpha = (0.04 + swell_open * 0.88).clamp(0.04, 0.95);
        self.swell_lp_l += sw_alpha * (sw_l - self.swell_lp_l);
        self.swell_lp_r += sw_alpha * (sw_r - self.swell_lp_r);
        let sw_gain = 0.25 + 0.75 * swell_open;

        // Wind-chest chiff edge-tone noise burst on pipe attack
        let chiff = self.next_noise() * self.chiff_env * params.chiff_amount.clamp(0.0, 1.0) * 0.14;

        let norm = (active_weight.sqrt().max(1.0)) * 1.65;
        let org_l = ((hw_l + self.swell_lp_l * sw_gain) / norm + chiff) * env_amp * trem_amp;
        let org_r = ((hw_r + self.swell_lp_r * sw_gain) / norm - chiff * 0.8) * env_amp * trem_amp;

        let m = params.mix.clamp(0.0, 1.0);
        (
            (in_l * (1.0 - m) + org_l.tanh() * m).clamp(-1.0, 1.0),
            (in_r * (1.0 - m) + org_r.tanh() * m).clamp(-1.0, 1.0),
        )
    }
}

/// Entry from a GrandOrgue / Bach `.mid` repertoire folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganMidiEntry {
    pub title: String,
    pub path: PathBuf,
    pub bytes: u64,
}

/// Default path to Sandra's local `grandorgue-mcp/midi_depot` Bach repertoire if present.
pub fn default_grandorgue_midi_dir() -> PathBuf {
    PathBuf::from(r"D:\Dev\repos\grandorgue-mcp\midi_depot")
}

/// Scan any directory for `.mid` / `.midi` organ repertoire files (ignoring macOS `._*` dotfiles).
pub fn scan_organ_midi_depot(dir: &Path) -> Vec<OrganMidiEntry> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(fname) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if fname.starts_with("._") {
            continue;
        }
        let is_mid = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("mid") || e.eq_ignore_ascii_case("midi"));
        if !is_mid {
            continue;
        }
        let bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(fname)
            .to_string();
        out.push(OrganMidiEntry { title, path, bytes });
    }
    out.sort_by_key(|a| a.title.to_lowercase());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipe_organ_pistons_and_dsp_synthesize_bounded_cathedral_audio() {
        let mut p = PipeOrganParams {
            enabled: true,
            ..Default::default()
        };
        p.load_piston("bach_toccata_tutti");
        assert!(p.active_stop_count() >= 14);

        let mut rt = PipeOrganRuntime::new();
        let mut peak = 0.0_f32;
        for _ in 0..2048 {
            let (l, r) = rt.process_stereo(0.0, 0.0, 293.66, 62, 0.9, &p, 48_000.0);
            assert!(l.is_finite() && r.is_finite());
            assert!((-1.0..=1.0).contains(&l) && (-1.0..=1.0).contains(&r));
            peak = peak.max(l.abs()).max(r.abs());
        }
        assert!(peak > 0.15, "pipe organ output too quiet: {peak}");
    }

    #[test]
    fn werckmeister_and_meantone_temperaments_differ_from_equal() {
        assert_eq!(OrganTemperament::Equal12.cents_offset(4), 0.0);
        assert!(OrganTemperament::WerckmeisterIII.cents_offset(4) < -5.0);
        assert!(OrganTemperament::QuarterCommaMeantone.cents_offset(4) < -10.0);
    }
}
