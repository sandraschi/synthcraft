//! Phase 5 Next-Gen DSP: Reorderable Studio FX Rack:
//! 1. OTT 3-Band Upward/Downward Multiband Compressor
//! 2. Convolution IR Reverb (Stephansdom Cathedral, EMT 140 Plate, 1963 Spring Tank, Vienna Studio)
//! 3. Shimmer Pitch-Shifted Reverb (+12 st / +7 st diffuse feedback)
//! 4. Analog Tape Wow/Flutter + Saturation + Bitcrusher/Sample-Rate Reducer

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FxSlotKind {
    OttMultiband,
    ConvolutionIr,
    ShimmerReverb,
    TapeBitcrush,
}

impl FxSlotKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::OttMultiband => "OTT 3-Band Compressor",
            Self::ConvolutionIr => "Convolution IR Reverb",
            Self::ShimmerReverb => "Shimmer Pitch Reverb",
            Self::TapeBitcrush => "Analog Tape & Bitcrusher",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrPreset {
    StephansdomCathedral,
    Emt140GoldPlate,
    SpringTank1963,
    ViennaStudioRoom,
}

impl IrPreset {
    pub const ALL: [Self; 4] = [
        Self::StephansdomCathedral,
        Self::Emt140GoldPlate,
        Self::SpringTank1963,
        Self::ViennaStudioRoom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::StephansdomCathedral => "Stephansdom Cathedral",
            Self::Emt140GoldPlate => "EMT 140 Gold Plate",
            Self::SpringTank1963 => "1963 Spring Tank",
            Self::ViennaStudioRoom => "Vienna Studio Room",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StudioFxParams {
    pub enabled: bool,
    pub order: [FxSlotKind; 4],
    // OTT Multiband
    pub ott_enabled: bool,
    pub ott_depth: f32, // 0.0 ..= 1.0
    pub ott_time: f32,  // 0.1 ..= 2.0
    // Convolution IR
    pub conv_enabled: bool,
    pub conv_preset: IrPreset,
    pub conv_mix: f32,
    // Shimmer Reverb
    pub shimmer_enabled: bool,
    pub shimmer_mix: f32,
    pub shimmer_pitch_st: f32, // 0.0 ..= 12.0 (default 12.0 = +1 octave)
    pub shimmer_feedback: f32,
    // Analog Tape & Bitcrusher
    pub tape_enabled: bool,
    pub tape_wow: f32,
    pub tape_drive: f32,
    pub crush_bits: f32,       // 4.0 ..= 16.0
    pub crush_downsample: u32, // 1 ..= 16
}

impl Default for StudioFxParams {
    fn default() -> Self {
        Self {
            enabled: false,
            order: [
                FxSlotKind::TapeBitcrush,
                FxSlotKind::OttMultiband,
                FxSlotKind::ShimmerReverb,
                FxSlotKind::ConvolutionIr,
            ],
            ott_enabled: true,
            ott_depth: 0.45,
            ott_time: 0.5,
            conv_enabled: false,
            conv_preset: IrPreset::StephansdomCathedral,
            conv_mix: 0.30,
            shimmer_enabled: false,
            shimmer_mix: 0.35,
            shimmer_pitch_st: 12.0,
            shimmer_feedback: 0.55,
            tape_enabled: false,
            tape_wow: 0.25,
            tape_drive: 0.35,
            crush_bits: 16.0,
            crush_downsample: 1,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct OttState {
    lp1_l: f32,
    lp1_r: f32,
    lp2_l: f32,
    lp2_r: f32,
    env_lo: f32,
    env_mid: f32,
    env_hi: f32,
}

#[derive(Debug, Clone)]
struct ConvIrState {
    ring_l: Vec<f32>,
    ring_r: Vec<f32>,
    pos: usize,
    taps_l: [(usize, f32); 16],
    taps_r: [(usize, f32); 16],
    active_preset: IrPreset,
}

impl ConvIrState {
    fn new(sr: f32) -> Self {
        let mut s = Self {
            ring_l: vec![0.0; (sr as usize * 2).max(96_000)],
            ring_r: vec![0.0; (sr as usize * 2).max(96_000)],
            pos: 0,
            taps_l: [(1, 0.0); 16],
            taps_r: [(1, 0.0); 16],
            active_preset: IrPreset::StephansdomCathedral,
        };
        s.rebuild_taps(IrPreset::StephansdomCathedral, sr);
        s
    }

    fn rebuild_taps(&mut self, preset: IrPreset, sr: f32) {
        self.active_preset = preset;
        let (base_ms, spread_ms, decay) = match preset {
            IrPreset::StephansdomCathedral => (28.0_f32, 92.0_f32, 0.88_f32),
            IrPreset::Emt140GoldPlate => (11.0, 38.0, 0.80),
            IrPreset::SpringTank1963 => (16.0, 44.0, 0.72),
            IrPreset::ViennaStudioRoom => (6.0, 18.0, 0.65),
        };
        let cap = self.ring_l.len().saturating_sub(2);
        let mut g = 0.42_f32;
        for i in 0..16usize {
            let d_l = (((base_ms + i as f32 * spread_ms * (1.0 + (i % 3) as f32 * 0.13))
                * 0.001
                * sr) as usize)
                .clamp(16, cap);
            let d_r = (((base_ms
                + (i as f32 + 0.37) * spread_ms * (1.0 + ((i + 1) % 3) as f32 * 0.11))
                * 0.001
                * sr) as usize)
                .clamp(16, cap);
            let sign = if i.is_multiple_of(2) { 1.0 } else { -1.0 };
            self.taps_l[i] = (d_l, g * sign);
            self.taps_r[i] = (d_r, g * -sign);
            g *= decay;
        }
    }
}

#[derive(Debug, Clone)]
pub struct StudioFxRuntime {
    ott: OttState,
    conv: ConvIrState,
    shimmer_l: Vec<f32>,
    shimmer_r: Vec<f32>,
    shimmer_w: usize,
    shimmer_r1: f32,
    shimmer_r2: f32,
    tape_l: Vec<f32>,
    tape_r: Vec<f32>,
    tape_w: usize,
    wow_phase: f32,
    flutter_phase: f32,
    crush_cnt: u32,
    crush_hold_l: f32,
    crush_hold_r: f32,
}

impl StudioFxRuntime {
    pub fn new(sr: f32) -> Self {
        Self {
            ott: OttState::default(),
            conv: ConvIrState::new(sr),
            shimmer_l: vec![0.0; 32_768],
            shimmer_r: vec![0.0; 32_768],
            shimmer_w: 0,
            shimmer_r1: 0.0,
            shimmer_r2: 4096.0,
            tape_l: vec![0.0; 4096],
            tape_r: vec![0.0; 4096],
            tape_w: 0,
            wow_phase: 0.0,
            flutter_phase: 0.0,
            crush_cnt: 0,
            crush_hold_l: 0.0,
            crush_hold_r: 0.0,
        }
    }

    #[inline]
    pub fn process_stereo(
        &mut self,
        mut l: f32,
        mut r: f32,
        params: &StudioFxParams,
        sr: f32,
    ) -> (f32, f32) {
        if !params.enabled {
            return (l, r);
        }
        for slot in params.order {
            match slot {
                FxSlotKind::OttMultiband if params.ott_enabled && params.ott_depth > 0.001 => {
                    (l, r) = self.tick_ott(l, r, params.ott_depth, params.ott_time, sr);
                }
                FxSlotKind::ConvolutionIr if params.conv_enabled && params.conv_mix > 0.001 => {
                    if self.conv.active_preset != params.conv_preset {
                        self.conv.rebuild_taps(params.conv_preset, sr);
                    }
                    (l, r) = self.tick_conv(l, r, params.conv_mix);
                }
                FxSlotKind::ShimmerReverb
                    if params.shimmer_enabled && params.shimmer_mix > 0.001 =>
                {
                    (l, r) = self.tick_shimmer(
                        l,
                        r,
                        params.shimmer_mix,
                        params.shimmer_pitch_st,
                        params.shimmer_feedback,
                    );
                }
                FxSlotKind::TapeBitcrush if params.tape_enabled => {
                    (l, r) = self.tick_tape_crush(
                        l,
                        r,
                        params.tape_wow,
                        params.tape_drive,
                        params.crush_bits,
                        params.crush_downsample,
                        sr,
                    );
                }
                _ => {}
            }
        }
        (l, r)
    }

    #[inline]
    fn tick_ott(&mut self, l: f32, r: f32, depth: f32, time: f32, sr: f32) -> (f32, f32) {
        let c_lo = (TAU * 250.0 / sr).clamp(0.005, 0.5);
        let c_hi = (TAU * 2500.0 / sr).clamp(0.05, 0.85);
        self.ott.lp1_l += (l - self.ott.lp1_l) * c_lo;
        self.ott.lp1_r += (r - self.ott.lp1_r) * c_lo;
        self.ott.lp2_l += (l - self.ott.lp2_l) * c_hi;
        self.ott.lp2_r += (r - self.ott.lp2_r) * c_hi;

        let lo_l = self.ott.lp1_l;
        let lo_r = self.ott.lp1_r;
        let mid_l = self.ott.lp2_l - self.ott.lp1_l;
        let mid_r = self.ott.lp2_r - self.ott.lp1_r;
        let hi_l = l - self.ott.lp2_l;
        let hi_r = r - self.ott.lp2_r;

        let env_c = (1.0 / (time.clamp(0.1, 2.0) * 0.04 * sr)).clamp(0.0005, 0.05);
        self.ott.env_lo += ((lo_l.abs().max(lo_r.abs())) - self.ott.env_lo) * env_c;
        self.ott.env_mid += ((mid_l.abs().max(mid_r.abs())) - self.ott.env_mid) * env_c;
        self.ott.env_hi += ((hi_l.abs().max(hi_r.abs())) - self.ott.env_hi) * env_c;

        // Upward expansion below 0.12 + downward compression above 0.35
        let ott_gain = |env: f32| -> f32 {
            let e = env.max(0.004);
            if e < 0.12 {
                (0.12 / e).powf(0.45).min(2.8)
            } else if e > 0.35 {
                (0.35 / e).powf(0.65).max(0.25)
            } else {
                1.0
            }
        };
        let g_lo = ott_gain(self.ott.env_lo);
        let g_mid = ott_gain(self.ott.env_mid);
        let g_hi = ott_gain(self.ott.env_hi);

        let wet_l = (lo_l * g_lo + mid_l * g_mid + hi_l * g_hi).tanh();
        let wet_r = (lo_r * g_lo + mid_r * g_mid + hi_r * g_hi).tanh();
        let d = depth.clamp(0.0, 1.0);
        (l * (1.0 - d) + wet_l * d, r * (1.0 - d) + wet_r * d)
    }

    #[inline]
    fn tick_conv(&mut self, l: f32, r: f32, mix: f32) -> (f32, f32) {
        let cap = self.conv.ring_l.len();
        self.conv.ring_l[self.conv.pos] = l;
        self.conv.ring_r[self.conv.pos] = r;
        let mut wl = 0.0_f32;
        let mut wr = 0.0_f32;
        for i in 0..16usize {
            let (dl, gl) = self.conv.taps_l[i];
            let (dr, gr) = self.conv.taps_r[i];
            wl += self.conv.ring_l[(self.conv.pos + cap - dl) % cap] * gl;
            wr += self.conv.ring_r[(self.conv.pos + cap - dr) % cap] * gr;
        }
        self.conv.pos = (self.conv.pos + 1) % cap;
        let m = mix.clamp(0.0, 1.0);
        (l * (1.0 - m * 0.45) + wl * m, r * (1.0 - m * 0.45) + wr * m)
    }

    #[inline]
    fn tick_shimmer(&mut self, l: f32, r: f32, mix: f32, pitch_st: f32, fb: f32) -> (f32, f32) {
        let cap = self.shimmer_l.len();
        let ratio = 2.0_f32.powf(pitch_st.clamp(0.0, 12.0) / 12.0);
        let i1 = (self.shimmer_r1 as usize) % cap;
        let i2 = (self.shimmer_r2 as usize) % cap;
        let sl = (self.shimmer_l[i1] + self.shimmer_r[i2]) * 0.5;
        let sr_s = (self.shimmer_r[i1] + self.shimmer_l[i2]) * 0.5;

        let f = fb.clamp(0.0, 0.85);
        self.shimmer_l[self.shimmer_w] = (l + sl * f).tanh();
        self.shimmer_r[self.shimmer_w] = (r + sr_s * f).tanh();
        self.shimmer_w = (self.shimmer_w + 1) % cap;
        self.shimmer_r1 = (self.shimmer_r1 + ratio).rem_euclid(cap as f32);
        self.shimmer_r2 = (self.shimmer_r2 + ratio * 0.998).rem_euclid(cap as f32);

        let m = mix.clamp(0.0, 1.0);
        (
            l * (1.0 - m * 0.35) + sl * m,
            r * (1.0 - m * 0.35) + sr_s * m,
        )
    }

    #[allow(clippy::too_many_arguments)]
    #[inline]
    fn tick_tape_crush(
        &mut self,
        l: f32,
        r: f32,
        wow: f32,
        drive: f32,
        bits: f32,
        downsample: u32,
        sr: f32,
    ) -> (f32, f32) {
        let cap = self.tape_l.len();
        self.tape_l[self.tape_w] = l;
        self.tape_r[self.tape_w] = r;

        self.wow_phase = (self.wow_phase + 0.65 / sr).fract();
        self.flutter_phase = (self.flutter_phase + 5.7 / sr).fract();
        let mod_samps = 160.0
            + ((TAU * self.wow_phase).sin() * 90.0 + (TAU * self.flutter_phase).sin() * 14.0)
                * wow.clamp(0.0, 1.0);
        let rpos = (self.tape_w as f32 - mod_samps).rem_euclid(cap as f32);
        let i0 = (rpos as usize) % cap;
        let i1 = (i0 + 1) % cap;
        let frac = rpos.fract();
        let mut ol = self.tape_l[i0] + (self.tape_l[i1] - self.tape_l[i0]) * frac;
        let mut or = self.tape_r[i0] + (self.tape_r[i1] - self.tape_r[i0]) * frac;
        self.tape_w = (self.tape_w + 1) % cap;

        // Warm asymmetric tape saturation
        let drv = 1.0 + drive.clamp(0.0, 2.0) * 2.2;
        ol = (ol * drv + 0.04 * drive).tanh();
        or = (or * drv + 0.04 * drive).tanh();

        // Sample-rate & bit-depth reduction
        let ds = downsample.clamp(1, 16);
        self.crush_cnt += 1;
        if self.crush_cnt >= ds {
            self.crush_cnt = 0;
            let levels = 2.0_f32.powf(bits.clamp(4.0, 16.0));
            self.crush_hold_l = (ol * levels).round() / levels;
            self.crush_hold_r = (or * levels).round() / levels;
        }
        (self.crush_hold_l, self.crush_hold_r)
    }
}
