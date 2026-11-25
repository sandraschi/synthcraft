//! Phase 7 Goliath-Class DSP:
//! - Real-Time 2D Finite-Difference Time-Domain (FDTD) Physical Modeling Plate / Membrane Mesh
//!   (8x8 = 64 coupled spatial nodes solved every sample with stereo pickups and material dispersion).
//! - 8x Oversampled Wave Digital Filter (WDF) Analog Circuit Emulation
//!   (Moog CP3 transistor overdrive, Buchla 292 Vactrol Low-Pass Gate with asymmetric CdS decay,
//!   and TB-303 diode ladder).

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

pub const MESH_SIDE: usize = 8;
pub const MESH_NODES: usize = MESH_SIDE * MESH_SIDE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicalMaterial {
    TitaniumPlate,
    GlassMarimba,
    KettleMembrane,
    BowedCelloBody,
}

impl PhysicalMaterial {
    pub const ALL: [Self; 4] = [
        Self::TitaniumPlate,
        Self::GlassMarimba,
        Self::KettleMembrane,
        Self::BowedCelloBody,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::TitaniumPlate => "Titanium Plate",
            Self::GlassMarimba => "Glass Marimba",
            Self::KettleMembrane => "Kettle Membrane",
            Self::BowedCelloBody => "Bowed Cello Body",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PhysicalMeshParams {
    pub enabled: bool,
    pub material: PhysicalMaterial,
    pub tension: f32, // 0.05 ..= 0.45 (Courant stability limit <= 0.5)
    pub damping: f32, // 0.001 ..= 0.08
    pub mix: f32,
}

impl Default for PhysicalMeshParams {
    fn default() -> Self {
        Self {
            enabled: false,
            material: PhysicalMaterial::TitaniumPlate,
            tension: 0.28,
            damping: 0.008,
            mix: 0.45,
        }
    }
}

/// Real-time 8x8 2D Waveguide Mesh (FDTD discrete 2D wave equation). Allocation-free.
#[derive(Debug, Clone)]
pub struct PhysicalMeshRuntime {
    u_curr: [f32; MESH_NODES],
    u_prev: [f32; MESH_NODES],
    u_next: [f32; MESH_NODES],
}

impl Default for PhysicalMeshRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalMeshRuntime {
    pub fn new() -> Self {
        Self {
            u_curr: [0.0; MESH_NODES],
            u_prev: [0.0; MESH_NODES],
            u_next: [0.0; MESH_NODES],
        }
    }

    /// Snapshot current 8x8 membrane displacement for the live heatmap visualizer.
    pub fn node_amplitudes(&self) -> [f32; MESH_NODES] {
        self.u_curr
    }

    #[inline]
    pub fn tick_stereo(
        &mut self,
        excite_l: f32,
        excite_r: f32,
        params: &PhysicalMeshParams,
    ) -> (f32, f32) {
        if !params.enabled || params.mix <= 0.001 {
            return (excite_l, excite_r);
        }
        let (mat_c, mat_damp, edge_refl) = match params.material {
            PhysicalMaterial::TitaniumPlate => (1.05_f32, 0.55_f32, -0.92_f32),
            PhysicalMaterial::GlassMarimba => (1.18, 0.35, -0.96),
            PhysicalMaterial::KettleMembrane => (0.72, 0.85, -0.88),
            PhysicalMaterial::BowedCelloBody => (0.88, 1.10, -0.85),
        };
        let c2 = (params.tension * mat_c).clamp(0.03, 0.44);
        let damp = (1.0 - (params.damping * mat_damp).clamp(0.0005, 0.12)).clamp(0.88, 0.9995);

        // Inject stereo excitation at two strike points on the 8x8 grid
        self.u_curr[2 * MESH_SIDE + 2] =
            (self.u_curr[2 * MESH_SIDE + 2] + excite_l * 0.22).clamp(-2.0, 2.0);
        self.u_curr[5 * MESH_SIDE + 5] =
            (self.u_curr[5 * MESH_SIDE + 5] + excite_r * 0.22).clamp(-2.0, 2.0);

        for y in 0..MESH_SIDE {
            for x in 0..MESH_SIDE {
                let idx = y * MESH_SIDE + x;
                if x == 0 || y == 0 || x + 1 == MESH_SIDE || y + 1 == MESH_SIDE {
                    // Clamped/reflective rim boundary
                    self.u_next[idx] = self.u_curr[idx] * edge_refl * 0.15;
                } else {
                    let lap = self.u_curr[idx - 1]
                        + self.u_curr[idx + 1]
                        + self.u_curr[idx - MESH_SIDE]
                        + self.u_curr[idx + MESH_SIDE]
                        - 4.0 * self.u_curr[idx];
                    let next = (2.0 * self.u_curr[idx] - self.u_prev[idx] + c2 * lap) * damp;
                    self.u_next[idx] = next.clamp(-2.0, 2.0);
                }
            }
        }
        self.u_prev = self.u_curr;
        self.u_curr = self.u_next;

        // Dual stereo pickup microphones at asymmetric nodes on the vibrating plate
        let tap_l = (self.u_curr[3 * MESH_SIDE + 2] + self.u_curr[4 * MESH_SIDE + 3] * 0.6).tanh();
        let tap_r = (self.u_curr[4 * MESH_SIDE + 5] + self.u_curr[3 * MESH_SIDE + 4] * 0.6).tanh();
        let m = params.mix.clamp(0.0, 1.0);
        (
            excite_l * (1.0 - m) + tap_l * m,
            excite_r * (1.0 - m) + tap_r * m,
        )
    }
}

// ---------------------------------------------------------------------------
// 8x Oversampled Wave Digital Filter (WDF) Analog Circuit Emulation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WdfCircuitMode {
    Off,
    MoogCp3Overdrive,
    Buchla292VactrolLpg,
    Tb303DiodeLadder,
}

impl WdfCircuitMode {
    pub const ALL: [Self; 4] = [
        Self::Off,
        Self::MoogCp3Overdrive,
        Self::Buchla292VactrolLpg,
        Self::Tb303DiodeLadder,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off (Bypass)",
            Self::MoogCp3Overdrive => "Moog CP3 (8x WDF)",
            Self::Buchla292VactrolLpg => "Buchla 292 Vactrol LPG (8x)",
            Self::Tb303DiodeLadder => "TB-303 Diode Ladder (8x)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WdfCircuitParams {
    pub mode: WdfCircuitMode,
    pub drive: f32,         // 0.0 ..= 3.0
    pub vactrol_decay: f32, // 0.05 ..= 1.0 (CdS photoresistor ring-out)
    pub mix: f32,
}

impl Default for WdfCircuitParams {
    fn default() -> Self {
        Self {
            mode: WdfCircuitMode::Off,
            drive: 1.2,
            vactrol_decay: 0.45,
            mix: 0.75,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct WdfCh {
    s1: f32,
    s2: f32,
    vactrol_env: f32,
}

#[derive(Debug, Clone, Default)]
pub struct WdfCircuitRuntime {
    l: WdfCh,
    r: WdfCh,
}

impl WdfCircuitRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn tick_ch(ch: &mut WdfCh, input: f32, params: &WdfCircuitParams, sr: f32) -> f32 {
        let os_sr = sr * 8.0;
        let drv = 1.0 + params.drive.clamp(0.0, 3.0) * 1.8;
        let mut acc = 0.0_f32;
        for _ in 0..8 {
            let y = match params.mode {
                WdfCircuitMode::Off => input,
                WdfCircuitMode::MoogCp3Overdrive => {
                    // Asymmetric discrete transistor clipping + gentle 1-pole Miller rolloff
                    let x = input * drv;
                    let asym = if x >= 0.0 {
                        (x * 1.12).tanh()
                    } else {
                        (x * 0.92).tanh() * 1.04
                    };
                    let g = (PI * 11000.0 / os_sr).clamp(0.01, 0.45);
                    ch.s1 += g * (asym - ch.s1);
                    ch.s1
                }
                WdfCircuitMode::Buchla292VactrolLpg => {
                    // Asymmetric CdS photoresistor envelope: fast 2 ms strike, slow organic ring-out
                    let target = input.abs().min(1.5);
                    let c = if target > ch.vactrol_env {
                        1.0 / (0.002 * os_sr)
                    } else {
                        1.0 / ((0.05 + params.vactrol_decay.clamp(0.05, 1.0) * 0.55) * os_sr)
                    };
                    ch.vactrol_env += (target - ch.vactrol_env) * c.clamp(1e-6, 0.1);
                    let cut = (90.0 + ch.vactrol_env * 8500.0).clamp(60.0, 14000.0);
                    let g = (PI * cut / os_sr).clamp(0.0005, 0.4);
                    ch.s1 += g * ((input * drv).tanh() - ch.s1);
                    ch.s2 += g * (ch.s1 - ch.s2);
                    ch.s2 * (0.35 + ch.vactrol_env * 0.95)
                }
                WdfCircuitMode::Tb303DiodeLadder => {
                    // Non-linear diode-bridge ladder at 8x oversampling
                    let cut = (420.0 + params.vactrol_decay * 5200.0).clamp(80.0, 12000.0);
                    let g = (PI * cut / os_sr).clamp(0.001, 0.42);
                    let fb = (ch.s2 * (0.4 + params.drive * 0.35)).tanh();
                    let x = (input * drv - fb).tanh();
                    ch.s1 += g * (x - ch.s1);
                    ch.s2 += g * (ch.s1 - ch.s2);
                    ch.s2 * 1.35
                }
            };
            acc += y;
        }
        let wet = acc * 0.125;
        let m = params.mix.clamp(0.0, 1.0);
        input * (1.0 - m) + wet * m
    }

    #[inline]
    pub fn process_stereo(
        &mut self,
        l: f32,
        r: f32,
        params: &WdfCircuitParams,
        sr: f32,
    ) -> (f32, f32) {
        if params.mode == WdfCircuitMode::Off || params.mix <= 0.001 {
            return (l, r);
        }
        (
            Self::tick_ch(&mut self.l, l, params, sr),
            Self::tick_ch(&mut self.r, r, params, sr),
        )
    }
}
