//! One-Shot #12: 8-Scene Live Performance Snapshot Morphing & 4-Corner XY Pad (Octatrack / Orb style).
//!
//! Stores 8 macro parameter snapshots per patch and interpolates smoothly between:
//! - DJ Crossfader (`Scene A <-> Scene B`, `0.0..=1.0`)
//! - 4-Corner Bilinear Scene XY Pad (morphing Scenes 1, 2, 3, 4 simultaneously in real time!)

use crate::model::SynthPatch;
use serde::{Deserialize, Serialize};

pub const SCENE_COUNT: usize = 8;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PerformanceScene {
    pub name: String,
    pub cutoff_hz: f32,
    pub resonance_q: f32,
    pub fm_index: f32,
    pub drive_saturation: f32,
    pub extreme_morph: f32,
    pub filter2_cutoff_hz: f32,
    pub filter2_vowel: f32,
    pub granular_mix: f32,
    pub shimmer_mix: f32,
    pub ott_depth: f32,
    pub ddsp_morph: f32,
}

impl Default for PerformanceScene {
    fn default() -> Self {
        Self {
            name: "Init Scene".into(),
            cutoff_hz: 1200.0,
            resonance_q: 3.5,
            fm_index: 0.25,
            drive_saturation: 0.4,
            extreme_morph: 0.3,
            filter2_cutoff_hz: 1400.0,
            filter2_vowel: 0.5,
            granular_mix: 0.0,
            shimmer_mix: 0.0,
            ott_depth: 0.25,
            ddsp_morph: 0.3,
        }
    }
}

impl PerformanceScene {
    pub fn capture_from(patch: &SynthPatch, name: &str) -> Self {
        Self {
            name: name.to_string(),
            cutoff_hz: patch.cutoff_hz,
            resonance_q: patch.resonance_q,
            fm_index: patch.fm_index,
            drive_saturation: patch.drive_saturation,
            extreme_morph: patch.extreme_morph,
            filter2_cutoff_hz: patch.filter2.cutoff_hz,
            filter2_vowel: patch.filter2.vowel_morph,
            granular_mix: patch.granular.mix,
            shimmer_mix: patch.studio_fx.shimmer_mix,
            ott_depth: patch.studio_fx.ott_depth,
            ddsp_morph: patch.ddsp.morph,
        }
    }

    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let k = t.clamp(0.0, 1.0);
        let mix = |x: f32, y: f32| x + (y - x) * k;
        Self {
            name: format!("{} -> {}", a.name, b.name),
            cutoff_hz: mix(a.cutoff_hz, b.cutoff_hz),
            resonance_q: mix(a.resonance_q, b.resonance_q),
            fm_index: mix(a.fm_index, b.fm_index),
            drive_saturation: mix(a.drive_saturation, b.drive_saturation),
            extreme_morph: mix(a.extreme_morph, b.extreme_morph),
            filter2_cutoff_hz: mix(a.filter2_cutoff_hz, b.filter2_cutoff_hz),
            filter2_vowel: mix(a.filter2_vowel, b.filter2_vowel),
            granular_mix: mix(a.granular_mix, b.granular_mix),
            shimmer_mix: mix(a.shimmer_mix, b.shimmer_mix),
            ott_depth: mix(a.ott_depth, b.ott_depth),
            ddsp_morph: mix(a.ddsp_morph, b.ddsp_morph),
        }
    }

    pub fn apply_to(&self, patch: &mut SynthPatch) {
        patch.cutoff_hz = self.cutoff_hz.clamp(40.0, 12000.0);
        patch.resonance_q = self.resonance_q.clamp(0.5, 12.0);
        patch.fm_index = self.fm_index.clamp(0.0, 4.0);
        patch.drive_saturation = self.drive_saturation.clamp(0.0, 3.0);
        patch.extreme_morph = self.extreme_morph.clamp(0.0, 1.0);
        patch.filter2.cutoff_hz = self.filter2_cutoff_hz.clamp(40.0, 16000.0);
        patch.filter2.vowel_morph = self.filter2_vowel.clamp(0.0, 4.0);
        patch.granular.mix = self.granular_mix.clamp(0.0, 1.0);
        patch.studio_fx.shimmer_mix = self.shimmer_mix.clamp(0.0, 1.0);
        patch.studio_fx.ott_depth = self.ott_depth.clamp(0.0, 1.0);
        patch.ddsp.morph = self.ddsp_morph.clamp(0.0, 1.0);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SceneMorphBank {
    pub scene_a: usize,
    pub scene_b: usize,
    pub crossfade: f32, // 0.0 = Scene A, 1.0 = Scene B
    pub xy_x: f32,      // 0.0 ..= 1.0 (Scenes 0/1 vs 2/3)
    pub xy_y: f32,      // 0.0 ..= 1.0 (Scenes 0/2 vs 1/3)
    pub scenes: Vec<PerformanceScene>,
}

impl Default for SceneMorphBank {
    fn default() -> Self {
        Self::factory_club_scenes()
    }
}

impl SceneMorphBank {
    pub fn factory_club_scenes() -> Self {
        let mk = |name: &str, cut, q, fm, drv, morph, f2c, vow, gran, shim, ott, ddsp| {
            PerformanceScene {
                name: name.into(),
                cutoff_hz: cut,
                resonance_q: q,
                fm_index: fm,
                drive_saturation: drv,
                extreme_morph: morph,
                filter2_cutoff_hz: f2c,
                filter2_vowel: vow,
                granular_mix: gran,
                shimmer_mix: shim,
                ott_depth: ott,
                ddsp_morph: ddsp,
            }
        };
        Self {
            scene_a: 0,
            scene_b: 1,
            crossfade: 0.0,
            xy_x: 0.0,
            xy_y: 0.0,
            scenes: vec![
                mk(
                    "1. Intro Filtered",
                    380.0,
                    2.2,
                    0.1,
                    0.2,
                    0.15,
                    700.0,
                    0.2,
                    0.1,
                    0.15,
                    0.2,
                    0.1,
                ),
                mk(
                    "2. Acid Squelch",
                    1450.0,
                    9.5,
                    0.45,
                    1.8,
                    0.55,
                    2200.0,
                    1.1,
                    0.0,
                    0.1,
                    0.55,
                    0.25,
                ),
                mk(
                    "3. Shimmer Cloud",
                    2800.0,
                    3.0,
                    0.3,
                    0.3,
                    0.72,
                    3400.0,
                    2.0,
                    0.65,
                    0.75,
                    0.35,
                    0.6,
                ),
                mk(
                    "4. FM Bell Glass",
                    6400.0,
                    4.5,
                    2.8,
                    0.6,
                    0.85,
                    5200.0,
                    2.8,
                    0.25,
                    0.45,
                    0.40,
                    0.8,
                ),
                mk(
                    "5. Sub Warehouse",
                    220.0,
                    5.0,
                    0.2,
                    2.4,
                    0.30,
                    480.0,
                    3.6,
                    0.0,
                    0.05,
                    0.75,
                    0.05,
                ),
                mk(
                    "6. Formant Talk",
                    1800.0,
                    6.2,
                    0.9,
                    1.1,
                    0.60,
                    1600.0,
                    1.8,
                    0.15,
                    0.20,
                    0.50,
                    0.5,
                ),
                mk(
                    "7. Cosmic Freeze",
                    4200.0,
                    2.0,
                    0.6,
                    0.2,
                    0.92,
                    4800.0,
                    3.2,
                    0.85,
                    0.85,
                    0.30,
                    0.9,
                ),
                mk(
                    "8. Peak Drop OTT",
                    8800.0,
                    8.2,
                    1.9,
                    2.6,
                    0.95,
                    7500.0,
                    0.4,
                    0.10,
                    0.30,
                    0.90,
                    0.4,
                ),
            ],
        }
    }

    pub fn ensure_len(&mut self) {
        if self.scenes.len() < SCENE_COUNT {
            let defaults = Self::factory_club_scenes().scenes;
            let cur = self.scenes.len();
            for item in defaults.into_iter().take(SCENE_COUNT).skip(cur) {
                self.scenes.push(item);
            }
        }
        self.scenes.truncate(SCENE_COUNT);
        self.scene_a = self.scene_a.min(SCENE_COUNT - 1);
        self.scene_b = self.scene_b.min(SCENE_COUNT - 1);
    }

    pub fn blended_crossfade(&self) -> PerformanceScene {
        let a = self.scenes.get(self.scene_a).cloned().unwrap_or_default();
        let b = self.scenes.get(self.scene_b).cloned().unwrap_or_default();
        PerformanceScene::lerp(&a, &b, self.crossfade)
    }

    pub fn blended_xy(&self) -> PerformanceScene {
        let s0 = self.scenes.first().cloned().unwrap_or_default();
        let s1 = self.scenes.get(1).cloned().unwrap_or_default();
        let s2 = self.scenes.get(2).cloned().unwrap_or_default();
        let s3 = self.scenes.get(3).cloned().unwrap_or_default();
        let top = PerformanceScene::lerp(&s0, &s1, self.xy_x);
        let bot = PerformanceScene::lerp(&s2, &s3, self.xy_x);
        PerformanceScene::lerp(&top, &bot, self.xy_y)
    }
}
