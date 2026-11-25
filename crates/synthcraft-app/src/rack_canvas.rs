//! Multi-row scrollable 3U Eurorack skiff, categorized Module Browser,
//! interactive rotary knobs, inline 3U oscilloscope, Euclidean rhythm ring,
//! and cross-row draggable catenary patch cables.

use crate::app::SynthCraftApp;
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use std::collections::HashMap;
use std::f32::consts::PI;
use synthcraft_core::{
    euclidean_pattern, Command, ExtremeDspMode, FilterMode, PcmProgram, RackCategory,
    RackModuleInstance, RackModuleKind, SynthPatch, WaveformKind,
};

/// Active drag state for interactive patch-cable creation or re-patching on the 3U rack.
#[derive(Debug, Clone)]
pub struct CableDrag {
    pub anchor_module: String,
    pub anchor_jack: String,
    pub anchor_is_out: bool,
    pub color_name: String,
    pub gain: f32,
}

#[derive(Clone, Copy)]
enum RackKnobId {
    Bpm,
    PulseWidth,
    Detune,
    Morph,
    RomMix,
    VcoMix,
    Cutoff,
    Resonance,
    LfoDepth,
    Decay,
    Drive,
    Master,
}

impl RackKnobId {
    fn get_norm(self, p: &SynthPatch) -> f32 {
        match self {
            Self::Bpm => ((p.bpm - 40.0) / 200.0).clamp(0.0, 1.0),
            Self::PulseWidth => ((p.pulse_width - 0.05) / 0.90).clamp(0.0, 1.0),
            Self::Detune => ((p.vco2_detune_cents + 100.0) / 200.0).clamp(0.0, 1.0),
            Self::Morph => p.extreme_morph.clamp(0.0, 1.0),
            Self::RomMix => p.m1_layer_mix.clamp(0.0, 1.0),
            Self::VcoMix => p.vco_mix.clamp(0.0, 1.0),
            Self::Cutoff => {
                let lo = 35.0_f32.ln();
                let hi = 16000.0_f32.ln();
                ((p.cutoff_hz.clamp(35.0, 16000.0).ln() - lo) / (hi - lo)).clamp(0.0, 1.0)
            }
            Self::Resonance => ((p.resonance_q - 0.5) / 11.5).clamp(0.0, 1.0),
            Self::LfoDepth => p.lfo_depth.clamp(0.0, 1.0),
            Self::Decay => ((p.decay_ms - 5.0) / 3995.0).clamp(0.0, 1.0),
            Self::Drive => (p.drive_saturation / 3.0).clamp(0.0, 1.0),
            Self::Master => p.master_gain.clamp(0.0, 1.0),
        }
    }

    fn set_norm(self, p: &mut SynthPatch, t: f32) {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Bpm => p.bpm = 40.0 + t * 200.0,
            Self::PulseWidth => p.pulse_width = 0.05 + t * 0.90,
            Self::Detune => p.vco2_detune_cents = -100.0 + t * 200.0,
            Self::Morph => p.extreme_morph = t,
            Self::RomMix => p.m1_layer_mix = t,
            Self::VcoMix => p.vco_mix = t,
            Self::Cutoff => {
                let lo = 35.0_f32.ln();
                let hi = 16000.0_f32.ln();
                p.cutoff_hz = (lo + t * (hi - lo)).exp();
            }
            Self::Resonance => p.resonance_q = 0.5 + t * 11.5,
            Self::LfoDepth => p.lfo_depth = t,
            Self::Decay => p.decay_ms = 5.0 + t * 3995.0,
            Self::Drive => p.drive_saturation = t * 3.0,
            Self::Master => p.master_gain = t,
        }
    }

    fn reset(self, p: &mut SynthPatch) {
        match self {
            Self::Bpm => p.bpm = 128.0,
            Self::PulseWidth => p.pulse_width = 0.5,
            Self::Detune => p.vco2_detune_cents = 12.0,
            Self::Morph => p.extreme_morph = 0.55,
            Self::RomMix => p.m1_layer_mix = 0.55,
            Self::VcoMix => p.vco_mix = 0.5,
            Self::Cutoff => p.cutoff_hz = 1200.0,
            Self::Resonance => p.resonance_q = 3.5,
            Self::LfoDepth => p.lfo_depth = 0.2,
            Self::Decay => p.decay_ms = 180.0,
            Self::Drive => p.drive_saturation = 0.0,
            Self::Master => p.master_gain = 0.7,
        }
    }

    fn format_value(self, p: &SynthPatch) -> String {
        match self {
            Self::Bpm => format!("{:.1} BPM", p.bpm),
            Self::PulseWidth => format!("{:.0}%", p.pulse_width * 100.0),
            Self::Detune => format!("{:+.1} ct", p.vco2_detune_cents),
            Self::Morph => format!("{:.0}%", p.extreme_morph * 100.0),
            Self::RomMix => format!("{:.0}%", p.m1_layer_mix * 100.0),
            Self::VcoMix => format!("{:.0}%", p.vco_mix * 100.0),
            Self::Cutoff => format!("{:.0} Hz", p.cutoff_hz),
            Self::Resonance => format!("Q {:.2}", p.resonance_q),
            Self::LfoDepth => format!("{:.0}%", p.lfo_depth * 100.0),
            Self::Decay => format!("{:.0} ms", p.decay_ms),
            Self::Drive => format!("{:.2}", p.drive_saturation),
            Self::Master => format!("{:.0}%", p.master_gain * 100.0),
        }
    }
}

#[derive(Clone, Copy)]
enum KnobBinding {
    Patch(RackKnobId),
    ModuleA,
    ModuleB,
}

fn module_knobs(kind: RackModuleKind) -> Option<[(&'static str, KnobBinding); 2]> {
    match kind {
        RackModuleKind::Vco1 => Some([
            ("BPM", KnobBinding::Patch(RackKnobId::Bpm)),
            ("PWM", KnobBinding::Patch(RackKnobId::PulseWidth)),
        ]),
        RackModuleKind::Vco2Xdsp => Some([
            ("DETUNE", KnobBinding::Patch(RackKnobId::Detune)),
            ("MORPH", KnobBinding::Patch(RackKnobId::Morph)),
        ]),
        RackModuleKind::PcmWorkstation => Some([
            ("ROM MIX", KnobBinding::Patch(RackKnobId::RomMix)),
            ("VCO MIX", KnobBinding::Patch(RackKnobId::VcoMix)),
        ]),
        RackModuleKind::VcfLadder => Some([
            ("CUTOFF", KnobBinding::Patch(RackKnobId::Cutoff)),
            ("RES Q", KnobBinding::Patch(RackKnobId::Resonance)),
        ]),
        RackModuleKind::LfoAdsr => Some([
            ("LFO DEPTH", KnobBinding::Patch(RackKnobId::LfoDepth)),
            ("DECAY", KnobBinding::Patch(RackKnobId::Decay)),
        ]),
        RackModuleKind::VcaMaster => Some([
            ("DRIVE", KnobBinding::Patch(RackKnobId::Drive)),
            ("MASTER", KnobBinding::Patch(RackKnobId::Master)),
        ]),
        RackModuleKind::WavefolderLpg => Some([
            ("FOLD", KnobBinding::ModuleA),
            ("SYM", KnobBinding::ModuleB),
        ]),
        RackModuleKind::SampleHoldSlew => Some([
            ("RATE", KnobBinding::ModuleA),
            ("SLEW", KnobBinding::ModuleB),
        ]),
        RackModuleKind::EuclideanClock => Some([
            ("STEPS", KnobBinding::ModuleA),
            ("PULSES", KnobBinding::ModuleB),
        ]),
        RackModuleKind::StereoDelay => Some([
            ("TIME", KnobBinding::ModuleA),
            ("FDBK", KnobBinding::ModuleB),
        ]),
        RackModuleKind::Attenuverter => Some([
            ("ATTEN 1", KnobBinding::ModuleA),
            ("ATTEN 2", KnobBinding::ModuleB),
        ]),
        RackModuleKind::InlineScope => None,
    }
}

fn format_custom_knob(kind: RackModuleKind, is_a: bool, v: f32) -> String {
    match (kind, is_a) {
        (RackModuleKind::WavefolderLpg, true) => format!("{:.0}%", v * 100.0),
        (RackModuleKind::WavefolderLpg, false) => format!("{:+.0}%", (v - 0.5) * 200.0),
        (RackModuleKind::SampleHoldSlew, true) => format!("{:.1} Hz", 0.5 + v * 19.5),
        (RackModuleKind::SampleHoldSlew, false) => format!("{:.0} ms", v * 250.0),
        (RackModuleKind::EuclideanClock, true) => {
            format!("{} steps", (2.0 + v * 14.0).round() as usize)
        }
        (RackModuleKind::EuclideanClock, false) => {
            format!("{} pulses", (1.0 + v * 15.0).round() as usize)
        }
        (RackModuleKind::StereoDelay, true) => format!("{:.0} ms", (0.06 + v * 0.69) * 1000.0),
        (RackModuleKind::StereoDelay, false) => format!("{:.0}%", v * 88.0),
        (RackModuleKind::Attenuverter, _) => format!("{:+.0}%", (v - 0.5) * 200.0),
        _ => format!("{:.0}%", v * 100.0),
    }
}

fn module_subtitle(m: &RackModuleInstance, patch: &SynthPatch) -> String {
    if m.bypassed {
        return "BYPASSED".to_string();
    }
    match m.kind {
        RackModuleKind::Vco1 => patch.vco1_wave.as_str().to_uppercase(),
        RackModuleKind::Vco2Xdsp => patch.extreme_dsp.as_str().to_uppercase(),
        RackModuleKind::PcmWorkstation => patch
            .m1_program
            .map(PcmProgram::as_str)
            .unwrap_or("OFF")
            .to_uppercase(),
        RackModuleKind::VcfLadder => patch.filter_mode.as_str().to_uppercase(),
        RackModuleKind::LfoAdsr => format!("{:.1} Hz", patch.lfo_rate_hz),
        RackModuleKind::VcaMaster => format!("{} KEY", patch.camelot_key),
        RackModuleKind::WavefolderLpg => format!("FOLD {:.0}%", m.param_a * 100.0),
        RackModuleKind::SampleHoldSlew => format!("{:.1} Hz", 0.5 + m.param_a * 19.5),
        RackModuleKind::EuclideanClock => {
            let s = (2.0 + m.param_a * 14.0).round() as usize;
            let p = ((1.0 + m.param_b * 15.0).round() as usize).min(s);
            format!("E({p},{s})")
        }
        RackModuleKind::StereoDelay => format!("{:.0} ms", (0.06 + m.param_a * 0.69) * 1000.0),
        RackModuleKind::Attenuverter => format!("{:+.0}%", (m.param_a - 0.5) * 200.0),
        RackModuleKind::InlineScope => "LIVE SCOPE".to_string(),
    }
}

fn canonical_rack_module(m: &str) -> &str {
    match m {
        "LFO-MOD" | "ADSR-ENV" => "LFO / ADSR",
        other => other,
    }
}

fn cable_color(
    name: &str,
    green: Color32,
    amber: Color32,
    purple: Color32,
    red: Color32,
    cyan: Color32,
) -> Color32 {
    match name {
        "green" => green,
        "amber" => amber,
        "purple" => purple,
        "red" => red,
        _ => cyan,
    }
}

fn draw_catenary(painter: &egui::Painter, p1: Pos2, p2: Pos2, col: Color32, width: f32) {
    let sag = 34.0 + ((p2.x - p1.x).abs() * 0.12);
    let ctrl = Pos2::new((p1.x + p2.x) * 0.5, p1.y.max(p2.y) + sag);
    let steps = 18;
    let shadow = Color32::from_rgba_unmultiplied(0, 0, 0, 95);
    let mut prev = p1;
    for s in 1..=steps {
        let t = (s as f32) / (steps as f32);
        let inv = 1.0 - t;
        let pt = Pos2::new(
            inv * inv * p1.x + 2.0 * inv * t * ctrl.x + t * t * p2.x,
            inv * inv * p1.y + 2.0 * inv * t * ctrl.y + t * t * p2.y,
        );
        painter.line_segment(
            [prev + Vec2::new(0.0, 2.5), pt + Vec2::new(0.0, 2.5)],
            Stroke::new(width + 1.0, shadow),
        );
        painter.line_segment([prev, pt], Stroke::new(width, col));
        prev = pt;
    }
    painter.circle_filled(p1, 4.2, col);
    painter.circle_filled(p2, 4.2, col);
}

fn next_waveform(w: WaveformKind) -> WaveformKind {
    match w {
        WaveformKind::Sine => WaveformKind::Saw,
        WaveformKind::Saw => WaveformKind::Square,
        WaveformKind::Square => WaveformKind::Triangle,
        WaveformKind::Triangle => WaveformKind::PulsePwm,
        WaveformKind::PulsePwm => WaveformKind::SuperSaw,
        WaveformKind::SuperSaw => WaveformKind::WhiteNoise,
        WaveformKind::WhiteNoise => WaveformKind::SubOctave,
        WaveformKind::SubOctave => WaveformKind::AtmoPad,
        WaveformKind::AtmoPad => WaveformKind::Sine,
    }
}

fn next_filter(f: FilterMode) -> FilterMode {
    match f {
        FilterMode::LowPass24dB => FilterMode::BandPass12dB,
        FilterMode::BandPass12dB => FilterMode::HighPass12dB,
        FilterMode::HighPass12dB => FilterMode::AcidResonant,
        FilterMode::AcidResonant => FilterMode::LowPass24dB,
    }
}

fn next_extreme(x: ExtremeDspMode) -> ExtremeDspMode {
    let all = ExtremeDspMode::all();
    let idx = all.iter().position(|&m| m == x).unwrap_or(0);
    all[(idx + 1) % all.len()]
}

fn next_pcm(p: Option<PcmProgram>) -> Option<PcmProgram> {
    let all = PcmProgram::all();
    match p {
        None => all.first().copied(),
        Some(cur) => {
            let idx = all.iter().position(|&x| x == cur).unwrap_or(0);
            all.get(idx + 1).copied()
        }
    }
}

/// Render the multi-row scrollable 3U Eurorack skiff, Module Browser, and draggable cables.
pub fn render_vcv_eurorack_canvas(app: &mut SynthCraftApp, ui: &mut egui::Ui) {
    let pal = app.skin.palette();
    let green = pal.arc_primary;
    let cyan = pal.arc_secondary;
    let amber = pal.led_accent;
    let purple = Color32::from_rgb(168, 85, 247);
    let red = Color32::from_rgb(239, 68, 68);

    // Rack Toolbar
    ui.horizontal_wrapped(|ui| {
        ui.strong("3U Eurorack Modular Skiff");
        ui.label(
            RichText::new(format!(
                "({} modules · {} cables)",
                app.engine.rack_modules.len(),
                app.engine.rack_cables.len()
            ))
            .small()
            .color(pal.text_muted),
        );
        let add_label = if app.show_module_browser {
            "Close Module Browser"
        } else {
            "+ Add Module"
        };
        if ui.button(add_label).clicked() {
            app.show_module_browser = !app.show_module_browser;
        }
        if ui.small_button("Reset Cables").clicked()
            && Command::ResetRackCables.dispatch(&mut app.engine).is_ok()
        {
            app.status_line = "Restored default 3U rack cables".into();
            app.persist_state();
        }
        if ui.small_button("Clear Cables").clicked() {
            let ids: Vec<String> = app
                .engine
                .rack_cables
                .iter()
                .map(|c| c.id.clone())
                .collect();
            for cable_id in ids {
                let _ = Command::DisconnectRackCable { cable_id }.dispatch(&mut app.engine);
            }
            app.status_line = "Cleared all 3U rack cables".into();
            app.persist_state();
        }
        if ui.small_button("Reset Rack").clicked()
            && Command::ResetRackLayout.dispatch(&mut app.engine).is_ok()
        {
            app.status_line = "Restored default 2-row 3U rack layout".into();
            app.persist_state();
        }
        ui.label(
            RichText::new(
                "Right-click module to move/bypass/remove · Drag jacks across rows to patch",
            )
            .small()
            .color(pal.text_muted),
        );
    });

    // Collapsible Categorized Module Browser
    if app.show_module_browser {
        egui::Frame::new()
            .fill(pal.module_bg)
            .stroke(Stroke::new(1.0, pal.module_border))
            .corner_radius(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.strong("Module Browser:");
                    if ui
                        .selectable_label(app.browser_category.is_none(), "All")
                        .clicked()
                    {
                        app.browser_category = None;
                    }
                    for cat in RackCategory::ALL {
                        if ui
                            .selectable_label(app.browser_category == Some(cat), cat.label())
                            .clicked()
                        {
                            app.browser_category = Some(cat);
                        }
                    }
                    ui.separator();
                    ui.label("Insert into:");
                    for r in 0..3 {
                        if ui
                            .selectable_label(app.browser_target_row == r, format!("Row {}", r + 1))
                            .clicked()
                        {
                            app.browser_target_row = r;
                        }
                    }
                });
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    for kind in RackModuleKind::ALL {
                        if let Some(filter) = app.browser_category {
                            if kind.category() != filter {
                                continue;
                            }
                        }
                        let btn = egui::Button::new(
                            RichText::new(format!("+ {} ({} HP)", kind.title(), kind.hp()))
                                .monospace()
                                .size(10.5),
                        );
                        if ui.add(btn).on_hover_text(kind.description()).clicked() {
                            match (Command::AddRackModule {
                                kind: kind.as_str().to_string(),
                                row: Some(app.browser_target_row),
                            })
                            .dispatch(&mut app.engine)
                            {
                                Ok(_) => {
                                    app.status_line = format!(
                                        "Added {} to Row {}",
                                        kind.title(),
                                        app.browser_target_row + 1
                                    );
                                    app.persist_state();
                                }
                                Err(e) => app.status_line = format!("Add module failed: {e}"),
                            }
                        }
                    }
                });
            });
        ui.add_space(4.0);
    }

    // Group placed modules by row (always show at least 2 rows, up to 3)
    let modules_snap = app.engine.rack_modules.clone();
    let max_row = modules_snap
        .iter()
        .map(|m| m.row)
        .max()
        .unwrap_or(0)
        .clamp(1, 2);
    let num_rows = max_row + 1;
    let row_h = 278.0_f32;
    let row_gap = 10.0_f32;
    let hp_px = 12.8_f32;

    let mut max_row_w = ui.available_width().max(760.0);
    for r in 0..num_rows {
        let rw: f32 = modules_snap
            .iter()
            .filter(|m| m.row == r)
            .map(|m| f32::from(m.kind.hp()) * hp_px + 6.0)
            .sum::<f32>()
            + 36.0;
        if rw > max_row_w {
            max_row_w = rw;
        }
    }
    let total_h = (num_rows as f32) * row_h + ((num_rows - 1) as f32) * row_gap;
    let viewport_h = total_h.min(585.0);

    let before = app.engine.active_patch().clone();
    let mut after = before.clone();
    let mut pending_cmds: Vec<Command> = Vec::new();
    let mut pending_status: Option<String> = None;

    let live_scope: Vec<f32> = app
        .audio
        .as_ref()
        .map(|a| a.meters.recent(96))
        .unwrap_or_default();

    egui::ScrollArea::both()
        .id_salt("eurorack_multirow_scroll")
        .max_height(viewport_h)
        .show(ui, |ui| {
            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(max_row_w, total_h), Sense::hover());
            let painter = ui.painter_at(rect);

            let mut jack_positions: HashMap<(String, String), Pos2> = HashMap::new();
            let mut jack_meta: Vec<(String, String, bool, Pos2)> = Vec::new();

            for r in 0..num_rows {
                let row_top = rect.top() + (r as f32) * (row_h + row_gap);
                let row_rect = Rect::from_min_size(
                    Pos2::new(rect.left(), row_top),
                    Vec2::new(max_row_w, row_h),
                );
                painter.rect_filled(row_rect, 8.0, pal.panel_bg);
                let top_rail = Rect::from_min_max(
                    row_rect.min,
                    Pos2::new(row_rect.max.x, row_rect.min.y + 13.0),
                );
                let bot_rail = Rect::from_min_max(
                    Pos2::new(row_rect.min.x, row_rect.max.y - 13.0),
                    row_rect.max,
                );
                painter.rect_filled(top_rail, 3.0, pal.rail_col);
                painter.rect_filled(bot_rail, 3.0, pal.rail_col);
                painter.text(
                    Pos2::new(row_rect.left() + 8.0, row_rect.top() + 6.5),
                    egui::Align2::LEFT_CENTER,
                    format!("ROW {} (3U)", r + 1),
                    egui::FontId::monospace(8.0),
                    pal.text_muted,
                );

                let mut cursor_x = row_rect.left() + 10.0;
                for m in modules_snap.iter().filter(|m| m.row == r) {
                    let mod_w = f32::from(m.kind.hp()) * hp_px;
                    let mod_rect = Rect::from_min_max(
                        Pos2::new(cursor_x + 2.0, row_rect.top() + 15.0),
                        Pos2::new(cursor_x + mod_w - 2.0, row_rect.bottom() - 15.0),
                    );
                    cursor_x += mod_w + 4.0;

                    let bg = if m.bypassed {
                        pal.panel_bg
                    } else {
                        pal.module_bg
                    };
                    painter.rect_filled(mod_rect, 4.0, bg);
                    painter.rect_stroke(
                        mod_rect,
                        4.0,
                        Stroke::new(
                            1.5,
                            if m.bypassed {
                                pal.text_muted
                            } else {
                                pal.module_border
                            },
                        ),
                        egui::StrokeKind::Outside,
                    );

                    for sx in [mod_rect.left() + 7.0, mod_rect.right() - 7.0] {
                        for sy in [mod_rect.top() + 7.0, mod_rect.bottom() - 7.0] {
                            painter.circle_filled(Pos2::new(sx, sy), 2.2, pal.text_muted);
                        }
                    }

                    // Module header (right-click for move/bypass/remove)
                    let hdr_rect = Rect::from_min_max(
                        Pos2::new(mod_rect.left() + 12.0, mod_rect.top() + 4.0),
                        Pos2::new(mod_rect.right() - 12.0, mod_rect.top() + 22.0),
                    );
                    let hdr_resp = ui.interact(
                        hdr_rect,
                        ui.scope_id().with(("mod_hdr", &m.id)),
                        Sense::click(),
                    );
                    painter.text(
                        hdr_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        m.kind.title(),
                        egui::FontId::monospace(10.0),
                        if m.bypassed { pal.text_muted } else { cyan },
                    );
                    hdr_resp.context_menu(|ui| {
                        ui.label(format!("{} ({})", m.kind.title(), m.id));
                        ui.separator();
                        if ui
                            .button(if m.bypassed {
                                "Enable Module"
                            } else {
                                "Bypass Module"
                            })
                            .clicked()
                        {
                            pending_cmds.push(Command::SetRackModuleParam {
                                module_id: m.id.clone(),
                                param_a: None,
                                param_b: None,
                                bypassed: Some(!m.bypassed),
                            });
                            ui.close();
                        }
                        if ui.button("Move Left").clicked() {
                            pending_cmds.push(Command::MoveRackModule {
                                module_id: m.id.clone(),
                                delta_row: 0,
                                delta_slot: -1,
                            });
                            ui.close();
                        }
                        if ui.button("Move Right").clicked() {
                            pending_cmds.push(Command::MoveRackModule {
                                module_id: m.id.clone(),
                                delta_row: 0,
                                delta_slot: 1,
                            });
                            ui.close();
                        }
                        if m.row > 0 && ui.button("Move Row Up").clicked() {
                            pending_cmds.push(Command::MoveRackModule {
                                module_id: m.id.clone(),
                                delta_row: -1,
                                delta_slot: 0,
                            });
                            ui.close();
                        }
                        if m.row < 2 && ui.button("Move Row Down").clicked() {
                            pending_cmds.push(Command::MoveRackModule {
                                module_id: m.id.clone(),
                                delta_row: 1,
                                delta_slot: 0,
                            });
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Remove Module").clicked() {
                            pending_cmds.push(Command::RemoveRackModule {
                                module_id: m.id.clone(),
                            });
                            pending_status = Some(format!("Removed {}", m.kind.title()));
                            ui.close();
                        }
                    });
                    hdr_resp.on_hover_text(format!(
                        "{}\nRight-click to move, bypass, or remove",
                        m.kind.description()
                    ));

                    // Clickable subtitle
                    let sub_rect = Rect::from_center_size(
                        Pos2::new(mod_rect.center().x, mod_rect.top() + 30.0),
                        Vec2::new((mod_rect.width() - 14.0).max(40.0), 15.0),
                    );
                    let sub_resp = ui.interact(
                        sub_rect,
                        ui.scope_id().with(("mod_sub", &m.id)),
                        Sense::click(),
                    );
                    let cycleable = matches!(
                        m.kind,
                        RackModuleKind::Vco1
                            | RackModuleKind::Vco2Xdsp
                            | RackModuleKind::PcmWorkstation
                            | RackModuleKind::VcfLadder
                    );
                    if sub_resp.hovered() && cycleable {
                        painter.rect_filled(sub_rect, 3.0, pal.module_header);
                    }
                    painter.text(
                        sub_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        module_subtitle(m, &after),
                        egui::FontId::monospace(9.0),
                        if m.bypassed { pal.text_muted } else { green },
                    );
                    if sub_resp.clicked() {
                        match m.kind {
                            RackModuleKind::Vco1 => after.vco1_wave = next_waveform(after.vco1_wave),
                            RackModuleKind::Vco2Xdsp => {
                                after.extreme_dsp = next_extreme(after.extreme_dsp)
                            }
                            RackModuleKind::PcmWorkstation => {
                                after.m1_program = next_pcm(after.m1_program)
                            }
                            RackModuleKind::VcfLadder => {
                                after.filter_mode = next_filter(after.filter_mode)
                            }
                            _ => {}
                        }
                    }

                    // Inline 3U Scope or dual interactive knobs
                    if m.kind == RackModuleKind::InlineScope {
                        let scope_rect = Rect::from_min_max(
                            Pos2::new(mod_rect.left() + 10.0, mod_rect.top() + 46.0),
                            Pos2::new(mod_rect.right() - 10.0, mod_rect.bottom() - 72.0),
                        );
                        painter.rect_filled(scope_rect, 4.0, pal.lcd_bg);
                        painter.rect_stroke(
                            scope_rect,
                            4.0,
                            Stroke::new(1.0, pal.lcd_border),
                            egui::StrokeKind::Inside,
                        );
                        let mid = scope_rect.center().y;
                        painter.line_segment(
                            [
                                Pos2::new(scope_rect.left() + 2.0, mid),
                                Pos2::new(scope_rect.right() - 2.0, mid),
                            ],
                            Stroke::new(0.8, pal.arc_track),
                        );
                        if live_scope.len() >= 2 {
                            let amp = scope_rect.height() * 0.42;
                            let pts: Vec<Pos2> = live_scope
                                .iter()
                                .enumerate()
                                .map(|(i, &s)| {
                                    let x = scope_rect.left()
                                        + (i as f32 / (live_scope.len() - 1) as f32)
                                            * scope_rect.width();
                                    Pos2::new(x, mid - s.clamp(-1.0, 1.0) * amp)
                                })
                                .collect();
                            painter.add(egui::Shape::line(pts, Stroke::new(1.6, pal.lcd_text)));
                        } else {
                            painter.text(
                                scope_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "IDLE",
                                egui::FontId::monospace(10.0),
                                pal.text_muted,
                            );
                        }
                    } else if let Some(knobs) = module_knobs(m.kind) {
                        for (kidx, (k_label, binding)) in knobs.iter().enumerate() {
                            let ky = mod_rect.top() + 68.0 + (kidx as f32) * 56.0;
                            let center = Pos2::new(mod_rect.center().x, ky);
                            let hit_rect = Rect::from_center_size(center, Vec2::splat(36.0));
                            let resp = ui.interact(
                                hit_rect,
                                ui.scope_id().with(("mod_knob", &m.id, kidx)),
                                Sense::click_and_drag(),
                            );

                            let mut norm_val = match binding {
                                KnobBinding::Patch(kid) => kid.get_norm(&after),
                                KnobBinding::ModuleA => m.param_a,
                                KnobBinding::ModuleB => m.param_b,
                            };
                            let mut custom_changed = false;

                            if resp.dragged() {
                                let fine = ui.input(|i| i.modifiers.shift);
                                let speed = if fine { 0.0015 } else { 0.0055 };
                                norm_val = (norm_val - resp.drag_delta().y * speed).clamp(0.0, 1.0);
                                match binding {
                                    KnobBinding::Patch(kid) => kid.set_norm(&mut after, norm_val),
                                    KnobBinding::ModuleA | KnobBinding::ModuleB => {
                                        custom_changed = true
                                    }
                                }
                            }
                            if resp.hovered() {
                                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                                if scroll.abs() > 0.1 {
                                    norm_val = (norm_val + scroll * 0.0015).clamp(0.0, 1.0);
                                    match binding {
                                        KnobBinding::Patch(kid) => {
                                            kid.set_norm(&mut after, norm_val)
                                        }
                                        KnobBinding::ModuleA | KnobBinding::ModuleB => {
                                            custom_changed = true
                                        }
                                    }
                                }
                            }
                            if resp.double_clicked() {
                                match binding {
                                    KnobBinding::Patch(kid) => {
                                        kid.reset(&mut after);
                                        norm_val = kid.get_norm(&after);
                                    }
                                    KnobBinding::ModuleA => {
                                        norm_val = m.kind.default_params().0;
                                        custom_changed = true;
                                    }
                                    KnobBinding::ModuleB => {
                                        norm_val = m.kind.default_params().1;
                                        custom_changed = true;
                                    }
                                }
                            }
                            if custom_changed {
                                pending_cmds.push(Command::SetRackModuleParam {
                                    module_id: m.id.clone(),
                                    param_a: matches!(binding, KnobBinding::ModuleA)
                                        .then_some(norm_val),
                                    param_b: matches!(binding, KnobBinding::ModuleB)
                                        .then_some(norm_val),
                                    bypassed: None,
                                });
                            }

                            let active = resp.hovered() || resp.dragged();
                            painter.circle_filled(center, 15.5, pal.knob_outer);
                            painter.circle_stroke(
                                center,
                                15.5,
                                Stroke::new(
                                    if active { 2.4 } else { 1.8 },
                                    if active { cyan } else { pal.arc_track },
                                ),
                            );
                            painter.circle_filled(center, 11.5, pal.knob_cap);

                            let start_ang = PI * 0.75;
                            let sweep = PI * 1.5 * norm_val.clamp(0.0, 1.0);
                            let arc_pts: Vec<Pos2> = (0..=16)
                                .map(|s| {
                                    let a = start_ang + (s as f32 / 16.0) * sweep;
                                    Pos2::new(center.x + a.cos() * 16.0, center.y + a.sin() * 16.0)
                                })
                                .collect();
                            painter.add(egui::Shape::line(
                                arc_pts,
                                Stroke::new(2.4, if active { amber } else { pal.arc_primary }),
                            ));
                            let ang = start_ang + sweep;
                            let tip = Pos2::new(
                                center.x + ang.cos() * 11.5,
                                center.y + ang.sin() * 11.5,
                            );
                            painter.line_segment([center, tip], Stroke::new(2.0, pal.knob_pointer));
                            painter.text(
                                Pos2::new(center.x, ky + 22.0),
                                egui::Align2::CENTER_CENTER,
                                *k_label,
                                egui::FontId::monospace(8.5),
                                pal.text_primary,
                            );
                            let val_str = match binding {
                                KnobBinding::Patch(kid) => kid.format_value(&after),
                                KnobBinding::ModuleA => format_custom_knob(m.kind, true, norm_val),
                                KnobBinding::ModuleB => format_custom_knob(m.kind, false, norm_val),
                            };
                            resp.on_hover_text(format!(
                                "{} {k_label}: {val_str}\nDrag vertically (Shift = fine) · Double-click to reset",
                                m.kind.title()
                            ));
                        }

                        if m.kind == RackModuleKind::EuclideanClock {
                            let steps = (2.0 + m.param_a * 14.0).round() as usize;
                            let pulses = ((1.0 + m.param_b * 15.0).round() as usize).min(steps);
                            let pat = euclidean_pattern(steps, pulses);
                            let y = mod_rect.bottom() - 67.0;
                            let span = (mod_rect.width() - 20.0).max(40.0);
                            for (si, &on) in pat.iter().take(steps).enumerate() {
                                let x = mod_rect.left()
                                    + 10.0
                                    + (si as f32 + 0.5) / (steps as f32) * span;
                                painter.circle_filled(
                                    Pos2::new(x, y),
                                    2.5,
                                    if on { amber } else { pal.arc_track },
                                );
                            }
                        }
                    }

                    // 3.5 mm Jack Sockets
                    let jacks = m.kind.jacks();
                    let j_count = jacks.len().max(1);
                    for (jidx, (j_name, is_out)) in jacks.iter().enumerate() {
                        let col_idx = jidx % 2;
                        let row_idx = jidx / 2;
                        let jx = if j_count == 1 {
                            mod_rect.center().x
                        } else if col_idx == 0 {
                            mod_rect.left() + mod_rect.width() * 0.28
                        } else {
                            mod_rect.left() + mod_rect.width() * 0.72
                        };
                        let jy = mod_rect.bottom() - 46.0 + (row_idx as f32) * 23.0;
                        let jpos = Pos2::new(jx, jy);
                        let jack_rect = Rect::from_center_size(jpos, Vec2::splat(20.0));
                        let jresp = ui.interact(
                            jack_rect,
                            ui.scope_id().with(("mod_jack", &m.id, jidx)),
                            Sense::click_and_drag(),
                        );

                        let valid_target = app.cable_drag.as_ref().is_some_and(|d| {
                            d.anchor_is_out != *is_out
                                && !(canonical_rack_module(&d.anchor_module) == m.id.as_str()
                                    && d.anchor_jack == *j_name)
                        });

                        if jresp.drag_started() {
                            let existing = app.engine.rack_cables.iter().rev().find(|c| {
                                (canonical_rack_module(&c.from_module) == m.id.as_str()
                                    && c.from_jack == *j_name)
                                    || (canonical_rack_module(&c.to_module) == m.id.as_str()
                                        && c.to_jack == *j_name)
                            });
                            if let Some(cab) = existing.cloned() {
                                let from_matches = canonical_rack_module(&cab.from_module)
                                    == m.id.as_str()
                                    && cab.from_jack == *j_name;
                                pending_cmds.push(Command::DisconnectRackCable {
                                    cable_id: cab.id.clone(),
                                });
                                if from_matches {
                                    app.cable_drag = Some(CableDrag {
                                        anchor_module: canonical_rack_module(&cab.to_module)
                                            .to_string(),
                                        anchor_jack: cab.to_jack,
                                        anchor_is_out: false,
                                        color_name: cab.color_name,
                                        gain: cab.gain,
                                    });
                                } else {
                                    app.cable_drag = Some(CableDrag {
                                        anchor_module: canonical_rack_module(&cab.from_module)
                                            .to_string(),
                                        anchor_jack: cab.from_jack,
                                        anchor_is_out: true,
                                        color_name: cab.color_name,
                                        gain: cab.gain,
                                    });
                                }
                                pending_status =
                                    Some(format!("Moving cable from {} / {j_name}", m.id));
                            } else {
                                let palette = ["green", "cyan", "amber", "purple", "red"];
                                let color =
                                    palette[app.engine.rack_cables.len() % palette.len()].to_string();
                                app.cable_drag = Some(CableDrag {
                                    anchor_module: m.id.clone(),
                                    anchor_jack: (*j_name).to_string(),
                                    anchor_is_out: *is_out,
                                    color_name: color,
                                    gain: 1.0,
                                });
                            }
                        }

                        let connected: Vec<_> = app
                            .engine
                            .rack_cables
                            .iter()
                            .filter(|c| {
                                (canonical_rack_module(&c.from_module) == m.id.as_str()
                                    && c.from_jack == *j_name)
                                    || (canonical_rack_module(&c.to_module) == m.id.as_str()
                                        && c.to_jack == *j_name)
                            })
                            .cloned()
                            .collect();
                        jresp.context_menu(|ui| {
                            ui.label(format!("{} · {j_name}", m.id));
                            ui.separator();
                            if connected.is_empty() {
                                ui.label("No cables connected");
                            } else {
                                for cab in &connected {
                                    let label = format!(
                                        "Unplug {} / {} -> {} / {}",
                                        cab.from_module, cab.from_jack, cab.to_module, cab.to_jack
                                    );
                                    if ui.button(label).clicked() {
                                        pending_cmds.push(Command::DisconnectRackCable {
                                            cable_id: cab.id.clone(),
                                        });
                                        pending_status = Some(format!("Unplugged {}", cab.id));
                                        ui.close();
                                    }
                                }
                            }
                            ui.separator();
                            if ui.button("Reset default cables").clicked() {
                                pending_cmds.push(Command::ResetRackCables);
                                ui.close();
                            }
                        });

                        let ring_col = if valid_target {
                            amber
                        } else if *is_out {
                            green
                        } else {
                            cyan
                        };
                        painter.circle_filled(jpos, 6.5, pal.knob_outer);
                        painter.circle_stroke(
                            jpos,
                            6.5,
                            Stroke::new(
                                if jresp.hovered() || valid_target {
                                    2.8
                                } else {
                                    1.9
                                },
                                ring_col,
                            ),
                        );
                        painter.circle_filled(jpos, 2.4, Color32::BLACK);
                        painter.text(
                            Pos2::new(jx, jy + 10.5),
                            egui::Align2::CENTER_CENTER,
                            *j_name,
                            egui::FontId::monospace(7.2),
                            pal.text_primary,
                        );
                        let dir = if *is_out { "OUT" } else { "IN" };
                        jresp.on_hover_text(format!(
                            "{} · {j_name} ({dir})\nDrag to patch/move · Right-click to unplug",
                            m.id
                        ));

                        jack_positions.insert((m.id.clone(), (*j_name).to_string()), jpos);
                        if m.id == "LFO / ADSR" {
                            jack_positions
                                .insert(("LFO-MOD".to_string(), (*j_name).to_string()), jpos);
                            jack_positions
                                .insert(("ADSR-ENV".to_string(), (*j_name).to_string()), jpos);
                        }
                        jack_meta.push((m.id.clone(), (*j_name).to_string(), *is_out, jpos));
                    }
                }
            }

            for cab in &app.engine.rack_cables {
                if let (Some(&p1), Some(&p2)) = (
                    jack_positions.get(&(cab.from_module.clone(), cab.from_jack.clone())),
                    jack_positions.get(&(cab.to_module.clone(), cab.to_jack.clone())),
                ) {
                    let col = cable_color(&cab.color_name, green, amber, purple, red, cyan);
                    draw_catenary(&painter, p1, p2, col, 3.0);
                }
            }

            if let Some(drag) = app.cable_drag.clone() {
                let pointer_pos = ui.input(|i| i.pointer.interact_pos());
                let snapped_target = pointer_pos.and_then(|ptr| {
                    jack_meta
                        .iter()
                        .filter(|(m, j, is_out, _)| {
                            *is_out != drag.anchor_is_out
                                && !(canonical_rack_module(&drag.anchor_module) == m.as_str()
                                    && drag.anchor_jack == *j)
                        })
                        .min_by(|a, b| {
                            a.3.distance(ptr)
                                .partial_cmp(&b.3.distance(ptr))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .filter(|(_, _, _, pos)| pos.distance(ptr) <= 18.0)
                        .cloned()
                });

                if let Some(&anchor_pos) =
                    jack_positions.get(&(drag.anchor_module.clone(), drag.anchor_jack.clone()))
                {
                    let end_pos = snapped_target
                        .as_ref()
                        .map(|t| t.3)
                        .or(pointer_pos)
                        .unwrap_or(anchor_pos);
                    let col = cable_color(&drag.color_name, green, amber, purple, red, cyan);
                    draw_catenary(&painter, anchor_pos, end_pos, col, 3.4);
                }

                if !ui.input(|i| i.pointer.primary_down()) {
                    app.cable_drag = None;
                    if let Some((tm, tj, t_is_out, _)) = snapped_target {
                        let (from_module, from_jack, to_module, to_jack) =
                            if drag.anchor_is_out && !t_is_out {
                                (drag.anchor_module, drag.anchor_jack, tm, tj)
                            } else {
                                (tm, tj, drag.anchor_module, drag.anchor_jack)
                            };
                        pending_cmds.push(Command::ConnectRackCable {
                            from_module: from_module.clone(),
                            from_jack: from_jack.clone(),
                            to_module: to_module.clone(),
                            to_jack: to_jack.clone(),
                            color_name: Some(drag.color_name),
                            gain: Some(drag.gain),
                        });
                        pending_status = Some(format!(
                            "Patched {from_module} / {from_jack} -> {to_module} / {to_jack}"
                        ));
                    }
                }
            }
        });

    app.apply_patch_edit(&before, &after);
    if !pending_cmds.is_empty() {
        for cmd in pending_cmds {
            let _ = cmd.dispatch(&mut app.engine);
        }
        if let Some(msg) = pending_status {
            app.status_line = msg;
        }
        app.persist_state();
    }
}
