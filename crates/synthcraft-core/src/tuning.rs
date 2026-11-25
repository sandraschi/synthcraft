//! Microtonal tuning engine with Scala (`.scl`) file parser and built-in tuning presets.
//!
//! Precomputes a 128-note frequency lookup table (`[f32; 128]`) so real-time voice pitch
//! lookup in `SynthRt` is a single O(1) array index with zero allocations.

use crate::error::SynthError;
use serde::{Deserialize, Serialize};

/// Built-in microtonal scales plus arbitrary custom Scala `.scl` support.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TuningPreset {
    #[default]
    Equal12,
    Just5Limit,
    Pythagorean,
    Equal19,
    Equal24QuarterTone,
    Equal31,
    BohlenPierce,
    CustomScala(String),
}

impl TuningPreset {
    pub const BUILTINS: [(&'static str, &'static str); 7] = [
        ("equal_12", "12-TET Standard Equal"),
        ("just_5_limit", "Just Intonation (5-Limit)"),
        ("pythagorean", "Pythagorean 12-Tone"),
        ("equal_19", "19-TET Meantone"),
        ("equal_24", "24-TET Quarter-Tone"),
        ("equal_31", "31-TET Extended Meantone"),
        ("bohlen_pierce", "Bohlen-Pierce (Tritave 13-Step)"),
    ];

    pub fn id(&self) -> &str {
        match self {
            Self::Equal12 => "equal_12",
            Self::Just5Limit => "just_5_limit",
            Self::Pythagorean => "pythagorean",
            Self::Equal19 => "equal_19",
            Self::Equal24QuarterTone => "equal_24",
            Self::Equal31 => "equal_31",
            Self::BohlenPierce => "bohlen_pierce",
            Self::CustomScala(_) => "custom_scala",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Equal12 => "12-TET Standard Equal",
            Self::Just5Limit => "Just Intonation (5-Limit)",
            Self::Pythagorean => "Pythagorean 12-Tone",
            Self::Equal19 => "19-TET Meantone",
            Self::Equal24QuarterTone => "24-TET Quarter-Tone",
            Self::Equal31 => "31-TET Extended Meantone",
            Self::BohlenPierce => "Bohlen-Pierce (Tritave)",
            Self::CustomScala(_) => "Custom Scala (.scl)",
        }
    }

    pub fn parse_builtin(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "equal_12" | "12tet" | "12-tet" | "standard" => Some(Self::Equal12),
            "just_5_limit" | "just" | "ji" => Some(Self::Just5Limit),
            "pythagorean" | "pyth" => Some(Self::Pythagorean),
            "equal_19" | "19tet" | "19-tet" => Some(Self::Equal19),
            "equal_24" | "24tet" | "24-tet" | "quarter" => Some(Self::Equal24QuarterTone),
            "equal_31" | "31tet" | "31-tet" => Some(Self::Equal31),
            "bohlen_pierce" | "bp" | "tritave" => Some(Self::BohlenPierce),
            _ => None,
        }
    }
}

/// Parse a standard Scala `.scl` text into positive frequency ratios per degree (ending with
/// the period ratio, e.g. `2.0` for an octave or `3.0` for a tritave).
pub fn parse_scala_scl(text: &str) -> Result<Vec<f64>, SynthError> {
    let mut lines = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('!'));
    let _description = lines
        .next()
        .ok_or_else(|| SynthError::Validation("Empty Scala .scl file".into()))?;
    let count_line = lines
        .next()
        .ok_or_else(|| SynthError::Validation("Missing note count in Scala .scl".into()))?;
    let expected: usize = count_line
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| SynthError::Validation(format!("Invalid Scala count '{count_line}'")))?;
    if expected == 0 || expected > 128 {
        return Err(SynthError::Validation(format!(
            "Scala note count {expected} out of range (1..=128)"
        )));
    }
    let mut ratios = Vec::with_capacity(expected);
    for raw in lines.take(expected) {
        let token = raw.split_whitespace().next().unwrap_or("");
        let ratio = if token.contains('.') {
            let cents: f64 = token
                .parse()
                .map_err(|_| SynthError::Validation(format!("Bad Scala cents '{token}'")))?;
            2.0_f64.powf(cents / 1200.0)
        } else if let Some((num_s, den_s)) = token.split_once('/') {
            let num: f64 = num_s
                .parse()
                .map_err(|_| SynthError::Validation(format!("Bad Scala ratio '{token}'")))?;
            let den: f64 = den_s
                .parse()
                .map_err(|_| SynthError::Validation(format!("Bad Scala ratio '{token}'")))?;
            if den <= 0.0 || num <= 0.0 {
                return Err(SynthError::Validation(format!(
                    "Non-positive Scala ratio '{token}'"
                )));
            }
            num / den
        } else {
            let int_ratio: f64 = token
                .parse()
                .map_err(|_| SynthError::Validation(format!("Bad Scala entry '{token}'")))?;
            if int_ratio <= 0.0 {
                return Err(SynthError::Validation(format!(
                    "Non-positive Scala entry '{token}'"
                )));
            }
            int_ratio
        };
        ratios.push(ratio);
    }
    if ratios.len() != expected {
        return Err(SynthError::Validation(format!(
            "Expected {expected} Scala intervals, found {}",
            ratios.len()
        )));
    }
    Ok(ratios)
}

fn preset_ratios(preset: &TuningPreset) -> Vec<f64> {
    match preset {
        TuningPreset::Equal12 => (1..=12)
            .map(|i| 2.0_f64.powf(f64::from(i) / 12.0))
            .collect(),
        TuningPreset::Just5Limit => vec![
            16.0 / 15.0,
            9.0 / 8.0,
            6.0 / 5.0,
            5.0 / 4.0,
            4.0 / 3.0,
            45.0 / 32.0,
            3.0 / 2.0,
            8.0 / 5.0,
            5.0 / 3.0,
            9.0 / 5.0,
            15.0 / 8.0,
            2.0,
        ],
        TuningPreset::Pythagorean => vec![
            256.0 / 243.0,
            9.0 / 8.0,
            32.0 / 27.0,
            81.0 / 64.0,
            4.0 / 3.0,
            729.0 / 512.0,
            3.0 / 2.0,
            128.0 / 81.0,
            27.0 / 16.0,
            16.0 / 9.0,
            243.0 / 128.0,
            2.0,
        ],
        TuningPreset::Equal19 => (1..=19)
            .map(|i| 2.0_f64.powf(f64::from(i) / 19.0))
            .collect(),
        TuningPreset::Equal24QuarterTone => (1..=24)
            .map(|i| 2.0_f64.powf(f64::from(i) / 24.0))
            .collect(),
        TuningPreset::Equal31 => (1..=31)
            .map(|i| 2.0_f64.powf(f64::from(i) / 31.0))
            .collect(),
        TuningPreset::BohlenPierce => (1..=13)
            .map(|i| 3.0_f64.powf(f64::from(i) / 13.0))
            .collect(),
        TuningPreset::CustomScala(scl) => parse_scala_scl(scl).unwrap_or_else(|_| {
            (1..=12)
                .map(|i| 2.0_f64.powf(f64::from(i) / 12.0))
                .collect()
        }),
    }
}

/// Build a 128-note frequency table anchored at MIDI note 69 (`A4 = a4_hz`, or C4 = 261.63 Hz
/// scaled by `a4_hz / 440.0`).
pub fn build_pitch_table(preset: &TuningPreset, a4_hz: f32) -> [f32; 128] {
    let ratios = preset_ratios(preset);
    let n = ratios.len().max(1) as i32;
    let period = *ratios.last().unwrap_or(&2.0);
    // Anchor at C4 (MIDI 60) scaled relative to A4 tuning reference.
    let c4_hz = f64::from(a4_hz.clamp(400.0, 480.0)) * 2.0_f64.powf(-9.0 / 12.0);
    let mut table = [440.0_f32; 128];
    for (midi, slot) in table.iter_mut().enumerate() {
        let diff = (midi as i32) - 60;
        let oct = diff.div_euclid(n);
        let deg = diff.rem_euclid(n) as usize;
        let step_ratio = if deg == 0 { 1.0 } else { ratios[deg - 1] };
        let hz = c4_hz * period.powi(oct) * step_ratio;
        *slot = (hz as f32).clamp(8.0, 22_000.0);
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scala_parser_handles_cents_ratios_and_comments() {
        let scl = "! just_test.scl\n5-limit triad\n3\n!\n5/4\n3/2\n1200.0\n";
        let ratios = parse_scala_scl(scl).unwrap_or_default();
        assert_eq!(ratios.len(), 3);
        assert!((ratios[0] - 1.25).abs() < 1e-6);
        assert!((ratios[1] - 1.5).abs() < 1e-6);
        assert!((ratios[2] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn pitch_table_matches_a4_440_for_equal_12_and_differs_for_just_5_limit() {
        let eq = build_pitch_table(&TuningPreset::Equal12, 440.0);
        assert!((eq[69] - 440.0).abs() < 0.05);
        let ji = build_pitch_table(&TuningPreset::Just5Limit, 440.0);
        // Just major third (5/4 = 1.25) vs equal (2^(4/12) ~= 1.2599)
        assert!((ji[64] / ji[60] - 1.25).abs() < 1e-4);
        assert!((ji[64] - eq[64]).abs() > 1.5);
    }
}
