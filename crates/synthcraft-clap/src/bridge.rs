//! Shared lock-free / try-lock bridge between the CLAP audio/host layer and the plugin GUI.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use synthcraft_core::params::PARAMS;
use synthcraft_core::plugin::PluginEngine;

/// Number of automatable parameters in `PARAMS`.
pub(crate) const N_PARAMS: usize = PARAMS.len();

/// Number of samples in the live GUI oscilloscope snapshot.
pub(crate) const SCOPE_SAMPLES: usize = 128;

/// Events pushed from the GUI thread and drained on the audio thread in `process`.
#[derive(Debug, Clone, Copy)]
pub(crate) enum UiEvent {
    ParamGestureBegin(u32),
    ParamValue(u32, f32),
    ParamGestureEnd(u32),
    NoteOn(u8, f32),
    NoteOff(u8),
}

/// Shared state between `Plugin` (audio + CLAP callbacks) and the embedded GUI.
pub(crate) struct GuiBridge {
    pub(crate) engine: Mutex<PluginEngine>,
    /// Latest value of every parameter (f32 bits), readable without touching the engine lock.
    pub(crate) values: [AtomicU32; N_PARAMS],
    pub(crate) peak_l: AtomicU32,
    pub(crate) peak_r: AtomicU32,
    pub(crate) active_voices: AtomicU32,
    pub(crate) scope: [AtomicU32; SCOPE_SAMPLES],
    ui_events: Mutex<Vec<UiEvent>>,
    pub(crate) bank_status: Mutex<String>,
}

impl GuiBridge {
    pub(crate) fn new(sample_rate: f32) -> Self {
        let values = std::array::from_fn(|i| AtomicU32::new(PARAMS[i].default.to_bits()));
        let scope = std::array::from_fn(|_| AtomicU32::new(0.0_f32.to_bits()));
        Self {
            engine: Mutex::new(PluginEngine::new(sample_rate)),
            values,
            peak_l: AtomicU32::new(0.0_f32.to_bits()),
            peak_r: AtomicU32::new(0.0_f32.to_bits()),
            active_voices: AtomicU32::new(0),
            scope,
            ui_events: Mutex::new(Vec::with_capacity(64)),
            bank_status: Mutex::new(String::from("No sample bank loaded")),
        }
    }

    pub(crate) fn sync_values(&self, engine: &PluginEngine) {
        for (slot, def) in self.values.iter().zip(PARAMS) {
            if let Some(v) = engine.get_param(def.id) {
                slot.store(v.to_bits(), Ordering::Relaxed);
            }
        }
    }

    pub(crate) fn get_value_by_index(&self, idx: usize) -> f32 {
        self.values
            .get(idx)
            .map_or(0.0, |a| f32::from_bits(a.load(Ordering::Relaxed)))
    }

    pub(crate) fn store_value_by_id(&self, id: u32, v: f32) {
        if let Some(i) = PARAMS.iter().position(|d| d.id == id) {
            self.values[i].store(v.to_bits(), Ordering::Relaxed);
        }
    }

    pub(crate) fn push_ui_event(&self, ev: UiEvent) {
        if let UiEvent::ParamValue(id, v) = ev {
            self.store_value_by_id(id, v);
        }
        if let Ok(mut q) = self.ui_events.lock() {
            if q.len() < 256 {
                q.push(ev);
            }
        }
    }

    /// Non-blocking drain for the audio thread (`process`).
    pub(crate) fn drain_ui_events_try(&self, out: &mut Vec<UiEvent>) {
        if let Ok(mut q) = self.ui_events.try_lock() {
            out.append(&mut *q);
        }
    }

    /// Audio-thread lock-free update of peak meters and oscilloscope snapshot.
    pub(crate) fn update_meters(&self, left: &[f32], right: &[f32], voices: usize) {
        let mut pl = 0.0_f32;
        let mut pr = 0.0_f32;
        for (&l, &r) in left.iter().zip(right.iter()) {
            pl = pl.max(l.abs());
            pr = pr.max(r.abs());
        }
        // Smooth decay so peaks are readable at 60 fps
        let prev_l = f32::from_bits(self.peak_l.load(Ordering::Relaxed)) * 0.85;
        let prev_r = f32::from_bits(self.peak_r.load(Ordering::Relaxed)) * 0.85;
        self.peak_l
            .store(pl.max(prev_l).to_bits(), Ordering::Relaxed);
        self.peak_r
            .store(pr.max(prev_r).to_bits(), Ordering::Relaxed);
        self.active_voices.store(voices as u32, Ordering::Relaxed);

        let n = left.len().min(right.len());
        if n > 0 {
            let stride = (n / SCOPE_SAMPLES).max(1);
            for (i, slot) in self.scope.iter().enumerate() {
                let idx = (i * stride).min(n - 1);
                let mono = 0.5 * (left[idx] + right[idx]);
                slot.store(mono.to_bits(), Ordering::Relaxed);
            }
        }
    }

    pub(crate) fn read_scope(&self) -> [f32; SCOPE_SAMPLES] {
        std::array::from_fn(|i| f32::from_bits(self.scope[i].load(Ordering::Relaxed)))
    }
}
