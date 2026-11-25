//! Safe trait and configuration types for hosting external `.clap` plugins inside a 3U rack slot.
//!
//! `synthcraft-core` stays `#![forbid(unsafe_code)]`; the actual native CLAP 1.2.2 C-ABI
//! dynamic loader lives in `synthcraft-clap-host` and implements [`ExternalClapProcessor`].

use serde::{Deserialize, Serialize};

/// Metadata describing a discovered or loaded CLAP plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClapPluginInfo {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub path: String,
    pub plugin_index: u32,
}

/// Metadata for one automatable parameter exposed by a hosted CLAP plugin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClapParamInfo {
    pub id: u32,
    pub name: String,
    pub min_value: f64,
    pub max_value: f64,
    pub default_value: f64,
}

/// Persisted configuration for the 3U `CLAP HOST` rack module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClapHostConfig {
    /// Filesystem path to the hosted `.clap` plugin binary, if any.
    #[serde(default)]
    pub plugin_path: Option<String>,
    /// Index of the plugin inside multi-plugin `.clap` bundles.
    #[serde(default)]
    pub plugin_index: u32,
    /// Optional CLAP parameter ID modulated by the rack's `PARAM CV` jack.
    #[serde(default)]
    pub cv_param_id: Option<u32>,
    /// Wet/dry insert mix (`0.0` = dry, `1.0` = 100% wet).
    #[serde(default = "default_one")]
    pub mix: f32,
    /// Bipolar CV modulation depth (`-1.0..=1.0`) applied to `cv_param_id`.
    #[serde(default = "default_half")]
    pub cv_depth: f32,
}

fn default_one() -> f32 {
    1.0
}

fn default_half() -> f32 {
    0.5
}

impl Default for ClapHostConfig {
    fn default() -> Self {
        Self {
            plugin_path: None,
            plugin_index: 0,
            cv_param_id: None,
            mix: 1.0,
            cv_depth: 0.5,
        }
    }
}

/// Real-time audio processor trait implemented by a loaded external `.clap` plugin instance.
///
/// Implementations must be allocation-free inside [`process_block`](Self::process_block)
/// so `SynthRt` can call them directly on the audio thread.
pub trait ExternalClapProcessor: Send {
    /// Plugin descriptor metadata.
    fn info(&self) -> &ClapPluginInfo;

    /// Automatable parameters exported by the plugin's `clap.params` extension.
    fn params(&self) -> &[ClapParamInfo];

    /// Queue a parameter value change for the next audio block.
    fn set_param(&mut self, param_id: u32, value: f64);

    /// Process one stereo block in-place (`left.len() == right.len()`).
    ///
    /// When `cv_mod` is `Some((param_id, amount))`, the host injects a
    /// `CLAP_EVENT_PARAM_MOD` event for that parameter at frame 0 of the block.
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32], cv_mod: Option<(u32, f64)>);
}
