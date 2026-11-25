//! Live-instrument glue on `SynthCraftApp`: audio start/restart, MIDI connection, note
//! input from mouse and computer keyboard, transport, and the live meters and scope.

use crate::app::SynthCraftApp;
use crate::audio_io::{list_output_devices, AudioHandle, AudioMsg, SCOPE_LEN};
use crate::midi_io::{self, list_ports};
use eframe::egui;
use std::time::Duration;
use synthcraft_core::dsp::{autocorrelation_pitch, log_bands, magnitude_db};
use synthcraft_core::midi::MidiEvent;
use synthcraft_core::{analyze_patch_spectrum, RtOptions, SpectrumAnalysisReport};

/// Computer-keyboard piano: two rows, semitone offsets from the base C.
const KEY_MAP: [(egui::Key, i32); 25] = [
    (egui::Key::Z, 0),
    (egui::Key::S, 1),
    (egui::Key::X, 2),
    (egui::Key::D, 3),
    (egui::Key::C, 4),
    (egui::Key::V, 5),
    (egui::Key::G, 6),
    (egui::Key::B, 7),
    (egui::Key::H, 8),
    (egui::Key::N, 9),
    (egui::Key::J, 10),
    (egui::Key::M, 11),
    (egui::Key::Q, 12),
    (egui::Key::Num2, 13),
    (egui::Key::W, 14),
    (egui::Key::Num3, 15),
    (egui::Key::E, 16),
    (egui::Key::R, 17),
    (egui::Key::Num5, 18),
    (egui::Key::T, 19),
    (egui::Key::Num6, 20),
    (egui::Key::Y, 21),
    (egui::Key::Num7, 22),
    (egui::Key::U, 23),
    (egui::Key::I, 24),
];

fn db(x: f32) -> f32 {
    20.0 * x.max(1e-4).log10()
}

impl SynthCraftApp {
    pub fn rt_options(&self) -> RtOptions {
        RtOptions {
            a4_hz: self.settings.master_tuning_a4_hz,
            max_voices: self.settings.max_polyphony_voices as usize,
            headroom_db: self.settings.master_headroom_db,
            dc_blocker: self.settings.dc_blocker_enabled,
            soft_clip: self.settings.soft_clip_limiter,
        }
    }

    /// Open the audio output and connect MIDI. Called once by the real binary; tests and
    /// `--smoke` never call it, so they need no sound hardware.
    pub fn start_audio(&mut self, ctx: &egui::Context) {
        self.repaint_ctx = Some(ctx.clone());
        self.restart_audio();
        self.refresh_midi_ports();
        if self.midi.is_none() {
            if let Some(first) = self.midi_ports.first().cloned() {
                self.connect_midi(&first);
            }
        }
    }

    pub fn restart_audio(&mut self) {
        self.audio = None;
        self.audio_devices = list_output_devices();
        let device =
            (!self.settings.output_device.is_empty()).then(|| self.settings.output_device.clone());
        let patch = self.engine.active_patch().clone();
        match AudioHandle::start(
            device.as_deref(),
            Some(self.engine.sample_rate_hz),
            Some(self.settings.buffer_frames),
            patch,
            self.rt_options(),
        ) {
            Ok(h) => {
                self.status_line = format!(
                    "Audio running: {} @ {} Hz, {} ch",
                    h.info.device, h.info.sample_rate, h.info.channels
                );
                self.audio_error = None;
                let routing = synthcraft_core::compute_rack_routing(
                    &self.engine.rack_modules,
                    &self.engine.rack_cables,
                );
                h.send(AudioMsg::Routing(routing));
                self.last_routing = Some(routing);
                self.last_opts = Some(self.rt_options());
                self.last_patch_json =
                    serde_json::to_string(self.engine.active_patch()).unwrap_or_default();
                self.audio = Some(h);
                // Reconnect MIDI so its note path points at the new stream.
                if let Some(m) = self.midi.take() {
                    self.connect_midi(&m.port_name);
                }
            }
            Err(e) => {
                self.audio_error = Some(e.clone());
                self.status_line = format!("Audio unavailable: {e}");
            }
        }
    }

    pub fn refresh_midi_ports(&mut self) {
        self.midi_ports = list_ports();
        self.engine.midi_hw.available_ports = self.midi_ports.clone();
    }

    pub fn connect_midi(&mut self, name: &str) {
        self.midi = None;
        let (Some(audio), Some(ctx)) = (self.audio.as_ref(), self.repaint_ctx.clone()) else {
            self.status_line = "MIDI needs a running audio output".into();
            return;
        };
        match midi_io::connect(
            name,
            audio.sender(),
            self.midi_tx.clone(),
            self.midi_shared.clone(),
            ctx,
        ) {
            Ok(h) => {
                self.status_line = format!("MIDI input connected: {}", h.port_name);
                self.engine.midi_hw.selected_port = h.port_name.clone();
                self.midi = Some(h);
            }
            Err(e) => self.status_line = format!("MIDI: {e}"),
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.engine.midi_hw.last_triggered_note = note;
        if self.seq_record {
            self.record_sequencer_note(note, velocity);
        }
        match &self.audio {
            Some(a) => a.send(AudioMsg::NoteOn(note, velocity)),
            None => self.status_line = "No audio output: notes cannot sound".into(),
        }
    }

    /// Capture a live note into the active patch's 16-step sequencer (overdubbing the playing
    /// step when running, or step-recording and advancing the cursor when stopped).
    pub fn record_sequencer_note(&mut self, note: u8, velocity: f32) {
        let len = self.engine.active_patch().steps.len();
        if len == 0 {
            return;
        }
        let step_idx = match self.playing_step() {
            Some(s) => s % len,
            None => {
                let cur = self.seq_record_cursor % len;
                self.seq_record_cursor = (cur + 1) % len;
                cur
            }
        };
        let before = self.engine.active_patch().clone();
        let mut edited = before.clone();
        if let Some(step) = edited.steps.get_mut(step_idx) {
            step.midi_note = note.clamp(0, 127);
            step.active = true;
            step.velocity = velocity.clamp(0.05, 1.0);
            step.accent = velocity >= 0.88;
        }
        self.apply_patch_edit(&before, &edited);
        self.status_line = format!(
            "Recorded note {} (vel {:.0}%) into step {}",
            note,
            velocity * 100.0,
            step_idx + 1
        );
    }

    pub fn note_off(&mut self, note: u8) {
        if let Some(a) = &self.audio {
            a.send(AudioMsg::NoteOff(note));
        }
    }

    /// Fire one of the 8 Camelot-key-locked live DJ performance pads (triggered either by
    /// DJCraft's Reloop Ready `DJPD` UDP bridge or by the Workbench 5 audition strip).
    pub fn fire_djcraft_pad(&mut self, pad_index: u8, velocity: f32) {
        let root_pc =
            synthcraft_core::camelot_to_root_semitone(&self.engine.active_patch().camelot_key)
                .unwrap_or(9); // default A (8A Am)
        let base_c3 = 48 + root_pc; // C3..B3 root
        let vel = velocity.clamp(0.1, 1.0);
        match pad_index {
            0 => {
                // Sub Drop (root - 12 st)
                self.note_on((base_c3 - 12).clamp(24, 96) as u8, 1.0);
                self.status_line = format!("DJ Pad 1: Sub Drop (MIDI {})", base_c3 - 12);
            }
            1 => {
                self.note_on(base_c3.clamp(24, 96) as u8, vel);
                self.status_line = format!("DJ Pad 2: Root Acid Stab (MIDI {base_c3})");
            }
            2 => {
                self.note_on((base_c3 + 3).clamp(24, 96) as u8, vel);
                self.status_line = format!("DJ Pad 3: Minor 3rd Stab (MIDI {})", base_c3 + 3);
            }
            3 => {
                self.note_on((base_c3 + 7).clamp(24, 96) as u8, vel);
                self.status_line = format!("DJ Pad 4: Fifth Stab (MIDI {})", base_c3 + 7);
            }
            4 => {
                self.note_on((base_c3 + 12).clamp(24, 96) as u8, vel);
                self.status_line = format!("DJ Pad 5: Octave Rave Stab (MIDI {})", base_c3 + 12);
            }
            5 => {
                // Dub Siren High (+19 st + mod wheel LFO depth)
                if let Some(a) = &self.audio {
                    a.send(AudioMsg::ModWheel(0.85));
                }
                self.note_on((base_c3 + 19).clamp(24, 108) as u8, vel);
                self.status_line = "DJ Pad 6: Dub Siren High".into();
            }
            6 => {
                // Riser Sweep: open filter cutoff and trigger root
                let before = self.engine.active_patch().clone();
                let mut edited = before.clone();
                edited.cutoff_hz = (edited.cutoff_hz * 1.6).clamp(120.0, 14000.0);
                self.apply_patch_edit(&before, &edited);
                self.note_on((base_c3 + 12).clamp(24, 96) as u8, 0.95);
                self.status_line = format!(
                    "DJ Pad 7: Riser Sweep (cutoff -> {:.0} Hz)",
                    edited.cutoff_hz
                );
            }
            _ => {
                if let Some(a) = &self.audio {
                    a.send(AudioMsg::AllNotesOff);
                    a.send(AudioMsg::ModWheel(0.0));
                }
                self.status_line = "DJ Pad 8: Panic Cut (All Notes Off)".into();
            }
        }
    }

    pub fn toggle_play(&mut self) {
        match &self.audio {
            Some(a) => {
                let playing = a.meters.playing();
                a.send(AudioMsg::Play(!playing));
                self.status_line = if playing {
                    "Sequencer stopped".into()
                } else {
                    "Sequencer playing".into()
                };
            }
            None => self.status_line = "No audio output: sequencer cannot play".into(),
        }
    }

    pub fn is_playing(&self) -> bool {
        self.audio.as_ref().is_some_and(|a| a.meters.playing())
    }

    pub fn playing_step(&self) -> Option<usize> {
        self.audio
            .as_ref()
            .filter(|a| a.meters.playing())
            .map(|a| a.meters.step())
    }

    /// Per-frame housekeeping: push patch and option changes to the audio thread, apply MIDI
    /// controller moves to the patch, and keep the meters animating.
    pub fn pump_live(&mut self, ctx: &egui::Context) {
        while let Ok(ev) = self.midi_rx.try_recv() {
            self.apply_midi_controller(ev);
        }
        // Poll DJCraft loopback Link UDP beacons (`DJLK`) and Reloop Ready pad events (`DJPD`).
        if self.djcraft_link_sync || self.djcraft_auto_camelot || self.djcraft_pad_listen {
            self.djcraft_link_rx.ensure_bound();
            let (beacon_opt, pad_events) = self.djcraft_link_rx.poll_all();
            if let Some(beacon) = beacon_opt {
                let before = self.engine.active_patch().clone();
                let mut edited = before.clone();
                let mut touched = false;
                if self.djcraft_link_sync && (edited.bpm - beacon.bpm as f32).abs() > 0.05 {
                    edited.bpm = (beacon.bpm as f32).clamp(40.0, 240.0);
                    touched = true;
                }
                if self.djcraft_auto_camelot
                    && !beacon.camelot_key.is_empty()
                    && !beacon
                        .camelot_key
                        .eq_ignore_ascii_case(edited.camelot_key.trim())
                {
                    let shift = synthcraft_core::transpose_patch_to_camelot(
                        &mut edited,
                        &beacon.camelot_key,
                    );
                    if shift != 0 {
                        self.status_line = format!(
                            "Auto-Camelot transposed {:+} st -> {}",
                            shift, edited.camelot_key
                        );
                    }
                    touched = true;
                }
                if touched {
                    self.apply_patch_edit(&before, &edited);
                }
                if self.djcraft_link_sync {
                    if let Some(a) = &self.audio {
                        if a.meters.playing() != beacon.playing {
                            a.send(AudioMsg::Play(beacon.playing));
                        }
                    }
                }
            }
            if self.djcraft_pad_listen {
                for pad in pad_events {
                    if pad.pressed {
                        self.fire_djcraft_pad(pad.pad_index, pad.velocity);
                    }
                }
            }
        }
        let opts = self.rt_options();
        let routing = synthcraft_core::compute_rack_routing(
            &self.engine.rack_modules,
            &self.engine.rack_cables,
        );
        let json = serde_json::to_string(self.engine.active_patch()).unwrap_or_default();
        let patch_changed = json != self.last_patch_json;
        let opts_changed = self.last_opts != Some(opts);
        let routing_changed = self.last_routing != Some(routing);
        self.sync_bank();
        self.sync_bank_b();
        if patch_changed && !self.last_patch_json.is_empty() {
            self.dirty_since.get_or_insert_with(std::time::Instant::now);
        }
        let Some(a) = &self.audio else {
            return;
        };
        a.reclaim();
        if patch_changed {
            a.send(AudioMsg::Patch(Box::new(
                self.engine.active_patch().clone(),
            )));
            self.last_patch_json = json;
        }
        if routing_changed {
            a.send(AudioMsg::Routing(routing));
            self.last_routing = Some(routing);
        }
        if opts_changed {
            a.send(AudioMsg::Options(opts));
            self.last_opts = Some(opts);
        }
        if let Some(e) = a.take_error() {
            self.status_line = format!("Audio stream error: {e}");
        }
        self.midi_shared.channel.store(
            self.engine.midi_hw.midi_channel,
            std::sync::atomic::Ordering::Relaxed,
        );
        self.midi_shared.bend_range.store(
            self.engine.midi_hw.pitch_bend_semitones,
            std::sync::atomic::Ordering::Relaxed,
        );
        ctx.request_repaint_after(Duration::from_millis(33));
    }

    /// Keep the audio thread's sample bank in step with the active patch. Decoding happens on a
    /// worker thread; the finished bank is handed to the audio thread through a message.
    fn sync_bank(&mut self) {
        while let Ok((spec, res)) = self.bank_rx.try_recv() {
            if self.bank_loading.as_deref() == Some(spec.as_str()) {
                self.bank_loading = None;
            }
            self.bank_loaded_spec = Some(spec.clone());
            match res {
                Ok(bank) => {
                    self.bank_status = format!(
                        "Loaded '{}': {} zones, {} frames",
                        bank.name,
                        bank.zones.len(),
                        bank.total_frames()
                    );
                    if let Some(a) = &self.audio {
                        a.send(AudioMsg::Bank(Some(bank)));
                    }
                }
                Err(e) => {
                    self.bank_status = format!("Bank load failed: {e}");
                    if let Some(a) = &self.audio {
                        a.send(AudioMsg::Bank(None));
                    }
                }
            }
        }
        let want = self.engine.active_patch().sampler.bank.clone();
        let settled = want == self.bank_loaded_spec;
        let in_flight = want.is_some() && want == self.bank_loading;
        if settled || in_flight {
            return;
        }
        match want {
            None => {
                self.bank_loaded_spec = None;
                self.bank_loading = None;
                if let Some(a) = &self.audio {
                    a.send(AudioMsg::Bank(None));
                }
            }
            Some(spec) => {
                self.bank_loading = Some(spec.clone());
                self.bank_status = format!("Loading {spec} ...");
                let tx = self.bank_tx.clone();
                std::thread::spawn(move || {
                    let res =
                        synthcraft_core::sampler::load_cached(&spec).map_err(|e| e.to_string());
                    let _ = tx.send((spec, res));
                });
            }
        }
    }

    /// Keep the audio thread's Layer B (AI2 Combi) sample bank in step with the active patch.
    fn sync_bank_b(&mut self) {
        while let Ok((spec, res)) = self.bank_b_rx.try_recv() {
            if self.bank_b_loading.as_deref() == Some(spec.as_str()) {
                self.bank_b_loading = None;
            }
            self.bank_b_loaded_spec = Some(spec.clone());
            match res {
                Ok(bank) => {
                    self.bank_status = format!(
                        "Loaded Layer B '{}': {} zones, {} frames",
                        bank.name,
                        bank.zones.len(),
                        bank.total_frames()
                    );
                    if let Some(a) = &self.audio {
                        a.send(AudioMsg::BankB(Some(bank)));
                    }
                }
                Err(e) => {
                    self.bank_status = format!("Layer B load failed: {e}");
                    if let Some(a) = &self.audio {
                        a.send(AudioMsg::BankB(None));
                    }
                }
            }
        }
        let want = self.engine.active_patch().sampler.bank_b.clone();
        let settled = want == self.bank_b_loaded_spec;
        let in_flight = want.is_some() && want == self.bank_b_loading;
        if settled || in_flight {
            return;
        }
        match want {
            None => {
                self.bank_b_loaded_spec = None;
                self.bank_b_loading = None;
                if let Some(a) = &self.audio {
                    a.send(AudioMsg::BankB(None));
                }
            }
            Some(spec) => {
                self.bank_b_loading = Some(spec.clone());
                self.bank_status = format!("Loading Layer B {spec} ...");
                let tx = self.bank_b_tx.clone();
                std::thread::spawn(move || {
                    let res =
                        synthcraft_core::sampler::load_cached(&spec).map_err(|e| e.to_string());
                    let _ = tx.send((spec, res));
                });
            }
        }
    }

    fn apply_midi_controller(&mut self, ev: MidiEvent) {
        if let MidiEvent::NoteOn { note, velocity, .. } = ev {
            self.engine.midi_hw.last_triggered_note = note;
            if self.seq_record {
                self.record_sequencer_note(note, velocity);
            }
            return;
        }
        let MidiEvent::Controller { cc, value, .. } = ev else {
            return;
        };
        let exp = |lo: f32, hi: f32| lo * (hi / lo).powf(value);
        let before = self.engine.active_patch().clone();
        let mut edited = before.clone();
        let p = &mut edited;
        match cc {
            74 => p.cutoff_hz = exp(35.0, 16000.0),
            71 => p.resonance_q = 0.5 + value * 11.5,
            73 => p.attack_ms = exp(1.0, 2000.0),
            72 => p.release_ms = exp(5.0, 4000.0),
            91 => p.extreme_morph = value,
            _ => {}
        }
        self.apply_patch_edit(&before, &edited);
    }

    /// Computer-keyboard piano and transport. Ignored while a text field has focus.
    pub fn handle_keyboard(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let events = ctx.input(|i| i.events.clone());
        for ev in events {
            let egui::Event::Key {
                key,
                pressed,
                repeat,
                modifiers,
                ..
            } = ev
            else {
                continue;
            };
            if modifiers.ctrl || modifiers.command || modifiers.alt {
                continue;
            }
            if pressed && !repeat {
                match key {
                    egui::Key::Space => {
                        self.toggle_play();
                        continue;
                    }
                    egui::Key::Minus => {
                        self.kbd_octave = (self.kbd_octave - 1).max(0);
                        self.status_line = format!("Keyboard octave C{}", self.kbd_octave);
                        continue;
                    }
                    egui::Key::Equals => {
                        self.kbd_octave = (self.kbd_octave + 1).min(7);
                        self.status_line = format!("Keyboard octave C{}", self.kbd_octave);
                        continue;
                    }
                    _ => {}
                }
            }
            let Some(&(_, offset)) = KEY_MAP.iter().find(|(k, _)| *k == key) else {
                continue;
            };
            if pressed && !repeat {
                let note = (12 * (self.kbd_octave + 1) + offset).clamp(0, 127) as u8;
                self.kbd_down.insert(key, note);
                self.note_on(note, 0.85);
            } else if !pressed {
                if let Some(note) = self.kbd_down.remove(&key) {
                    self.note_off(note);
                }
            }
        }
    }

    /// RMS and peak in dBFS: live from the output ring when audio runs, else a cached offline render.
    pub fn level_dbfs(&mut self) -> (f32, f32) {
        if let Some(a) = &self.audio {
            let recent = a.meters.recent(1024);
            if !recent.is_empty() {
                let rms = (recent.iter().map(|s| s * s).sum::<f32>() / recent.len() as f32).sqrt();
                let peak = recent.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
                return (db(rms).max(-80.0), db(peak).max(-80.0));
            }
        }
        let rep = self.cached_analysis(80);
        (rep.rms_dbfs, rep.peak_dbfs)
    }

    /// Offline analysis, recomputed only when the patch, rate or duration changes.
    pub fn cached_analysis(&mut self, duration_ms: u32) -> SpectrumAnalysisReport {
        let json = serde_json::to_string(self.engine.active_patch()).unwrap_or_default();
        let key = (json, duration_ms, self.engine.sample_rate_hz);
        if let Some((k, rep)) = &self.analysis_cache {
            if *k == key {
                return rep.clone();
            }
        }
        let rep = analyze_patch_spectrum(self.engine.active_patch(), key.2, duration_ms);
        self.analysis_cache = Some((key, rep.clone()));
        rep
    }

    /// Scope and spectrum of what the audio device is playing right now, or `None` without audio.
    pub fn live_report(&self) -> Option<SpectrumAnalysisReport> {
        let a = self.audio.as_ref()?;
        let block = a.meters.recent(4096.min(SCOPE_LEN));
        if block.len() < 4096 {
            return None;
        }
        let sr = a.info.sample_rate as f32;
        let energy: f32 = block.iter().map(|s| s * s).sum();
        let rms = (energy / block.len() as f32).sqrt();
        let peak = block.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
        let pitch = if rms > 1e-3 {
            autocorrelation_pitch(&block[block.len() - 2048..], sr)
        } else {
            0.0
        };
        let from = block
            .windows(2)
            .take(2048)
            .position(|w| w[0] < 0.0 && w[1] >= 0.0)
            .unwrap_or(0);
        Some(SpectrumAnalysisReport {
            patch_id: self.engine.active_patch_id.clone(),
            sample_rate_hz: a.info.sample_rate,
            sample_count: block.len(),
            rms_dbfs: db(rms).max(-80.0),
            peak_dbfs: db(peak).max(-80.0),
            crest_factor_db: (db(peak) - db(rms)).max(0.0),
            estimated_pitch_hz: pitch,
            scope_samples: block.iter().skip(from).take(256).copied().collect(),
            spectrum_bands_db: log_bands(&magnitude_db(&block), sr / block.len() as f32),
        })
    }

    /// Real MIDI input picker plus channel and bend-range controls.
    pub fn render_midi_port_picker(&mut self, ui: &mut egui::Ui) {
        let ports = self.midi_ports.clone();
        let current = if self.midi.is_some() {
            self.engine.midi_hw.selected_port.clone()
        } else {
            "(none connected)".to_string()
        };
        let mut pick: Option<String> = None;
        ui.horizontal(|ui| {
            egui::ComboBox::from_label("MIDI Input Device")
                .selected_text(current)
                .show_ui(ui, |ui| {
                    for p in &ports {
                        if ui.selectable_label(false, p).clicked() {
                            pick = Some(p.clone());
                        }
                    }
                });
            if ui.button("Rescan").clicked() {
                self.refresh_midi_ports();
            }
        });
        if ports.is_empty() {
            ui.small("No MIDI input devices found. Plug one in and press Rescan.");
        }
        if let Some(p) = pick {
            self.connect_midi(&p);
        }
        ui.add(
            egui::Slider::new(&mut self.engine.midi_hw.midi_channel, 0..=16)
                .text("Receive channel (0 = omni)"),
        );
        ui.add(
            egui::Slider::new(&mut self.engine.midi_hw.pitch_bend_semitones, 1..=24)
                .text("Pitch bend range (+/- st)"),
        );
    }
}
