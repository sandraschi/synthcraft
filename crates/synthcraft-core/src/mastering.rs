//! One-Shot #13: Studio Mastering Chain (`#![forbid(unsafe_code)]`).
//!
//! Real-time allocation-free final mastering bus right before the DAC / WAV / CLAP output:
//! - 4-Band Parametric EQ (Low Shelf 80 Hz, Low-Mid 350 Hz, High-Mid 2.8 kHz, High Shelf 9.5 kHz)
//! - Sub-Bass Mono Maker (`< 120 Hz` summed to mono for club PA / vinyl / DJCraft decks)
//! - Mid/Side Stereo Width Imager (`0.0` Mono .. `1.0` Natural .. `2.0` Super-Wide)
//! - True-Peak Brickwall Limiter with Input Drive, Ceiling (`-6.0 .. -0.1 dBFS`), and Gain Reduction dB!

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MasteringParams {
    pub enabled: bool,
    // 4-Band Parametric EQ (-12.0 ..= +12.0 dB)
    pub eq_low_db: f32,
    pub eq_low_mid_db: f32,
    pub eq_high_mid_db: f32,
    pub eq_high_db: f32,
    // Stereo Imaging
    pub mono_bass_below_120hz: bool,
    pub stereo_width: f32, // 0.0 (Mono) .. 1.0 (Natural) .. 2.0 (Super-Wide)
    // Brickwall Limiter
    pub limiter_enabled: bool,
    pub limiter_drive_db: f32,   // 0.0 ..= +12.0 dB
    pub limiter_ceiling_db: f32, // -6.0 ..= -0.1 dBFS
}

impl Default for MasteringParams {
    fn default() -> Self {
        Self {
            enabled: false,
            eq_low_db: 0.0,
            eq_low_mid_db: 0.0,
            eq_high_mid_db: 0.0,
            eq_high_db: 0.0,
            mono_bass_below_120hz: true,
            stereo_width: 1.0,
            limiter_enabled: true,
            limiter_drive_db: 1.5,
            limiter_ceiling_db: -0.3,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct EqCh {
    lp_80: f32,
    lp_120: f32,
    lp_350: f32,
    lp_2800: f32,
    lp_9500: f32,
}

#[derive(Debug, Clone, Default)]
pub struct MasteringRuntime {
    l: EqCh,
    r: EqCh,
    lim_env: f32,
    pub last_gr_db: f32,
}

impl MasteringRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn db_to_lin(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    /// Process one stereo frame through the 4-band EQ, Sub-Bass Mono Maker, M/S Imager, and Brickwall Limiter.
    #[inline]
    pub fn process_stereo(
        &mut self,
        mut l: f32,
        mut r: f32,
        params: &MasteringParams,
        sr: f32,
    ) -> (f32, f32) {
        if !params.enabled {
            self.last_gr_db = 0.0;
            return (l, r);
        }

        let c80 = (TAU * 80.0 / sr).clamp(0.002, 0.35);
        let c120 = (TAU * 120.0 / sr).clamp(0.003, 0.40);
        let c350 = (TAU * 350.0 / sr).clamp(0.01, 0.55);
        let c2800 = (TAU * 2800.0 / sr).clamp(0.05, 0.85);
        let c9500 = (TAU * 9500.0 / sr).clamp(0.15, 0.95);

        // 1. 4-Band Split & Gain Sculpting (L & R)
        let g_lo = Self::db_to_lin(params.eq_low_db.clamp(-12.0, 12.0)) - 1.0;
        let g_lm = Self::db_to_lin(params.eq_low_mid_db.clamp(-12.0, 12.0)) - 1.0;
        let g_hm = Self::db_to_lin(params.eq_high_mid_db.clamp(-12.0, 12.0)) - 1.0;
        let g_hi = Self::db_to_lin(params.eq_high_db.clamp(-12.0, 12.0)) - 1.0;

        for (sig, ch) in [(&mut l, &mut self.l), (&mut r, &mut self.r)] {
            let x = *sig;
            ch.lp_80 += (x - ch.lp_80) * c80;
            ch.lp_120 += (x - ch.lp_120) * c120;
            ch.lp_350 += (x - ch.lp_350) * c350;
            ch.lp_2800 += (x - ch.lp_2800) * c2800;
            ch.lp_9500 += (x - ch.lp_9500) * c9500;

            let band_lo = ch.lp_80;
            let band_lm = ch.lp_350 - ch.lp_80;
            let band_hm = ch.lp_2800 - ch.lp_350;
            let band_hi = x - ch.lp_9500;
            *sig = x + band_lo * g_lo + band_lm * g_lm + band_hm * g_hm + band_hi * g_hi;
        }

        // 2. Sub-Bass Mono Maker (< 120 Hz summed to mono)
        if params.mono_bass_below_120hz {
            let sub_mono = (self.l.lp_120 + self.r.lp_120) * 0.5;
            l = (l - self.l.lp_120) + sub_mono;
            r = (r - self.r.lp_120) + sub_mono;
        }

        // 3. Mid/Side Stereo Width Imager
        let width = params.stereo_width.clamp(0.0, 2.0);
        let mid = (l + r) * 0.5;
        let side = (l - r) * 0.5 * width;
        l = mid + side;
        r = mid - side;

        // 4. True-Peak Brickwall Limiter
        if params.limiter_enabled {
            let drive = Self::db_to_lin(params.limiter_drive_db.clamp(0.0, 12.0));
            let ceil = Self::db_to_lin(params.limiter_ceiling_db.clamp(-6.0, -0.1));
            l *= drive;
            r *= drive;

            let peak = l.abs().max(r.abs());
            let c = if peak > self.lim_env {
                0.35 // ultra-fast attack
            } else {
                (1.0 / (0.065 * sr)).clamp(0.0001, 0.02)
            };
            self.lim_env += (peak - self.lim_env) * c;

            let gain = if self.lim_env > ceil {
                ceil / self.lim_env.max(1e-6)
            } else {
                1.0
            };
            self.last_gr_db = 20.0 * gain.clamp(0.01, 1.0).log10();
            l = (l * gain).clamp(-ceil, ceil);
            r = (r * gain).clamp(-ceil, ceil);
        } else {
            self.last_gr_db = 0.0;
        }

        (l, r)
    }
}
