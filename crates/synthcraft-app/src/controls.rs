//! Custom GPU-rendered hardware synthesizer controls (`egui::Painter`),
//! switchable hardware skins (`SynthSkin`), and switchable studio layouts (`SynthLayout`).

use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SynthSkin {
    ObsidianVcv,
    WorkstationVintage,
    SilverAcid,
    WestCoastGold,
    DaylightStudio,
}

impl SynthSkin {
    pub fn all() -> &'static [Self] {
        &[
            Self::ObsidianVcv,
            Self::WorkstationVintage,
            Self::SilverAcid,
            Self::WestCoastGold,
            Self::DaylightStudio,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ObsidianVcv => "Obsidian VCV",
            Self::WorkstationVintage => "1990-style PCM",
            Self::SilverAcid => "Silver Acid",
            Self::WestCoastGold => "West-Coast Gold",
            Self::DaylightStudio => "Daylight AAA",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::ObsidianVcv => {
                "Anodized dark slate Eurorack faceplates with emerald & cyan LED rings"
            }
            Self::WorkstationVintage => {
                "1990 charcoal workstation chassis with iconic warm amber backlit LCD & red LEDs"
            }
            Self::SilverAcid => {
                "Brushed metallic silver acid-box faceplates with crimson & acid accents"
            }
            Self::WestCoastGold => {
                "Deep navy-bronze West-Coast synthesis panels with gold & violet harmonic arcs"
            }
            Self::DaylightStudio => {
                "High-contrast ivory studio console with WCAG AAA ink and cobalt LED arcs"
            }
        }
    }

    pub fn palette(self) -> SkinPalette {
        match self {
            Self::ObsidianVcv => SkinPalette {
                is_dark: true,
                panel_bg: egui::Color32::from_rgb(9, 14, 26),
                module_bg: egui::Color32::from_rgb(19, 27, 46),
                module_header: egui::Color32::from_rgb(28, 39, 64),
                module_border: egui::Color32::from_rgb(56, 78, 118),
                rail_col: egui::Color32::from_rgb(36, 46, 66),
                lcd_bg: egui::Color32::from_rgb(6, 20, 28),
                lcd_border: egui::Color32::from_rgb(34, 197, 94),
                lcd_text: egui::Color32::from_rgb(74, 222, 128),
                lcd_subtext: egui::Color32::from_rgb(56, 189, 248),
                knob_outer: egui::Color32::from_rgb(28, 36, 52),
                knob_cap: egui::Color32::from_rgb(48, 60, 82),
                knob_pointer: egui::Color32::from_rgb(248, 250, 252),
                arc_track: egui::Color32::from_rgb(30, 41, 59),
                arc_primary: egui::Color32::from_rgb(34, 197, 94),
                arc_secondary: egui::Color32::from_rgb(56, 189, 248),
                fader_track: egui::Color32::from_rgb(12, 18, 30),
                fader_cap: egui::Color32::from_rgb(71, 85, 105),
                led_on: egui::Color32::from_rgb(34, 197, 94),
                led_accent: egui::Color32::from_rgb(249, 115, 22),
                led_off: egui::Color32::from_rgb(30, 41, 59),
                text_primary: egui::Color32::from_rgb(241, 245, 249),
                text_muted: egui::Color32::from_rgb(148, 163, 184),
            },
            Self::WorkstationVintage => SkinPalette {
                is_dark: true,
                panel_bg: egui::Color32::from_rgb(20, 21, 24),
                module_bg: egui::Color32::from_rgb(31, 33, 38),
                module_header: egui::Color32::from_rgb(42, 45, 52),
                module_border: egui::Color32::from_rgb(82, 86, 96),
                rail_col: egui::Color32::from_rgb(50, 52, 58),
                lcd_bg: egui::Color32::from_rgb(38, 22, 4),
                lcd_border: egui::Color32::from_rgb(217, 119, 6),
                lcd_text: egui::Color32::from_rgb(251, 191, 36),
                lcd_subtext: egui::Color32::from_rgb(245, 158, 11),
                knob_outer: egui::Color32::from_rgb(24, 26, 30),
                knob_cap: egui::Color32::from_rgb(45, 48, 54),
                knob_pointer: egui::Color32::from_rgb(251, 191, 36),
                arc_track: egui::Color32::from_rgb(48, 44, 38),
                arc_primary: egui::Color32::from_rgb(245, 158, 11),
                arc_secondary: egui::Color32::from_rgb(239, 68, 68),
                fader_track: egui::Color32::from_rgb(15, 16, 18),
                fader_cap: egui::Color32::from_rgb(64, 68, 76),
                led_on: egui::Color32::from_rgb(239, 68, 68),
                led_accent: egui::Color32::from_rgb(251, 191, 36),
                led_off: egui::Color32::from_rgb(48, 36, 36),
                text_primary: egui::Color32::from_rgb(245, 245, 244),
                text_muted: egui::Color32::from_rgb(168, 162, 158),
            },
            Self::SilverAcid => SkinPalette {
                is_dark: false,
                panel_bg: egui::Color32::from_rgb(203, 213, 225),
                module_bg: egui::Color32::from_rgb(226, 232, 240),
                module_header: egui::Color32::from_rgb(186, 198, 214),
                module_border: egui::Color32::from_rgb(100, 116, 139),
                rail_col: egui::Color32::from_rgb(148, 163, 184),
                lcd_bg: egui::Color32::from_rgb(24, 28, 20),
                lcd_border: egui::Color32::from_rgb(220, 38, 38),
                lcd_text: egui::Color32::from_rgb(248, 113, 113),
                lcd_subtext: egui::Color32::from_rgb(250, 204, 21),
                knob_outer: egui::Color32::from_rgb(30, 41, 59),
                knob_cap: egui::Color32::from_rgb(51, 65, 85),
                knob_pointer: egui::Color32::from_rgb(239, 68, 68),
                arc_track: egui::Color32::from_rgb(180, 190, 204),
                arc_primary: egui::Color32::from_rgb(220, 38, 38),
                arc_secondary: egui::Color32::from_rgb(234, 88, 12),
                fader_track: egui::Color32::from_rgb(71, 85, 105),
                fader_cap: egui::Color32::from_rgb(30, 41, 59),
                led_on: egui::Color32::from_rgb(220, 38, 38),
                led_accent: egui::Color32::from_rgb(234, 88, 12),
                led_off: egui::Color32::from_rgb(160, 174, 192),
                text_primary: egui::Color32::from_rgb(15, 23, 42),
                text_muted: egui::Color32::from_rgb(51, 65, 85),
            },
            Self::WestCoastGold => SkinPalette {
                is_dark: true,
                panel_bg: egui::Color32::from_rgb(15, 14, 26),
                module_bg: egui::Color32::from_rgb(26, 23, 44),
                module_header: egui::Color32::from_rgb(39, 34, 66),
                module_border: egui::Color32::from_rgb(161, 98, 7),
                rail_col: egui::Color32::from_rgb(66, 52, 32),
                lcd_bg: egui::Color32::from_rgb(18, 12, 32),
                lcd_border: egui::Color32::from_rgb(234, 179, 8),
                lcd_text: egui::Color32::from_rgb(250, 204, 21),
                lcd_subtext: egui::Color32::from_rgb(192, 132, 252),
                knob_outer: egui::Color32::from_rgb(38, 30, 56),
                knob_cap: egui::Color32::from_rgb(76, 58, 104),
                knob_pointer: egui::Color32::from_rgb(250, 204, 21),
                arc_track: egui::Color32::from_rgb(45, 38, 68),
                arc_primary: egui::Color32::from_rgb(234, 179, 8),
                arc_secondary: egui::Color32::from_rgb(168, 85, 247),
                fader_track: egui::Color32::from_rgb(18, 15, 30),
                fader_cap: egui::Color32::from_rgb(180, 130, 40),
                led_on: egui::Color32::from_rgb(250, 204, 21),
                led_accent: egui::Color32::from_rgb(192, 132, 252),
                led_off: egui::Color32::from_rgb(44, 38, 64),
                text_primary: egui::Color32::from_rgb(254, 249, 195),
                text_muted: egui::Color32::from_rgb(196, 181, 253),
            },
            Self::DaylightStudio => SkinPalette {
                is_dark: false,
                panel_bg: egui::Color32::from_rgb(241, 245, 249),
                module_bg: egui::Color32::from_rgb(255, 255, 255),
                module_header: egui::Color32::from_rgb(226, 232, 240),
                module_border: egui::Color32::from_rgb(148, 163, 184),
                rail_col: egui::Color32::from_rgb(203, 213, 225),
                lcd_bg: egui::Color32::from_rgb(15, 23, 42),
                lcd_border: egui::Color32::from_rgb(37, 99, 235),
                lcd_text: egui::Color32::from_rgb(96, 165, 250),
                lcd_subtext: egui::Color32::from_rgb(52, 211, 153),
                knob_outer: egui::Color32::from_rgb(100, 116, 139),
                knob_cap: egui::Color32::from_rgb(226, 232, 240),
                knob_pointer: egui::Color32::from_rgb(15, 23, 42),
                arc_track: egui::Color32::from_rgb(203, 213, 225),
                arc_primary: egui::Color32::from_rgb(37, 99, 235),
                arc_secondary: egui::Color32::from_rgb(13, 148, 136),
                fader_track: egui::Color32::from_rgb(203, 213, 225),
                fader_cap: egui::Color32::from_rgb(51, 65, 85),
                led_on: egui::Color32::from_rgb(37, 99, 235),
                led_accent: egui::Color32::from_rgb(217, 119, 6),
                led_off: egui::Color32::from_rgb(203, 213, 225),
                text_primary: egui::Color32::from_rgb(15, 23, 42),
                text_muted: egui::Color32::from_rgb(71, 85, 105),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SkinPalette {
    pub is_dark: bool,
    pub panel_bg: egui::Color32,
    pub module_bg: egui::Color32,
    pub module_header: egui::Color32,
    pub module_border: egui::Color32,
    pub rail_col: egui::Color32,
    pub lcd_bg: egui::Color32,
    pub lcd_border: egui::Color32,
    pub lcd_text: egui::Color32,
    pub lcd_subtext: egui::Color32,
    pub knob_outer: egui::Color32,
    pub knob_cap: egui::Color32,
    pub knob_pointer: egui::Color32,
    pub arc_track: egui::Color32,
    pub arc_primary: egui::Color32,
    pub arc_secondary: egui::Color32,
    pub fader_track: egui::Color32,
    pub fader_cap: egui::Color32,
    pub led_on: egui::Color32,
    pub led_accent: egui::Color32,
    pub led_off: egui::Color32,
    pub text_primary: egui::Color32,
    pub text_muted: egui::Color32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SynthLayout {
    WorkbenchSplit,
    EurorackBay,
    Workstation1990,
    LiveCockpit,
}

impl SynthLayout {
    pub fn all() -> &'static [Self] {
        &[
            Self::WorkbenchSplit,
            Self::EurorackBay,
            Self::Workstation1990,
            Self::LiveCockpit,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::WorkbenchSplit => "Workbench Studio",
            Self::EurorackBay => "VCV 3U Eurorack Bay",
            Self::Workstation1990 => "PCM Workstation",
            Self::LiveCockpit => "Live Performance Cockpit",
        }
    }
}

/// Renders an interactive 270-degree hardware rotary potentiometer knob with glowing LED arc,
/// outer tick ring, metallic cap, indicator line, and digital value readout.
/// Drag vertically or horizontally to adjust; double-click to reset to midpoint.
pub fn rotary_knob(
    ui: &mut egui::Ui,
    skin: &SkinPalette,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    unit: &str,
) -> bool {
    let desired_size = egui::vec2(86.0, 98.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());
    let mut changed = false;

    let span = (max - min).max(1e-6);
    if response.dragged() {
        let delta = response.drag_delta();
        let fine = ui.input(|i| i.modifiers.shift);
        let speed = if fine { 0.0015 } else { 0.006 };
        let step = (-delta.y + delta.x * 0.5) * speed * span;
        if step.abs() > 0.0 {
            *value = (*value + step).clamp(min, max);
            changed = true;
        }
    }
    if response.double_clicked() {
        *value = min + span * 0.5;
        changed = true;
    }

    let norm = ((*value - min) / span).clamp(0.0, 1.0);
    let painter = ui.painter_at(rect);

    // Subtle card plate
    let bg_fill = if response.hovered() || response.dragged() {
        skin.module_header
    } else {
        skin.module_bg
    };
    painter.rect_filled(rect, 6.0, bg_fill);
    painter.rect_stroke(
        rect,
        6.0,
        egui::Stroke::new(
            if response.dragged() { 1.5 } else { 1.0 },
            if response.dragged() {
                skin.arc_primary
            } else {
                skin.module_border
            },
        ),
        egui::epaint::StrokeKind::Inside,
    );

    // Top label
    painter.text(
        egui::pos2(rect.center().x, rect.min.y + 11.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(11.0),
        skin.text_primary,
    );

    let center = egui::pos2(rect.center().x, rect.min.y + 49.0);
    let arc_radius = 24.0;
    let knob_radius = 18.0;

    // Angles: 135 deg (bottom-left) to 405 deg (bottom-right) = 270 deg sweep
    let start_angle = std::f32::consts::PI * 0.75;
    let sweep_angle = std::f32::consts::PI * 1.5;

    // Outer scale ticks (11 ticks)
    for t in 0..=10 {
        let frac = t as f32 / 10.0;
        let ang = start_angle + frac * sweep_angle;
        let p_in = egui::pos2(
            center.x + ang.cos() * (arc_radius + 2.0),
            center.y + ang.sin() * (arc_radius + 2.0),
        );
        let p_out = egui::pos2(
            center.x + ang.cos() * (arc_radius + 5.0),
            center.y + ang.sin() * (arc_radius + 5.0),
        );
        let tick_col = if frac <= norm + 0.01 {
            skin.arc_primary
        } else {
            skin.arc_track
        };
        painter.line_segment([p_in, p_out], egui::Stroke::new(1.3, tick_col));
    }

    // Background 270-degree arc
    let segments = 28usize;
    let track_pts: Vec<egui::Pos2> = (0..=segments)
        .map(|i| {
            let a = start_angle + (i as f32 / segments as f32) * sweep_angle;
            egui::pos2(
                center.x + a.cos() * arc_radius,
                center.y + a.sin() * arc_radius,
            )
        })
        .collect();
    painter.add(egui::Shape::line(
        track_pts,
        egui::Stroke::new(3.5, skin.arc_track),
    ));

    // Active glowing LED value arc
    let active_segs = ((norm * segments as f32).ceil() as usize).max(1);
    if norm > 0.005 {
        let active_pts: Vec<egui::Pos2> = (0..=active_segs)
            .map(|i| {
                let frac = (i as f32 / active_segs as f32) * norm;
                let a = start_angle + frac * sweep_angle;
                egui::pos2(
                    center.x + a.cos() * arc_radius,
                    center.y + a.sin() * arc_radius,
                )
            })
            .collect();
        painter.add(egui::Shape::line(
            active_pts,
            egui::Stroke::new(3.8, skin.arc_primary),
        ));
    }

    // Knob body + metallic inner cap
    painter.circle_filled(center, knob_radius, skin.knob_outer);
    painter.circle_stroke(
        center,
        knob_radius,
        egui::Stroke::new(1.5, skin.module_border),
    );
    painter.circle_filled(center, knob_radius - 4.0, skin.knob_cap);

    // Indicator pointer line
    let val_angle = start_angle + norm * sweep_angle;
    let ptr_start = egui::pos2(
        center.x + val_angle.cos() * 4.0,
        center.y + val_angle.sin() * 4.0,
    );
    let ptr_end = egui::pos2(
        center.x + val_angle.cos() * (knob_radius - 2.0),
        center.y + val_angle.sin() * (knob_radius - 2.0),
    );
    painter.line_segment(
        [ptr_start, ptr_end],
        egui::Stroke::new(2.4, skin.knob_pointer),
    );
    painter.circle_filled(center, 2.5, skin.arc_secondary);

    // Digital readout at bottom
    let formatted = format_control_value(*value, max, unit);
    painter.text(
        egui::pos2(rect.center().x, rect.max.y - 11.0),
        egui::Align2::CENTER_CENTER,
        formatted,
        egui::FontId::monospace(10.5),
        skin.lcd_text,
    );

    changed
}

/// Renders a rotary knob with a Vital/Serum-style live animated outer modulation deflection arc
/// (`mod_offset` in `-1.0..=1.0`, representing real-time MSEG / LFO deflection around the base value).
#[allow(clippy::too_many_arguments)]
pub fn rotary_knob_mod(
    ui: &mut egui::Ui,
    skin: &SkinPalette,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    unit: &str,
    mod_offset: f32,
) -> bool {
    let before_cursor = ui.cursor().min;
    let changed = rotary_knob(ui, skin, label, value, min, max, unit);
    if mod_offset.abs() > 0.005 {
        let span = (max - min).max(1e-6);
        let base_norm = ((*value - min) / span).clamp(0.0, 1.0);
        let mod_norm = (base_norm + mod_offset).clamp(0.0, 1.0);
        let center = egui::pos2(before_cursor.x + 43.0, before_cursor.y + 49.0);
        let mod_radius = 28.0;
        let start_angle = std::f32::consts::PI * 0.75;
        let sweep_angle = std::f32::consts::PI * 1.5;
        let lo = base_norm.min(mod_norm);
        let hi = base_norm.max(mod_norm);
        let segs = 16usize;
        let pts: Vec<egui::Pos2> = (0..=segs)
            .map(|i| {
                let f = lo + (hi - lo) * (i as f32 / segs as f32);
                let a = start_angle + f * sweep_angle;
                egui::pos2(
                    center.x + a.cos() * mod_radius,
                    center.y + a.sin() * mod_radius,
                )
            })
            .collect();
        ui.painter().add(egui::Shape::line(
            pts,
            egui::Stroke::new(2.8, skin.arc_secondary),
        ));
    }
    changed
}

/// Renders an interactive studio console vertical hardware fader with recessed LED track,
/// dual dB/percentage scale ticks, 3D ridged fader cap, and value readout.
pub fn vertical_fader(
    ui: &mut egui::Ui,
    skin: &SkinPalette,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    unit: &str,
) -> bool {
    let desired_size = egui::vec2(64.0, 142.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());
    let mut changed = false;

    let track_top = rect.min.y + 24.0;
    let track_bottom = rect.max.y - 24.0;
    let track_h = (track_bottom - track_top).max(10.0);
    let span = (max - min).max(1e-6);

    if response.dragged() || response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let norm = (1.0 - ((pos.y - track_top) / track_h)).clamp(0.0, 1.0);
            *value = min + norm * span;
            changed = true;
        }
    }
    if response.double_clicked() {
        *value = min + span * 0.75;
        changed = true;
    }

    let norm = ((*value - min) / span).clamp(0.0, 1.0);
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 6.0, skin.module_bg);
    painter.rect_stroke(
        rect,
        6.0,
        egui::Stroke::new(1.0, skin.module_border),
        egui::epaint::StrokeKind::Inside,
    );

    // Top label
    painter.text(
        egui::pos2(rect.center().x, rect.min.y + 11.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(10.5),
        skin.text_primary,
    );

    // Scale tick marks left & right of fader slot
    let cx = rect.center().x;
    for t in 0..=8 {
        let y = track_top + (t as f32 / 8.0) * track_h;
        let w = if t % 2 == 0 { 14.0 } else { 9.0 };
        painter.line_segment(
            [egui::pos2(cx - w, y), egui::pos2(cx - 5.0, y)],
            egui::Stroke::new(1.0, skin.text_muted),
        );
        painter.line_segment(
            [egui::pos2(cx + 5.0, y), egui::pos2(cx + w, y)],
            egui::Stroke::new(1.0, skin.text_muted),
        );
    }

    // Recessed fader slot
    let slot_rect = egui::Rect::from_min_max(
        egui::pos2(cx - 3.5, track_top),
        egui::pos2(cx + 3.5, track_bottom),
    );
    painter.rect_filled(slot_rect, 2.0, skin.fader_track);

    // Glowing LED fill from bottom up to thumb
    let thumb_y = track_bottom - norm * track_h;
    let fill_rect = egui::Rect::from_min_max(
        egui::pos2(cx - 2.5, thumb_y),
        egui::pos2(cx + 2.5, track_bottom),
    );
    painter.rect_filled(fill_rect, 2.0, skin.arc_primary);

    // 3D Hardware Fader Cap
    let cap_rect = egui::Rect::from_center_size(egui::pos2(cx, thumb_y), egui::vec2(28.0, 13.0));
    painter.rect_filled(cap_rect, 3.0, skin.fader_cap);
    painter.rect_stroke(
        cap_rect,
        3.0,
        egui::Stroke::new(1.2, skin.knob_pointer),
        egui::epaint::StrokeKind::Inside,
    );
    // Center grip line on fader cap
    painter.line_segment(
        [
            egui::pos2(cap_rect.min.x + 3.0, thumb_y),
            egui::pos2(cap_rect.max.x - 3.0, thumb_y),
        ],
        egui::Stroke::new(2.0, skin.arc_secondary),
    );

    // Bottom readout
    let formatted = format_control_value(*value, max, unit);
    painter.text(
        egui::pos2(cx, rect.max.y - 10.0),
        egui::Align2::CENTER_CENTER,
        formatted,
        egui::FontId::monospace(10.0),
        skin.lcd_text,
    );

    changed
}

/// Renders a backlit PCM workstation / hardware workstation LCD status screen with a 16-segment
/// stereo LED VU Peak Meter.
pub fn lcd_display_banner(
    ui: &mut egui::Ui,
    skin: &SkinPalette,
    mode_tag: &str,
    headline: &str,
    subline: &str,
    rms_dbfs: f32,
    peak_dbfs: f32,
) {
    let desired_h = 64.0;
    let avail_w = ui.available_width().max(320.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(avail_w, desired_h), egui::Sense::hover());
    let painter = ui.painter_at(rect);

    // Recessed LCD bezel
    painter.rect_filled(rect, 6.0, skin.lcd_bg);
    painter.rect_stroke(
        rect,
        6.0,
        egui::Stroke::new(1.6, skin.lcd_border),
        egui::epaint::StrokeKind::Inside,
    );

    // Subtle LCD scanline grid
    for row in 1..4 {
        let y = rect.min.y + row as f32 * 16.0;
        painter.line_segment(
            [
                egui::pos2(rect.min.x + 6.0, y),
                egui::pos2(rect.max.x - 6.0, y),
            ],
            egui::Stroke::new(
                0.5,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10),
            ),
        );
    }

    // Left badge & digital readouts
    painter.text(
        egui::pos2(rect.min.x + 12.0, rect.min.y + 16.0),
        egui::Align2::LEFT_CENTER,
        format!("[{}] {}", mode_tag, headline),
        egui::FontId::monospace(13.5),
        skin.lcd_text,
    );
    painter.text(
        egui::pos2(rect.min.x + 12.0, rect.min.y + 42.0),
        egui::Align2::LEFT_CENTER,
        subline,
        egui::FontId::monospace(11.5),
        skin.lcd_subtext,
    );

    // Right-hand 16-segment Stereo LED VU Meter
    let meter_w = 180.0;
    if rect.width() > 480.0 {
        let meter_x = rect.max.x - meter_w - 14.0;
        let rms_norm = ((rms_dbfs + 48.0) / 48.0).clamp(0.0, 1.0);
        let peak_norm = ((peak_dbfs + 48.0) / 48.0).clamp(0.0, 1.0);

        for (ch_idx, (ch_lbl, level)) in [("L", rms_norm), ("R", peak_norm)].iter().enumerate() {
            let y = rect.min.y + 18.0 + ch_idx as f32 * 22.0;
            painter.text(
                egui::pos2(meter_x - 12.0, y + 5.0),
                egui::Align2::LEFT_CENTER,
                *ch_lbl,
                egui::FontId::monospace(10.0),
                skin.lcd_text,
            );
            for seg in 0..16 {
                let frac = (seg + 1) as f32 / 16.0;
                let sx = meter_x + seg as f32 * 10.5;
                let seg_rect = egui::Rect::from_min_size(egui::pos2(sx, y), egui::vec2(8.0, 11.0));
                let col = if *level >= frac {
                    if seg < 11 {
                        egui::Color32::from_rgb(34, 197, 94)
                    } else if seg < 14 {
                        egui::Color32::from_rgb(250, 204, 21)
                    } else {
                        egui::Color32::from_rgb(239, 68, 68)
                    }
                } else {
                    skin.led_off
                };
                painter.rect_filled(seg_rect, 1.5, col);
            }
        }
    }
}

/// Renders an illuminated hardware step-sequencer pad with clickable Gate pad,
/// Accent LED pill, and 303 Slide LED pill.
pub fn illuminated_step_pad(
    ui: &mut egui::Ui,
    skin: &SkinPalette,
    step_num: usize,
    note_label: &str,
    active: &mut bool,
    accent: &mut bool,
    slide: &mut bool,
) -> bool {
    let desired_size = egui::vec2(62.0, 88.0);
    let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let mut changed = false;

    let pad_rect = egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.max.y - 24.0));
    let acc_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, rect.max.y - 21.0),
        egui::pos2(rect.center().x - 1.5, rect.max.y),
    );
    let sld_rect = egui::Rect::from_min_max(
        egui::pos2(rect.center().x + 1.5, rect.max.y - 21.0),
        rect.max,
    );

    let pad_resp = ui.interact(
        pad_rect,
        ui.scope_id().with(("step_pad", step_num)),
        egui::Sense::click(),
    );
    if pad_resp.clicked() {
        *active = !*active;
        changed = true;
    }
    let acc_resp = ui.interact(
        acc_rect,
        ui.scope_id().with(("step_acc", step_num)),
        egui::Sense::click(),
    );
    if acc_resp.clicked() {
        *accent = !*accent;
        changed = true;
    }
    let sld_resp = ui.interact(
        sld_rect,
        ui.scope_id().with(("step_sld", step_num)),
        egui::Sense::click(),
    );
    if sld_resp.clicked() {
        *slide = !*slide;
        changed = true;
    }

    let painter = ui.painter_at(rect);

    // Main hardware pad
    let pad_fill = if *active {
        skin.module_header
    } else {
        skin.module_bg
    };
    let pad_border = if *active {
        skin.led_on
    } else {
        skin.module_border
    };
    painter.rect_filled(pad_rect, 5.0, pad_fill);
    painter.rect_stroke(
        pad_rect,
        5.0,
        egui::Stroke::new(if *active { 1.8 } else { 1.0 }, pad_border),
        egui::epaint::StrokeKind::Inside,
    );

    // Glowing LED bar at top of pad
    let led_rect = egui::Rect::from_center_size(
        egui::pos2(pad_rect.center().x, pad_rect.min.y + 8.0),
        egui::vec2(26.0, 5.0),
    );
    painter.rect_filled(
        led_rect,
        2.5,
        if *active { skin.led_on } else { skin.led_off },
    );

    // Step number & Note name
    painter.text(
        egui::pos2(pad_rect.center().x, pad_rect.min.y + 23.0),
        egui::Align2::CENTER_CENTER,
        format!("#{:02}", step_num),
        egui::FontId::monospace(10.0),
        skin.text_muted,
    );
    painter.text(
        egui::pos2(pad_rect.center().x, pad_rect.min.y + 44.0),
        egui::Align2::CENTER_CENTER,
        note_label,
        egui::FontId::monospace(12.5),
        if *active {
            skin.lcd_text
        } else {
            skin.text_muted
        },
    );

    // Accent button pill
    painter.rect_filled(
        acc_rect,
        3.0,
        if *accent {
            skin.led_accent
        } else {
            skin.module_bg
        },
    );
    painter.rect_stroke(
        acc_rect,
        3.0,
        egui::Stroke::new(1.0, skin.module_border),
        egui::epaint::StrokeKind::Inside,
    );
    painter.text(
        acc_rect.center(),
        egui::Align2::CENTER_CENTER,
        "ACC",
        egui::FontId::monospace(8.5),
        if *accent {
            egui::Color32::BLACK
        } else {
            skin.text_muted
        },
    );

    // Slide button pill
    painter.rect_filled(
        sld_rect,
        3.0,
        if *slide {
            skin.arc_secondary
        } else {
            skin.module_bg
        },
    );
    painter.rect_stroke(
        sld_rect,
        3.0,
        egui::Stroke::new(1.0, skin.module_border),
        egui::epaint::StrokeKind::Inside,
    );
    painter.text(
        sld_rect.center(),
        egui::Align2::CENTER_CENTER,
        "SLD",
        egui::FontId::monospace(8.5),
        if *slide {
            egui::Color32::BLACK
        } else {
            skin.text_muted
        },
    );

    changed
}

fn format_control_value(value: f32, max: f32, unit: &str) -> String {
    if max >= 1000.0 {
        if unit.is_empty() {
            format!("{:.0}", value)
        } else {
            format!("{:.0} {}", value, unit)
        }
    } else if max <= 1.0 && unit == "%" {
        format!("{:.0}%", value * 100.0)
    } else if unit.is_empty() {
        format!("{:.2}", value)
    } else {
        format!("{:.1} {}", value, unit)
    }
}
