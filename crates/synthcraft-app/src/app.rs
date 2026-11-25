//! SynthCraft L3 Native Desktop Studio (`eframe` / `egui` 0.37).

use crate::audio_io::AudioHandle;
use crate::controls::{
    illuminated_step_pad, lcd_display_banner, rotary_knob, vertical_fader, SynthLayout, SynthSkin,
};
use crate::midi_io::{MidiHandle, MidiShared};
use crate::settings_ui::StudioSettings;
use crate::subapps;
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use synthcraft_core::midi::MidiEvent;
use synthcraft_core::{
    default_db_path, export_clap_descriptor, export_djcraft_stem_manifest, midi_to_freq_hz,
    write_wav_file, FilterMode, PcmProgram, SynthEngine, WaveformKind, APP_VERSION,
};

const ATTRIBUTION_MD: &str = include_str!("../../../ATTRIBUTION.md");

/// UI choices remembered between runs.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct UiPrefs {
    skin: Option<SynthSkin>,
    layout: Option<SynthLayout>,
    dark_mode: Option<bool>,
    zoom: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLocale {
    En,
    De,
}

impl AppLocale {
    pub fn label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::De => "Deutsch",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    ModularRack,
    M1ExtremeDsp,
    MidiDepot,
    DemoShowcase,
    SequencerEnv,
    ScopeSpectrum,
    ClapSoundcraft,
    DjcraftForge,
    Chat,
    Help,
    Settings,
    NextGenLab,
    ProLab,
    VisualsMpeDdsp,
    PerfMaster,
    PipeOrgan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTab {
    SubpageEncyclopedia,
    GetStarted,
    ModularRack,
    SequencerEnv,
    ClapDjcraft,
    CliMcp,
    TechnicalInfo,
}

/// Result of a background bank load: the requested spec and the decoded bank or an error.
pub type BankResult = (
    String,
    Result<std::sync::Arc<synthcraft_core::sampler::SampleBank>, String>,
);

pub struct SynthCraftApp {
    pub engine: SynthEngine,
    pub db_path: PathBuf,
    pub page: Page,
    pub subpage: usize,
    pub locale: AppLocale,
    pub dark_mode: bool,
    pub skin: SynthSkin,
    pub layout_mode: SynthLayout,
    pub settings: StudioSettings,
    pub zoom_factor: f32,
    pub status_line: String,
    // Workbench 3: Scope & Spectrum
    pub analysis_duration_ms: u32,
    // Workbench 5: DJCraft & WAV export
    pub djcraft_bars: usize,
    pub wav_export_path: String,
    pub djcraft_link_rx: synthcraft_core::DjcraftLinkReceiver,
    pub djcraft_link_sync: bool,
    pub djcraft_auto_camelot: bool,
    pub djcraft_pad_listen: bool,
    pub djcraft_aux_in_gain: f32,
    pub djcraft_target_deck: String,
    // Workbench 6: Assistant & Modals
    pub chat_input: String,
    pub chat_history: Vec<(String, String)>,
    pub help_tab: HelpTab,
    pub show_subpage_help: bool,
    pub help_search_query: String,
    pub help_selected_wb: usize,
    pub help_selected_sub: usize,
    // Pro Preset Browser state (129+ presets)
    pub preset_search: String,
    pub preset_category_filter: String,
    pub preset_engine_filter: String,
    pub preset_favorites_only: bool,
    pub favorite_patch_ids: Vec<String>,
    pub mutate_counter: u64,
    pub wt_harmonics: [f32; 16],
    // Workbench 15: Cathedral Pipe Organ & GrandOrgue Bach MIDI Depot
    pub organ_midi_dir: String,
    pub organ_midi_filter: String,
    pub organ_midi_entries: Vec<synthcraft_core::OrganMidiEntry>,
    pub show_palette: bool,
    pub palette_query: String,
    pub show_shortcuts: bool,
    pub show_licenses: bool,
    pub show_about: bool,
    // Live instrument state (see live.rs)
    pub audio: Option<AudioHandle>,
    pub audio_error: Option<String>,
    pub audio_devices: Vec<String>,
    pub midi: Option<MidiHandle>,
    pub midi_ports: Vec<String>,
    pub midi_shared: Arc<MidiShared>,
    pub midi_tx: Sender<MidiEvent>,
    pub midi_rx: Receiver<MidiEvent>,
    pub repaint_ctx: Option<egui::Context>,
    pub last_patch_json: String,
    pub saver: Option<crate::saver::SaveWorker>,
    pub dirty_since: Option<std::time::Instant>,
    pub persist_blocked: Option<String>,
    pub save_requested: bool,
    pub save_requested_in_flight: bool,
    pub last_saved_prefs: String,
    pub bank_input: String,
    pub bank_status: String,
    pub bank_loaded_spec: Option<String>,
    pub bank_loading: Option<String>,
    pub bank_tx: std::sync::mpsc::Sender<BankResult>,
    pub bank_rx: std::sync::mpsc::Receiver<BankResult>,
    pub bank_b_input: String,
    pub bank_b_loaded_spec: Option<String>,
    pub bank_b_loading: Option<String>,
    pub bank_b_tx: std::sync::mpsc::Sender<BankResult>,
    pub bank_b_rx: std::sync::mpsc::Receiver<BankResult>,
    pub bank_fetch_rx: Option<std::sync::mpsc::Receiver<crate::netfetch::FetchMsg>>,
    pub bank_fetch_id: Option<String>,
    pub sf2_presets: Vec<String>,
    pub last_opts: Option<synthcraft_core::RtOptions>,
    pub last_routing: Option<synthcraft_core::RackRouting>,
    pub cable_drag: Option<crate::rack_ui::CableDrag>,
    pub show_module_browser: bool,
    pub browser_category: Option<synthcraft_core::RackCategory>,
    pub browser_target_row: usize,
    pub undo_stack: Vec<synthcraft_core::SynthPatch>,
    pub redo_stack: Vec<synthcraft_core::SynthPatch>,
    pub ab_slot_other: Option<synthcraft_core::SynthPatch>,
    pub ab_is_b: bool,
    pub clap_discovered: Vec<PathBuf>,
    pub clap_loaded_info: Option<synthcraft_core::ClapPluginInfo>,
    pub clap_loaded_params: Vec<synthcraft_core::ClapParamInfo>,
    pub held_ui_note: Option<u8>,
    pub seq_record: bool,
    pub seq_record_cursor: usize,
    pub seq_bar_page: usize,
    pub drum_bar_page: usize,
    pub drum_selected_voice: usize,
    pub genopatch_wav_path: String,
    pub genopatch_gens: usize,
    pub genopatch_pop: usize,
    pub genopatch_report: Option<synthcraft_core::GenopatchReport>,
    pub waterfall_history: Vec<[f32; 32]>,
    pub mpe_pad_pitch_st: f32,
    pub mpe_pad_timbre: f32,
    pub mpe_pad_pressure: f32,
    pub kbd_down: HashMap<egui::Key, u8>,
    pub kbd_octave: i32,
    pub analysis_cache: Option<((String, u32, u32), synthcraft_core::SpectrumAnalysisReport)>,
}

impl Default for SynthCraftApp {
    fn default() -> Self {
        Self::with_db_path(default_db_path())
    }
}

impl SynthCraftApp {
    /// Build the app on `db_path` and apply the stored studio settings and UI preferences.
    pub fn with_db_path(db_path: PathBuf) -> Self {
        let mut app = Self::build(db_path);
        app.load_prefs();
        app
    }

    fn build(db_path: PathBuf) -> Self {
        // A database that cannot be opened is never replaced: the app runs on factory data,
        // saving is disabled, and the reason is shown.
        let (engine, persist_blocked) = match SynthEngine::open_or_seed(&db_path) {
            Ok(e) => (e, None),
            Err(e) => (
                SynthEngine::factory_seed(),
                Some(format!(
                    "Cannot use {}: {e}. Running on factory data; nothing will be saved.",
                    db_path.display()
                )),
            ),
        };
        let (midi_tx, midi_rx) = channel();
        let (bank_tx, bank_rx) = channel();
        let (bank_b_tx, bank_b_rx) = channel();
        let midi_shared = MidiShared::new(
            engine.midi_hw.midi_channel,
            engine.midi_hw.pitch_bend_semitones,
        );
        Self {
            engine,
            db_path,
            page: Page::ModularRack,
            subpage: 0,
            locale: AppLocale::En,
            dark_mode: true,
            skin: SynthSkin::ObsidianVcv,
            layout_mode: SynthLayout::WorkbenchSplit,
            settings: StudioSettings::default(),
            zoom_factor: 1.0,
            status_line: format!(
                "SynthCraft v{APP_VERSION} ready -- CLAP 1.2.2 (dev.sandraschi.synthcraft) & DJCraft Stem Forge"
            ),
            analysis_duration_ms: 250,
            djcraft_bars: 4,
            wav_export_path: "vienna_acid_303_128bpm_8A.wav".into(),
            djcraft_link_rx: synthcraft_core::DjcraftLinkReceiver::new(),
            djcraft_link_sync: false,
            djcraft_auto_camelot: false,
            djcraft_pad_listen: true,
            djcraft_aux_in_gain: 0.0,
            djcraft_target_deck: "B".into(),
            chat_input: String::new(),
            chat_history: vec![(
                "SynthCraft".into(),
                "no model selected -- connect an MCP 2026-07-28 client or local Ollama endpoint to design synth patches, CLAP mappings, and DJCraft stems.".into(),
            )],
            help_tab: HelpTab::SubpageEncyclopedia,
            show_subpage_help: false,
            help_search_query: String::new(),
            help_selected_wb: 0,
            help_selected_sub: 0,
            preset_search: String::new(),
            preset_category_filter: "All".into(),
            preset_engine_filter: "All".into(),
            preset_favorites_only: false,
            favorite_patch_ids: vec![
                "vienna_acid_303".into(),
                "organ_silbermann_pleno".into(),
                "organ_bach_toccata_tutti".into(),
            ],
            mutate_counter: 0,
            wt_harmonics: [
                1.0, 0.5, 0.33, 0.25, 0.2, 0.16, 0.14, 0.12, 0.11, 0.1, 0.09, 0.08, 0.07, 0.06,
                0.05, 0.04,
            ],
            organ_midi_dir: synthcraft_core::default_grandorgue_midi_dir()
                .display()
                .to_string(),
            organ_midi_filter: String::new(),
            organ_midi_entries: synthcraft_core::scan_organ_midi_depot(
                &synthcraft_core::default_grandorgue_midi_dir(),
            ),
            show_palette: false,
            palette_query: String::new(),
            show_shortcuts: false,
            show_licenses: false,
            show_about: false,
            audio: None,
            audio_error: None,
            audio_devices: Vec::new(),
            midi: None,
            midi_ports: Vec::new(),
            midi_shared,
            midi_tx,
            midi_rx,
            repaint_ctx: None,
            last_patch_json: String::new(),
            saver: Some(crate::saver::SaveWorker::spawn()),
            dirty_since: None,
            persist_blocked,
            save_requested: false,
            save_requested_in_flight: false,
            last_saved_prefs: String::new(),
            bank_input: String::new(),
            bank_status: String::new(),
            bank_loaded_spec: None,
            bank_loading: None,
            bank_tx,
            bank_rx,
            bank_b_input: String::new(),
            bank_b_loaded_spec: None,
            bank_b_loading: None,
            bank_b_tx,
            bank_b_rx,
            bank_fetch_rx: None,
            bank_fetch_id: None,
            sf2_presets: Vec::new(),
            last_opts: None,
            last_routing: None,
            cable_drag: None,
            show_module_browser: false,
            browser_category: None,
            browser_target_row: 1,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            ab_slot_other: None,
            ab_is_b: false,
            clap_discovered: Vec::new(),
            clap_loaded_info: None,
            clap_loaded_params: Vec::new(),
            held_ui_note: None,
            seq_record: false,
            seq_record_cursor: 0,
            seq_bar_page: 0,
            drum_bar_page: 0,
            drum_selected_voice: 0,
            genopatch_wav_path: "demo_1990_m1_piano.wav".into(),
            genopatch_gens: 6,
            genopatch_pop: 32,
            genopatch_report: None,
            waterfall_history: Vec::new(),
            mpe_pad_pitch_st: 0.0,
            mpe_pad_timbre: 0.5,
            mpe_pad_pressure: 0.0,
            kbd_down: HashMap::new(),
            kbd_octave: 3,
            analysis_cache: None,
        }
    }

    pub fn goto(&mut self, page: Page, sub: usize) {
        self.page = page;
        self.subpage = sub;
    }

    pub fn persist_state(&mut self) {
        self.persist();
    }

    fn persist(&mut self) {
        self.mark_dirty();
    }

    /// Note that something changed. The debounced saver writes it shortly after.
    pub fn mark_dirty(&mut self) {
        if self.dirty_since.is_none() {
            self.dirty_since = Some(std::time::Instant::now());
        }
    }

    /// Save as soon as the next frame runs (File menu), regardless of the auto-save setting.
    pub fn save_now(&mut self) {
        self.save_requested = true;
        self.dirty_since = Some(std::time::Instant::now() - std::time::Duration::from_secs(5));
    }

    fn settings_blobs(&self) -> Vec<(String, String)> {
        let prefs = UiPrefs {
            skin: Some(self.skin),
            layout: Some(self.layout_mode),
            dark_mode: Some(self.dark_mode),
            zoom: Some(self.zoom_factor),
        };
        vec![
            (
                "app:studio_settings".to_string(),
                serde_json::to_string(&self.settings).unwrap_or_default(),
            ),
            (
                "app:ui_prefs".to_string(),
                serde_json::to_string(&prefs).unwrap_or_default(),
            ),
        ]
    }

    fn load_prefs(&mut self) {
        if self.persist_blocked.is_some() {
            return;
        }
        let get = |key: &str| synthcraft_core::db::load_setting(&self.db_path, key);
        let settings = get("app:studio_settings");
        let prefs = get("app:ui_prefs");
        match settings {
            Ok(Some(raw)) => match serde_json::from_str::<StudioSettings>(&raw) {
                Ok(s) => self.settings = s,
                Err(e) => {
                    self.status_line =
                        format!("Stored studio settings unreadable ({e}); defaults in use.")
                }
            },
            Ok(None) => {}
            Err(e) => self.status_line = format!("Could not load studio settings: {e}"),
        }
        if let Ok(Some(raw)) = prefs {
            if let Ok(p) = serde_json::from_str::<UiPrefs>(&raw) {
                if let Some(s) = p.skin {
                    self.set_skin(s);
                }
                if let Some(l) = p.layout {
                    self.layout_mode = l;
                }
                if let Some(d) = p.dark_mode {
                    self.dark_mode = d;
                }
                if let Some(z) = p.zoom {
                    self.zoom_factor = z.clamp(0.75, 1.75);
                }
            }
        }
        for w in &self.engine.load_warnings {
            self.status_line = format!("Database warning: {w}");
        }
        if let Some(msg) = &self.persist_blocked {
            self.status_line.clone_from(msg);
        }
        self.last_saved_prefs = self.settings_blobs().into_iter().map(|(_, v)| v).collect();
    }

    /// Debounced background save. Called once per frame.
    /// Turn a panel edit (`before` -> `after`) into engine commands and run them, so a panel
    /// can do nothing an agent cannot do through the same commands (AGENTS rule 14).
    pub fn apply_patch_edit(
        &mut self,
        before: &synthcraft_core::SynthPatch,
        after: &synthcraft_core::SynthPatch,
    ) {
        let cmds = before.edit_commands(after);
        if cmds.is_empty() {
            return;
        }
        // Coalesce rapid knob drags by keeping up to 64 undo snapshots.
        if self.undo_stack.len() >= 64 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(before.clone());
        self.redo_stack.clear();
        for c in cmds {
            if let Err(e) = c.dispatch(&mut self.engine) {
                self.status_line = format!("Edit rejected: {e}");
            }
        }
        self.persist();
    }

    pub fn undo_patch(&mut self) {
        let Some(prev) = self.undo_stack.pop() else {
            self.status_line = "Nothing to undo".into();
            return;
        };
        let cur = self.engine.active_patch().clone();
        self.redo_stack.push(cur.clone());
        for c in cur.edit_commands(&prev) {
            let _ = c.dispatch(&mut self.engine);
        }
        self.status_line = format!("Undid patch edit ({} left)", self.undo_stack.len());
        self.persist();
    }

    pub fn redo_patch(&mut self) {
        let Some(next) = self.redo_stack.pop() else {
            self.status_line = "Nothing to redo".into();
            return;
        };
        let cur = self.engine.active_patch().clone();
        self.undo_stack.push(cur.clone());
        for c in cur.edit_commands(&next) {
            let _ = c.dispatch(&mut self.engine);
        }
        self.status_line = format!("Redid patch edit ({} left)", self.redo_stack.len());
        self.persist();
    }

    pub fn toggle_ab_compare(&mut self) {
        let cur = self.engine.active_patch().clone();
        match self.ab_slot_other.replace(cur.clone()) {
            None => {
                self.ab_is_b = true;
                self.status_line =
                    "Stored A snapshot -> now editing B (click A/B to compare)".into();
            }
            Some(other) => {
                for c in cur.edit_commands(&other) {
                    let _ = c.dispatch(&mut self.engine);
                }
                self.ab_is_b = !self.ab_is_b;
                self.status_line = format!(
                    "A/B Compare: now auditioning slot {}",
                    if self.ab_is_b { "B" } else { "A" }
                );
                self.persist();
            }
        }
    }

    /// Poll the background sample-bank download and, when it finishes, load the bank through
    /// the engine command so persistence and the live audio sync follow the normal path.
    pub fn pump_bank_fetch(&mut self, ctx: &egui::Context) {
        use crate::netfetch::FetchMsg;
        let Some(rx) = self.bank_fetch_rx.as_ref() else {
            return;
        };
        let mut finished: Option<Result<String, String>> = None;
        while let Ok(msg) = rx.try_recv() {
            match msg {
                FetchMsg::Progress(s) => self.bank_status = s,
                FetchMsg::Done(r) => finished = Some(r),
            }
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
        let Some(result) = finished else {
            return;
        };
        self.bank_fetch_rx = None;
        self.bank_fetch_id = None;
        match result {
            Ok(spec) => {
                let mix = {
                    let m = self.engine.active_patch().sampler.mix;
                    if m < 0.05 {
                        0.8
                    } else {
                        m
                    }
                };
                let cmd = synthcraft_core::Command::SetSamplerBank {
                    bank: Some(spec.clone()),
                    mix,
                };
                match cmd.dispatch(&mut self.engine) {
                    Ok(_) => {
                        self.bank_input = spec;
                        self.bank_status = "Bank loaded.".to_string();
                        self.persist_state();
                    }
                    Err(e) => self.bank_status = format!("Bank not loaded: {e}"),
                }
            }
            Err(e) => self.bank_status = format!("Bank not loaded: {e}"),
        }
    }

    pub fn pump_saves(&mut self, ctx: &egui::Context) {
        use std::time::Duration;
        let blobs = self.settings_blobs();
        let joined: String = blobs.iter().map(|(_, v)| v.as_str()).collect();
        if joined != self.last_saved_prefs {
            self.mark_dirty();
        }
        if let Some(w) = &self.saver {
            for r in w.poll() {
                match r {
                    Ok(()) => {
                        if self.save_requested_in_flight {
                            self.save_requested_in_flight = false;
                            self.status_line = format!("Saved to {}", self.db_path.display());
                        }
                    }
                    Err(e) => {
                        self.save_requested_in_flight = false;
                        self.status_line = format!("SAVE FAILED: {e}");
                    }
                }
            }
        }
        let Some(since) = self.dirty_since else {
            return;
        };
        let auto = self.settings.auto_save_sqlite_on_tweak || self.save_requested;
        if since.elapsed() < Duration::from_millis(400) || !auto {
            ctx.request_repaint_after(Duration::from_millis(450));
            return;
        }
        self.dirty_since = None;
        if let Some(msg) = &self.persist_blocked {
            self.status_line.clone_from(msg);
            self.save_requested = false;
            return;
        }
        if let Some(w) = &self.saver {
            w.submit(crate::saver::SaveJob {
                path: self.db_path.clone(),
                engine: self.engine.clone(),
                settings: blobs,
            });
            self.last_saved_prefs = joined;
            self.save_requested_in_flight = self.save_requested;
            self.save_requested = false;
        }
    }

    pub fn set_skin(&mut self, skin: SynthSkin) {
        self.skin = skin;
        self.dark_mode = skin.palette().is_dark;
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let pal = self.skin.palette();
        let theme = if self.dark_mode {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        };
        ctx.set_theme(theme);
        let mut style = (*ctx.style_of(theme)).clone();
        style.visuals.override_text_color = Some(pal.text_primary);
        style.visuals.panel_fill = pal.panel_bg;
        style.visuals.window_fill = pal.module_bg;
        ctx.set_style_of(theme, style);
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let zoom_delta = ctx.input(|i| {
            if i.modifiers.ctrl {
                i.smooth_scroll_delta.y
            } else {
                0.0
            }
        });
        if zoom_delta.abs() > 0.1 {
            let step = if zoom_delta > 0.0 { 0.1 } else { -0.1 };
            self.zoom_factor = (self.zoom_factor + step).clamp(0.75, 1.75);
            ctx.set_zoom_factor(self.zoom_factor);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            let (wb, sub) = crate::help_docs::map_page_to_doc(self.page, self.subpage);
            self.help_selected_wb = wb;
            self.help_selected_sub = sub;
            self.help_tab = HelpTab::SubpageEncyclopedia;
            self.goto(Page::Help, 0);
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::K)) {
            self.show_palette = !self.show_palette;
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Comma)) {
            self.goto(Page::Settings, 0);
        }
        if ctx.input(|i| i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::Z)) {
            self.undo_patch();
        }
        if ctx.input(|i| {
            i.modifiers.ctrl
                && (i.key_pressed(egui::Key::Y)
                    || (i.modifiers.shift && i.key_pressed(egui::Key::Z)))
        }) {
            self.redo_patch();
        }
    }

    pub fn render_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Save Patch Bank to SQLite").clicked() {
                    self.save_now();
                    self.status_line = format!("Saving to {} ...", self.db_path.display());
                    ui.close();
                }
                if ui
                    .button("Studio Settings & Preferences (Ctrl+,)")
                    .clicked()
                {
                    self.goto(Page::Settings, 0);
                    ui.close();
                }
                if ui.button("Reset Factory Patches").clicked() {
                    self.engine = SynthEngine::factory_seed();
                    self.persist();
                    ui.close();
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui
                    .button("Studio Settings & Preferences (Ctrl+,)")
                    .clicked()
                {
                    self.goto(Page::Settings, 0);
                    ui.close();
                }
                if ui.button("Randomize Acid Sequence").clicked() {
                    let scale = [45_u8, 48, 50, 52, 55, 57, 60, 62];
                    let before = self.engine.active_patch().clone();
                    let mut edited = before.clone();
                    let p = &mut edited;
                    for (i, step) in p.steps.iter_mut().enumerate() {
                        step.midi_note = scale[(i * 3 + 1) % scale.len()];
                        step.accent = i % 3 == 0;
                        step.slide = i % 4 == 2;
                    }
                    self.persist();
                    ui.close();
                    self.apply_patch_edit(&before, &edited);
                }
            });
            ui.menu_button("View", |ui| {
                if ui.button("Dark").clicked() {
                    self.set_skin(SynthSkin::ObsidianVcv);
                    ui.close();
                }
                if ui.button("Light").clicked() {
                    self.set_skin(SynthSkin::DaylightStudio);
                    ui.close();
                }
                ui.separator();
                ui.strong("Hardware Skin Theme:");
                for sk in SynthSkin::all() {
                    if ui
                        .selectable_label(
                            self.skin == *sk,
                            format!("{} -- {}", sk.label(), sk.description()),
                        )
                        .clicked()
                    {
                        self.set_skin(*sk);
                        ui.close();
                    }
                }
                ui.separator();
                ui.strong("Studio Layout Mode:");
                for lay in SynthLayout::all() {
                    if ui
                        .selectable_label(self.layout_mode == *lay, lay.label())
                        .clicked()
                    {
                        self.layout_mode = *lay;
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("English").clicked() {
                    self.locale = AppLocale::En;
                    ui.close();
                }
                if ui.button("Deutsch").clicked() {
                    self.locale = AppLocale::De;
                    ui.close();
                }
            });
            ui.menu_button("Insert", |ui| {
                let menu_patches: Vec<(String, String)> = self
                    .engine
                    .patches
                    .iter()
                    .map(|p| {
                        (
                            p.id.clone(),
                            format!("Patch: {} ({:.0} BPM {})", p.name, p.bpm, p.camelot_key),
                        )
                    })
                    .collect();
                for (id, label) in menu_patches {
                    if ui.button(label).clicked() {
                        let _ = self.engine.select_patch(&id);
                        self.persist();
                        ui.close();
                    }
                }
            });
            ui.menu_button("Modify", |ui| {
                if ui.button("Boost Filter Resonance (+1.5 Q)").clicked() {
                    let before = self.engine.active_patch().clone();
                    let mut edited = before.clone();
                    let p = &mut edited;
                    p.resonance_q = (p.resonance_q + 1.5).clamp(0.5, 12.0);
                    self.persist();
                    ui.close();
                    self.apply_patch_edit(&before, &edited);
                }
            });
            ui.menu_button("Commands", |ui| {
                if ui.button("Inspect 32-Band DFT Spectrum").clicked() {
                    self.goto(Page::ScopeSpectrum, 1);
                    ui.close();
                }
                if ui
                    .button("Export CLAP 1.2.2 Descriptor for SoundCraft")
                    .clicked()
                {
                    self.goto(Page::ClapSoundcraft, 0);
                    ui.close();
                }
                if ui.button("Export DJCraft Stem & Cue Points").clicked() {
                    self.goto(Page::DjcraftForge, 0);
                    ui.close();
                }
            });
            ui.menu_button("Window", |ui| {
                for (label, p, s) in [
                    ("Modular Rack & VCO/VCF", Page::ModularRack, 0),
                    ("16-Step Acid Sequencer", Page::SequencerEnv, 0),
                    ("Oscilloscope & Spectrum", Page::ScopeSpectrum, 0),
                    ("CLAP & SoundCraft Bridge", Page::ClapSoundcraft, 0),
                    ("DJCraft Stem & WAV Forge", Page::DjcraftForge, 0),
                    ("Studio Settings & Preferences (Ctrl+,)", Page::Settings, 0),
                    ("Chat", Page::Chat, 0),
                    ("Help (F1)", Page::Help, 0),
                ] {
                    if ui.button(label).clicked() {
                        self.goto(p, s);
                        ui.close();
                    }
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("Command Palette (Ctrl+K)").clicked() {
                    self.show_palette = true;
                    ui.close();
                }
                if ui.button("Keyboard Shortcuts").clicked() {
                    self.show_shortcuts = true;
                    ui.close();
                }
                if ui.button("Documentation (F1)").clicked() {
                    self.goto(Page::Help, 0);
                    ui.close();
                }
                if ui.button("Licenses").clicked() {
                    self.show_licenses = true;
                    ui.close();
                }
                if ui.button("About SynthCraft").clicked() {
                    self.show_about = true;
                    ui.close();
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let theme_lbl = if self.dark_mode { "Light" } else { "Dark" };
                if ui.button(theme_lbl).clicked() {
                    if self.dark_mode {
                        self.set_skin(SynthSkin::DaylightStudio);
                    } else {
                        self.set_skin(SynthSkin::ObsidianVcv);
                    }
                }
                let loc_lbl = self.locale.label();
                if ui.button(loc_lbl).clicked() {
                    self.locale = match self.locale {
                        AppLocale::En => AppLocale::De,
                        AppLocale::De => AppLocale::En,
                    };
                }
                if ui
                    .selectable_label(self.page == Page::Settings, "Settings")
                    .on_hover_text("Open 6-Tab Studio Settings & Preferences (Ctrl+,)")
                    .clicked()
                {
                    self.goto(Page::Settings, 0);
                }
                if ui
                    .selectable_label(self.show_subpage_help, "? Guide")
                    .on_hover_text("Toggle dedicated human-readable guide, history & famous tracks for this subpage (F1 for full Encyclopedia)")
                    .clicked()
                {
                    self.show_subpage_help = !self.show_subpage_help;
                }
            });
        });

        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.strong("Layout:");
            for lay in SynthLayout::all() {
                if ui
                    .selectable_label(self.layout_mode == *lay, lay.label())
                    .clicked()
                {
                    self.layout_mode = *lay;
                }
            }
            ui.separator();
            ui.strong("Skin:");
            for sk in SynthSkin::all() {
                if ui
                    .selectable_label(self.skin == *sk, sk.label())
                    .on_hover_text(sk.description())
                    .clicked()
                {
                    self.set_skin(*sk);
                }
            }
        });
        let _ = ctx;
    }

    pub fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.heading("SynthCraft");
        ui.small("Modular DSP, CLAP & DJ Stem Studio");
        ui.separator();

        for wb in subapps::registry() {
            let subs = wb.subpages();
            let has_active = subs
                .iter()
                .any(|sp| sp.page == self.page && sp.sub == self.subpage);
            egui::CollapsingHeader::new(RichText::new(wb.title(self.locale)).strong())
                .id_salt(wb.id())
                .default_open(has_active || wb.id() == "modular_rack")
                .show(ui, |ui| {
                    for sp in subs {
                        let active = self.page == sp.page && self.subpage == sp.sub;
                        if ui.selectable_label(active, sp.label(self.locale)).clicked() {
                            self.goto(sp.page, sp.sub);
                        }
                    }
                });
        }
    }

    pub fn render_lcd_header(&mut self, ui: &mut egui::Ui) {
        let pal = self.skin.palette();
        let (rms_db, peak_db) = self.level_dbfs();
        let ap = self.engine.active_patch();
        let m1_tag = ap.m1_program.map(PcmProgram::as_str).unwrap_or("OFF");
        let headline = format!(
            "PROG: {} ({:.0} BPM | KEY {})",
            ap.name, ap.bpm, ap.camelot_key
        );
        let subline = format!(
            "VCO: {}+{} | PCM: {} | X-DSP: {} | VCF: {:.0} Hz (Q {:.1})",
            ap.vco1_wave.as_str(),
            ap.vco2_wave.as_str(),
            m1_tag,
            ap.extreme_dsp.as_str(),
            ap.cutoff_hz,
            ap.resonance_q
        );
        lcd_display_banner(
            ui,
            &pal,
            self.skin.label(),
            &headline,
            &subline,
            rms_db,
            peak_db,
        );
        let transport = match (&self.audio, &self.audio_error) {
            (Some(a), _) => format!(
                "{} | {} voices | {} Hz",
                if a.meters.playing() {
                    format!("playing step {}", a.meters.step() + 1)
                } else {
                    "stopped".to_string()
                },
                a.meters.voices(),
                a.info.sample_rate
            ),
            (None, Some(e)) => format!("audio unavailable: {e}"),
            (None, None) => "audio not started".to_string(),
        };
        ui.horizontal_wrapped(|ui| {
            // Always-visible Patch selector (< [Patch] >) across every workbench
            let patches: Vec<(String, String, String)> = self
                .engine
                .patches
                .iter()
                .map(|p| (p.id.clone(), p.name.clone(), p.category.clone()))
                .collect();
            if !patches.is_empty() {
                let cur_idx = patches
                    .iter()
                    .position(|(id, _, _)| *id == self.engine.active_patch_id)
                    .unwrap_or(0);
                let cur_name = patches[cur_idx].1.clone();
                let mut next_id: Option<String> = None;
                if ui
                    .small_button("<")
                    .on_hover_text("Previous patch")
                    .clicked()
                {
                    let prev = (cur_idx + patches.len() - 1) % patches.len();
                    next_id = Some(patches[prev].0.clone());
                }
                egui::ComboBox::from_label("Patch")
                    .width(210.0)
                    .selected_text(cur_name)
                    .show_ui(ui, |ui| {
                        for (id, name, cat) in &patches {
                            let label = format!("{name} [{cat}]");
                            if ui
                                .selectable_label(self.engine.active_patch_id == *id, label)
                                .clicked()
                            {
                                next_id = Some(id.clone());
                            }
                        }
                    });
                if ui.small_button(">").on_hover_text("Next patch").clicked() {
                    let next = (cur_idx + 1) % patches.len();
                    next_id = Some(patches[next].0.clone());
                }
                if let Some(id) = next_id {
                    if (synthcraft_core::Command::LoadPatch { patch_id: id })
                        .dispatch(&mut self.engine)
                        .is_ok()
                    {
                        self.undo_stack.clear();
                        self.redo_stack.clear();
                        self.ab_slot_other = None;
                        self.ab_is_b = false;
                        self.status_line =
                            format!("Loaded patch: {}", self.engine.active_patch().name);
                        self.persist();
                    }
                }
                if ui
                    .add_enabled(
                        !self.undo_stack.is_empty(),
                        egui::Button::new("Undo").small(),
                    )
                    .on_hover_text("Undo last patch edit (Ctrl+Z)")
                    .clicked()
                {
                    self.undo_patch();
                }
                if ui
                    .add_enabled(
                        !self.redo_stack.is_empty(),
                        egui::Button::new("Redo").small(),
                    )
                    .on_hover_text("Redo patch edit (Ctrl+Y)")
                    .clicked()
                {
                    self.redo_patch();
                }
                let ab_lbl = if self.ab_slot_other.is_none() {
                    "A/B"
                } else if self.ab_is_b {
                    "Slot B"
                } else {
                    "Slot A"
                };
                if ui
                    .small_button(ab_lbl)
                    .on_hover_text("Snapshot current patch to A/B and toggle between versions")
                    .clicked()
                {
                    self.toggle_ab_compare();
                }
                ui.separator();
            }

            let label = if self.is_playing() {
                "Stop sequencer (Space)"
            } else {
                "Play sequencer (Space)"
            };
            if ui.button(label).clicked() {
                self.toggle_play();
            }
            let rec_label = if self.seq_record {
                if self.is_playing() {
                    "REC LIVE".to_string()
                } else {
                    format!("REC [Step {}]", self.seq_record_cursor + 1)
                }
            } else {
                "Rec".to_string()
            };
            let rec_btn = if self.seq_record {
                egui::Button::new(
                    egui::RichText::new(rec_label)
                        .strong()
                        .color(egui::Color32::WHITE),
                )
                .fill(egui::Color32::from_rgb(200, 38, 38))
            } else {
                egui::Button::new(rec_label)
            };
            if ui
                .add(rec_btn)
                .on_hover_text(
                    "Arm Live Sequencer Record: overdubs into playing step when running, or step-records & advances when stopped",
                )
                .clicked()
            {
                self.seq_record = !self.seq_record;
                if self.seq_record {
                    self.seq_record_cursor = self.playing_step().unwrap_or(0);
                    self.status_line = if self.is_playing() {
                        "Live Sequencer Record armed (overdubbing playing steps)".into()
                    } else {
                        format!(
                            "Step Record armed at step {} -- play notes on keyboard or MIDI",
                            self.seq_record_cursor + 1
                        )
                    };
                } else {
                    self.status_line = "Sequencer Record disarmed".into();
                }
            }
            if ui.button("All notes off").clicked() {
                if let Some(a) = &self.audio {
                    a.send(crate::audio_io::AudioMsg::AllNotesOff);
                }
            }
            ui.monospace(transport);
            ui.small(format!(
                "Keys Z..M / Q..I play notes, -/= octave (C{})",
                self.kbd_octave
            ));
        });
    }

    pub fn render_page(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("synthcraft_main_scroll")
            .show(ui, |ui| {
                self.render_lcd_header(ui);
                ui.add_space(6.0);

                // Special pages (Help, Chat, Settings) always render directly when opened
                if matches!(self.page, Page::Help | Page::Chat | Page::Settings) {
                    match self.page {
                        Page::Chat => self.render_chat(ui),
                        Page::Help => self.render_help_view(ui),
                        Page::Settings => crate::settings_ui::render_studio_settings(self, ui),
                        _ => {}
                    }
                    return;
                }

                match self.layout_mode {
                    SynthLayout::WorkbenchSplit => match self.page {
                        Page::ModularRack => self.render_modular_rack(ui),
                        Page::M1ExtremeDsp => crate::rack_ui::render_m1_and_extreme_dsp(self, ui),
                        Page::MidiDepot => crate::rack_ui::render_midi_and_depot(self, ui),
                        Page::DemoShowcase => crate::rack_ui::render_demo_showcase(self, ui),
                        Page::SequencerEnv => self.render_sequencer_env(ui),
                        Page::ScopeSpectrum => self.render_scope_spectrum(ui),
                        Page::ClapSoundcraft => self.render_clap_soundcraft(ui),
                        Page::DjcraftForge => self.render_djcraft_forge(ui),
                        Page::Chat => self.render_chat(ui),
                        Page::Help => self.render_help_view(ui),
                        Page::Settings => crate::settings_ui::render_studio_settings(self, ui),
                        Page::NextGenLab => crate::nextgen_ui::render_nextgen_lab(self, ui),
                        Page::ProLab => crate::pro_lab_ui::render_pro_lab(self, ui),
                        Page::VisualsMpeDdsp => {
                            crate::visualizers_ui::render_visuals_mpe_ddsp(self, ui)
                        }
                        Page::PerfMaster => crate::perf_master_ui::render_perf_master(self, ui),
                        Page::PipeOrgan => {
                            crate::pipe_organ_ui::render_pipe_organ_workbench(self, ui)
                        }
                    },
                    SynthLayout::EurorackBay => {
                        crate::rack_ui::render_vcv_eurorack_canvas(self, ui);
                        ui.add_space(6.0);
                        self.render_rotary_console_strip(ui);
                        ui.add_space(6.0);
                        crate::rack_ui::render_piano_keyboard_strip(self, ui);
                    }
                    SynthLayout::Workstation1990 => {
                        crate::rack_ui::render_m1_and_extreme_dsp(self, ui);
                        ui.add_space(6.0);
                        self.render_sequencer_env(ui);
                        ui.add_space(6.0);
                        crate::rack_ui::render_piano_keyboard_strip(self, ui);
                    }
                    SynthLayout::LiveCockpit => {
                        self.render_rotary_console_strip(ui);
                        ui.add_space(6.0);
                        self.render_scope_spectrum(ui);
                        ui.add_space(6.0);
                        crate::rack_ui::render_piano_keyboard_strip(self, ui);
                    }
                }
            });
    }

    fn render_rotary_console_strip(&mut self, ui: &mut egui::Ui) {
        let pal = self.skin.palette();
        let mut changed = false;
        {
            let before = self.engine.active_patch().clone();
            let mut edited = before.clone();
            let p = &mut edited;
            ui.group(|ui| {
                ui.strong("Hardware Rotary Potentiometer & Fader Performance Console (Drag or Shift+Drag; Double-Click Reset)");
                ui.horizontal_wrapped(|ui| {
                    changed |= rotary_knob(ui, &pal, "DETUNE", &mut p.vco2_detune_cents, -50.0, 50.0, "ct");
                    changed |= rotary_knob(ui, &pal, "VCO MIX", &mut p.vco_mix, 0.0, 1.0, "%");
                    changed |= rotary_knob(ui, &pal, "FM INDEX", &mut p.fm_index, 0.0, 4.0, "idx");
                    changed |= rotary_knob(ui, &pal, "PWM DUTY", &mut p.pulse_width, 0.05, 0.95, "%");
                    changed |= rotary_knob(ui, &pal, "CUTOFF", &mut p.cutoff_hz, 40.0, 12000.0, "Hz");
                    changed |= rotary_knob(ui, &pal, "RESON Q", &mut p.resonance_q, 0.5, 12.0, "Q");
                    changed |= rotary_knob(ui, &pal, "ENV MOD", &mut p.env_mod_depth, 0.0, 1.0, "%");
                    changed |= rotary_knob(ui, &pal, "DRIVE", &mut p.drive_saturation, 0.0, 3.0, "x");
                    changed |= rotary_knob(ui, &pal, "PCM", &mut p.m1_layer_mix, 0.0, 1.0, "%");
                    changed |= rotary_knob(ui, &pal, "X-MORPH", &mut p.extreme_morph, 0.0, 1.0, "%");
                });
            });
            self.apply_patch_edit(&before, &edited);
        }
        if changed {
            self.persist();
        }
    }

    fn render_modular_rack(&mut self, ui: &mut egui::Ui) {
        ui.heading("1. 3U Eurorack & Dual VCO/VCF Bay");
        ui.horizontal_wrapped(|ui| {
            ui.label("Factory Patches:");
            let patch_list: Vec<(String, String)> = self
                .engine
                .patches
                .iter()
                .map(|p| (p.id.clone(), p.name.clone()))
                .collect();
            for (id, name) in patch_list {
                if ui
                    .selectable_label(self.engine.active_patch_id == id, name)
                    .clicked()
                {
                    let _ = self.engine.select_patch(&id);
                    self.persist();
                }
            }
        });
        ui.separator();

        if self.subpage == 0 {
            crate::mod_wavetable_ui::render_drag_drop_mod_bar(self, ui);
            ui.add_space(4.0);
            crate::mod_wavetable_ui::render_wavetable_frame_editor(self, ui);
            ui.add_space(6.0);
            crate::rack_ui::render_vcv_eurorack_canvas(self, ui);
            ui.add_space(6.0);
        }

        if self.subpage == 1 {
            crate::preset_browser_ui::render_pro_preset_browser(self, ui);
            ui.add_space(6.0);
        }

        let pal = self.skin.palette();
        let mut changed = false;
        {
            let before = self.engine.active_patch().clone();
            let mut edited = before.clone();
            let p = &mut edited;
            ui.columns(2, |cols| {
                cols[0].group(|ui| {
                    ui.strong("Dual Oscillators (VCO 1 & VCO 2) + FM Rotary Console");
                    ui.horizontal_wrapped(|ui| {
                        ui.label("VCO 1:");
                        for w in [
                            WaveformKind::Sine,
                            WaveformKind::Saw,
                            WaveformKind::Square,
                            WaveformKind::Triangle,
                            WaveformKind::PulsePwm,
                            WaveformKind::SuperSaw,
                            WaveformKind::WhiteNoise,
                            WaveformKind::SubOctave,
                            WaveformKind::AtmoPad,
                            WaveformKind::Wavetable,
                        ] {
                            if ui.selectable_label(p.vco1_wave == w, w.as_str()).clicked() {
                                p.vco1_wave = w;
                                changed = true;
                            }
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label("VCO 2:");
                        for w in [
                            WaveformKind::Sine,
                            WaveformKind::Saw,
                            WaveformKind::Square,
                            WaveformKind::Triangle,
                            WaveformKind::PulsePwm,
                            WaveformKind::SuperSaw,
                            WaveformKind::WhiteNoise,
                            WaveformKind::SubOctave,
                            WaveformKind::AtmoPad,
                            WaveformKind::Wavetable,
                        ] {
                            if ui.selectable_label(p.vco2_wave == w, w.as_str()).clicked() {
                                p.vco2_wave = w;
                                changed = true;
                            }
                        }
                    });
                    if p.vco1_wave == WaveformKind::Wavetable
                        || p.vco2_wave == WaveformKind::Wavetable
                    {
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Wavetable:");
                            for (name, desc) in synthcraft_core::Wavetable::BUILTINS {
                                if ui
                                    .selectable_label(p.wavetable == name, name)
                                    .on_hover_text(desc)
                                    .clicked()
                                {
                                    p.wavetable = name.to_string();
                                    changed = true;
                                }
                            }
                        });
                    }
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        changed |= rotary_knob(
                            ui,
                            &pal,
                            "VCO2 DETUNE",
                            &mut p.vco2_detune_cents,
                            -50.0,
                            50.0,
                            "ct",
                        );
                        changed |= rotary_knob(ui, &pal, "VCO MIX", &mut p.vco_mix, 0.0, 1.0, "%");
                        changed |=
                            rotary_knob(ui, &pal, "FM INDEX", &mut p.fm_index, 0.0, 4.0, "idx");
                        changed |=
                            rotary_knob(ui, &pal, "PWM DUTY", &mut p.pulse_width, 0.05, 0.95, "%");
                    });
                });

                cols[1].group(|ui| {
                    ui.strong("4-Pole Resonant Ladder VCF & Saturation Rotary Console");
                    ui.horizontal_wrapped(|ui| {
                        for fm in [
                            FilterMode::LowPass24dB,
                            FilterMode::BandPass12dB,
                            FilterMode::HighPass12dB,
                            FilterMode::AcidResonant,
                        ] {
                            if ui
                                .selectable_label(p.filter_mode == fm, fm.as_str())
                                .clicked()
                            {
                                p.filter_mode = fm;
                                changed = true;
                            }
                        }
                    });
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        changed |=
                            rotary_knob(ui, &pal, "CUTOFF", &mut p.cutoff_hz, 40.0, 12000.0, "Hz");
                        changed |=
                            rotary_knob(ui, &pal, "RESON Q", &mut p.resonance_q, 0.5, 12.0, "Q");
                        changed |=
                            rotary_knob(ui, &pal, "ENV MOD", &mut p.env_mod_depth, 0.0, 1.0, "%");
                        changed |=
                            rotary_knob(ui, &pal, "DRIVE", &mut p.drive_saturation, 0.0, 3.0, "x");
                    });
                });
            });
            self.apply_patch_edit(&before, &edited);
        }
        if changed {
            self.persist();
        }
    }

    fn render_sequencer_env(&mut self, ui: &mut egui::Ui) {
        if self.subpage == 2 {
            crate::drums_ui::render_drum_machine(self, ui);
            return;
        }
        let pal = self.skin.palette();
        ui.horizontal_wrapped(|ui| {
            ui.heading("5. 64-Step Acid Sequencer (4 Bars), ADSR Fader Bank & Tempo LFO");
            if ui.button("Open TR-808/909 8-Voice Drum Machine").clicked() {
                self.subpage = 2;
            }
        });
        ui.separator();

        let note_names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let mut changed = false;
        let playing_step = self.playing_step();
        {
            let before = self.engine.active_patch().clone();
            let mut edited = before.clone();
            let p = &mut edited;
            ui.group(|ui| {
                ui.strong("Studio ADSR Envelope Fader Bank & Sinusoidal Filter LFO Rotaries");
                ui.horizontal_wrapped(|ui| {
                    changed |=
                        vertical_fader(ui, &pal, "ATTACK", &mut p.attack_ms, 1.0, 1000.0, "ms");
                    changed |=
                        vertical_fader(ui, &pal, "DECAY", &mut p.decay_ms, 10.0, 2000.0, "ms");
                    changed |=
                        vertical_fader(ui, &pal, "SUSTAIN", &mut p.sustain_level, 0.0, 1.0, "%");
                    changed |=
                        vertical_fader(ui, &pal, "RELEASE", &mut p.release_ms, 10.0, 2000.0, "ms");
                    ui.separator();
                    changed |=
                        rotary_knob(ui, &pal, "LFO RATE", &mut p.lfo_rate_hz, 0.1, 20.0, "Hz");
                    changed |= rotary_knob(ui, &pal, "LFO DEPTH", &mut p.lfo_depth, 0.0, 1.0, "%");
                    changed |= rotary_knob(ui, &pal, "TEMPO", &mut p.bpm, 70.0, 190.0, "BPM");
                    changed |= rotary_knob(ui, &pal, "MASTER", &mut p.master_gain, 0.05, 1.0, "%");
                });
            });
            ui.add_space(6.0);

            ui.group(|ui| {
                // Pattern length selector (16 / 32 / 48 / 64 steps) & Bar A/B/C/D pager
                ui.horizontal_wrapped(|ui| {
                    ui.strong("Pattern Length:");
                    let cur_len = p.steps.len();
                    for target_len in [16_usize, 32, 48, 64] {
                        if ui
                            .selectable_label(cur_len == target_len, format!("{target_len} steps"))
                            .clicked()
                        {
                            if target_len > cur_len {
                                let seed = if cur_len > 0 {
                                    p.steps.clone()
                                } else {
                                    vec![synthcraft_core::SequencerStep {
                                        step_index: 0,
                                        midi_note: 45,
                                        velocity: 0.85,
                                        gate_pct: 0.65,
                                        accent: false,
                                        slide: false,
                                        active: true,
                                    }]
                                };
                                for i in cur_len..target_len {
                                    let mut c = seed[i % seed.len()].clone();
                                    c.step_index = i;
                                    p.steps.push(c);
                                }
                            } else {
                                p.steps.truncate(target_len);
                            }
                            let max_bars = (p.steps.len() / 16).max(1);
                            if self.seq_bar_page >= max_bars {
                                self.seq_bar_page = max_bars - 1;
                            }
                            changed = true;
                        }
                    }

                    ui.separator();
                    ui.strong("Bar:");
                    let num_bars = (p.steps.len() / 16).clamp(1, 4);
                    if self.seq_bar_page >= num_bars {
                        self.seq_bar_page = 0;
                    }
                    for b in 0..num_bars {
                        let letter = ["A", "B", "C", "D"][b];
                        let lo = b * 16 + 1;
                        let hi = lo + 15;
                        let is_playing_bar = playing_step.is_some_and(|ps| {
                            let w = ps % p.steps.len().max(1);
                            w >= b * 16 && w < (b + 1) * 16
                        });
                        let lbl = if is_playing_bar {
                            format!(">Bar {letter} ({lo}-{hi})")
                        } else {
                            format!("Bar {letter} ({lo}-{hi})")
                        };
                        if ui.selectable_label(self.seq_bar_page == b, lbl).clicked() {
                            self.seq_bar_page = b;
                        }
                    }

                    if p.steps.len() > 16 && ui.button("Copy Bar A -> All").clicked() {
                        let bar_a: Vec<_> = p.steps.iter().take(16).cloned().collect();
                        for (i, s) in p.steps.iter_mut().enumerate().skip(16) {
                            let mut c = bar_a[i % 16].clone();
                            c.step_index = i;
                            *s = c;
                        }
                        changed = true;
                    }
                });

                let bar_start = self.seq_bar_page * 16;
                let bar_end = (bar_start + 16).min(p.steps.len());

                ui.strong(format!(
                    "{}-Step Illuminated Hardware Step Pads -- Bar {} (Steps {}..{}) | {:.0} BPM | Key {} -- Click Pad = Gate, ACC = Accent, SLD = 303 Slide",
                    p.steps.len(),
                    ["A", "B", "C", "D"][self.seq_bar_page.min(3)],
                    bar_start + 1,
                    bar_end,
                    p.bpm,
                    p.camelot_key
                ));
                ui.horizontal_wrapped(|ui| {
                    for s in &mut p.steps[bar_start..bar_end] {
                        let semitone = (s.midi_note % 12) as usize;
                        let octave = (s.midi_note / 12) - 1;
                        let n_lbl = if playing_step == Some(s.step_index) {
                            format!(">{}{}", note_names[semitone], octave)
                        } else {
                            format!("{}{}", note_names[semitone], octave)
                        };
                        changed |= illuminated_step_pad(
                            ui,
                            &pal,
                            s.step_index + 1,
                            &n_lbl,
                            &mut s.active,
                            &mut s.accent,
                            &mut s.slide,
                        );
                    }
                });

                if self.subpage == 0 {
                    ui.add_space(6.0);
                    egui::CollapsingHeader::new("Fine MIDI Note & Frequency Step Matrix (Active Bar)")
                        .default_open(false)
                        .show(ui, |ui| {
                            egui::Grid::new("acid_seq_grid")
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.strong("Step");
                                    ui.strong("MIDI Note");
                                    ui.strong("Freq (Hz)");
                                    ui.end_row();
                                    for s in &mut p.steps[bar_start..bar_end] {
                                        ui.monospace(format!("#{:02}", s.step_index + 1));
                                        changed |= ui
                                            .add(
                                                egui::Slider::new(&mut s.midi_note, 36..=84)
                                                    .text(format!("S{}", s.step_index + 1)),
                                            )
                                            .changed();
                                        ui.monospace(format!(
                                            "{:.1} Hz",
                                            midi_to_freq_hz(s.midi_note)
                                        ));
                                        ui.end_row();
                                    }
                                });
                        });
                }
            });
            self.apply_patch_edit(&before, &edited);
        }

        // Arpeggiator, Microtonal Tuning (Scala .scl), and General Modulation Matrix
        ui.add_space(6.0);
        {
            let before = self.engine.active_patch().clone();
            let mut edited = before.clone();

            ui.group(|ui| {
                ui.strong("Arpeggiator & Microtonal Tuning (Scala .scl)");
                ui.horizontal_wrapped(|ui| {
                    egui::ComboBox::from_label("Arp Mode")
                        .width(100.0)
                        .selected_text(edited.arp.mode.label())
                        .show_ui(ui, |ui| {
                            for m in synthcraft_core::ArpMode::ALL {
                                ui.selectable_value(&mut edited.arp.mode, m, m.label());
                            }
                        });
                    egui::ComboBox::from_label("Rate")
                        .width(68.0)
                        .selected_text(match edited.arp.rate_div {
                            1 => "1/4",
                            2 => "1/8",
                            8 => "1/32",
                            _ => "1/16",
                        })
                        .show_ui(ui, |ui| {
                            for (div, lbl) in [(1u8, "1/4"), (2, "1/8"), (4, "1/16"), (8, "1/32")] {
                                ui.selectable_value(&mut edited.arp.rate_div, div, lbl);
                            }
                        });
                    ui.add(egui::Slider::new(&mut edited.arp.octaves, 1..=4).text("Oct"));
                    ui.add(egui::Slider::new(&mut edited.arp.gate, 0.1..=1.0).text("Gate"));
                    ui.checkbox(&mut edited.arp.latch, "Latch");

                    ui.separator();
                    let cur_tuning = edited.tuning.label();
                    egui::ComboBox::from_label("Tuning")
                        .width(175.0)
                        .selected_text(cur_tuning)
                        .show_ui(ui, |ui| {
                            for (id, label) in synthcraft_core::TuningPreset::BUILTINS {
                                if let Some(preset) =
                                    synthcraft_core::TuningPreset::parse_builtin(id)
                                {
                                    if ui
                                        .selectable_label(edited.tuning == preset, label)
                                        .clicked()
                                    {
                                        edited.tuning = preset;
                                    }
                                }
                            }
                        });
                });
            });

            ui.add_space(4.0);
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.strong(format!(
                        "General Modulation Matrix ({}/8 routes)",
                        edited.mod_matrix.len()
                    ));
                    if edited.mod_matrix.len() < 8 && ui.small_button("+ Add Route").clicked() {
                        edited.mod_matrix.push(synthcraft_core::ModRoute {
                            source: synthcraft_core::ModSource::Lfo,
                            dest: synthcraft_core::ModDest::Cutoff,
                            depth: 0.35,
                            enabled: true,
                        });
                    }
                    if !edited.mod_matrix.is_empty() && ui.small_button("Clear Matrix").clicked() {
                        edited.mod_matrix.clear();
                    }
                });
                let mut remove_idx: Option<usize> = None;
                for (i, route) in edited.mod_matrix.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut route.enabled, format!("#{}", i + 1));
                        egui::ComboBox::from_label(format!("Src {}", i + 1))
                            .width(105.0)
                            .selected_text(route.source.label())
                            .show_ui(ui, |ui| {
                                for s in synthcraft_core::ModSource::ALL {
                                    ui.selectable_value(&mut route.source, s, s.label());
                                }
                            });
                        ui.label("->");
                        egui::ComboBox::from_label(format!("Dst {}", i + 1))
                            .width(115.0)
                            .selected_text(route.dest.label())
                            .show_ui(ui, |ui| {
                                for d in synthcraft_core::ModDest::ALL {
                                    ui.selectable_value(&mut route.dest, d, d.label());
                                }
                            });
                        ui.add(egui::Slider::new(&mut route.depth, -1.0..=1.0).text("Depth"));
                        if ui.small_button("X").on_hover_text("Remove route").clicked() {
                            remove_idx = Some(i);
                        }
                    });
                }
                if let Some(i) = remove_idx {
                    edited.mod_matrix.remove(i);
                }
            });

            self.apply_patch_edit(&before, &edited);
        }

        if changed {
            self.persist();
        }
    }

    fn render_scope_spectrum(&mut self, ui: &mut egui::Ui) {
        let green = Color32::from_rgb(34, 197, 94);
        let cyan = Color32::from_rgb(56, 189, 248);

        ui.heading("3. Oscilloscope & 32-Band Spectrum Analyzer");
        ui.small(if self.audio.is_some() {
            "Live: analysing the audio device output (last 4096 samples)."
        } else {
            "Offline: no audio output is running, showing a render of the active patch."
        });
        ui.horizontal(|ui| {
            ui.add(
                egui::Slider::new(&mut self.analysis_duration_ms, 80..=1000)
                    .text("Analysis Window (ms)"),
            );
        });
        ui.separator();

        let rep = match self.live_report() {
            Some(r) => r,
            None => self.cached_analysis(self.analysis_duration_ms),
        };

        ui.horizontal_wrapped(|ui| {
            for (lbl, val) in [
                ("RMS Level", format!("{:.2} dBFS", rep.rms_dbfs)),
                ("Peak Level", format!("{:.2} dBFS", rep.peak_dbfs)),
                ("Crest Factor", format!("{:.2} dB", rep.crest_factor_db)),
                (
                    "Pitch (autocorrelation)",
                    if rep.estimated_pitch_hz > 0.0 {
                        format!("{:.1} Hz", rep.estimated_pitch_hz)
                    } else {
                        "unpitched".to_string()
                    },
                ),
                (
                    "PCM Samples",
                    format!("{} @ {} Hz", rep.sample_count, rep.sample_rate_hz),
                ),
            ] {
                ui.group(|ui| {
                    ui.small(lbl);
                    ui.strong(val);
                });
            }
        });

        ui.add_space(8.0);
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width().max(560.0), 210.0),
            Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        let bg = if self.dark_mode {
            Color32::from_rgb(9, 14, 28)
        } else {
            Color32::from_rgb(241, 245, 249)
        };
        painter.rect_filled(rect, 6.0, bg);

        if self.subpage == 1 {
            // Draw 32-band DFT harmonic spectrum bars
            let bar_w = (rect.width() - 24.0) / (rep.spectrum_bands_db.len().max(1) as f32);
            for (idx, &(_freq, db)) in rep.spectrum_bands_db.iter().enumerate() {
                let norm = ((db + 80.0) / 86.0).clamp(0.04, 1.0);
                let bh = norm * (rect.height() - 28.0);
                let x0 = rect.left() + 12.0 + (idx as f32) * bar_w;
                let bar_rect = Rect::from_min_max(
                    Pos2::new(x0, rect.bottom() - 14.0 - bh),
                    Pos2::new(x0 + (bar_w - 3.0).max(2.0), rect.bottom() - 14.0),
                );
                painter.rect_filled(bar_rect, 2.0, cyan);
            }
        } else {
            // Draw 128-sample time-domain oscilloscope waveform
            let mid_y = rect.center().y;
            let amp_h = (rect.height() * 0.42).max(20.0);
            let step_x = (rect.width() - 24.0) / (rep.scope_samples.len().max(2) as f32);
            for (idx, pair) in rep.scope_samples.windows(2).enumerate() {
                let x0 = rect.left() + 12.0 + (idx as f32) * step_x;
                let x1 = x0 + step_x;
                let y0 = mid_y - pair[0] * amp_h;
                let y1 = mid_y - pair[1] * amp_h;
                painter.line_segment(
                    [Pos2::new(x0, y0), Pos2::new(x1, y1)],
                    Stroke::new(2.0, green),
                );
            }
        }
    }

    fn render_clap_soundcraft(&mut self, ui: &mut egui::Ui) {
        ui.heading("4. CLAP 1.2.2 Plugin Bridge for SoundCraft");
        ui.label(
            "SoundCraft hosts open-source CLAP 1.2.2 plugins (dev.sandraschi.synthcraft) with polyphonic per-note modulation.",
        );
        ui.separator();

        let clap = export_clap_descriptor(self.engine.active_patch());
        if self.subpage == 1 {
            if let Some(params) = clap.get("parameters").and_then(serde_json::Value::as_array) {
                egui::Grid::new("clap_params_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("CLAP Param ID");
                        ui.strong("Name");
                        ui.strong("Module");
                        ui.strong("Range");
                        ui.strong("Current Value");
                        ui.strong("Poly-Modulatable");
                        ui.end_row();
                        for p in params {
                            ui.monospace(p["clap_id"].as_str().unwrap_or(""));
                            ui.label(p["name"].as_str().unwrap_or(""));
                            ui.monospace(p["module"].as_str().unwrap_or(""));
                            ui.monospace(format!(
                                "{:.1} .. {:.1}",
                                p["min_val"].as_f64().unwrap_or(0.0),
                                p["max_val"].as_f64().unwrap_or(1.0)
                            ));
                            ui.monospace(format!(
                                "{:.2}",
                                p["current_val"].as_f64().unwrap_or(0.0)
                            ));
                            ui.monospace(if p["poly_modulatable"].as_bool().unwrap_or(false) {
                                "YES (Per-Note)"
                            } else {
                                "Global"
                            });
                            ui.end_row();
                        }
                    });
                return;
            }
        }

        let pretty = serde_json::to_string_pretty(&clap).unwrap_or_default();
        ui.add(
            egui::TextEdit::multiline(&mut pretty.as_str())
                .hint_text("CLAP 1.2.2 Plugin Manifest JSON")
                .code_editor()
                .desired_width(f32::INFINITY)
                .desired_rows(18),
        );
    }

    fn render_djcraft_forge(&mut self, ui: &mut egui::Ui) {
        ui.heading("5. DJCraft BPM / Camelot Stem & RIFF WAV Forge");
        ui.label(
            "Synthesize BPM-locked, Camelot-keyed acid loops, sub-drops, and transition risers directly to 48 kHz 16-bit PCM .wav files for DJCraft and SoundCraft.",
        );
        ui.separator();

        // Live DJCraft Link & 1-Click Deck Send Bridge
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("Live DJCraft Bridge:");
                if self.djcraft_link_rx.is_connected() {
                    if let Some(b) = self.djcraft_link_rx.last_beacon() {
                        let key_str = if b.camelot_key.is_empty() {
                            "--"
                        } else {
                            b.camelot_key.as_str()
                        };
                        let state = if b.playing { "PLAYING" } else { "PAUSED" };
                        ui.colored_label(
                            egui::Color32::from_rgb(34, 197, 94),
                            format!("LINKED: {:.1} BPM | {} | {}", b.bpm, key_str, state),
                        );
                    }
                } else {
                    ui.label(
                        egui::RichText::new("Waiting for DJCraft Link (UDP 127.0.0.1:20809)")
                            .small(),
                    );
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.checkbox(
                    &mut self.djcraft_link_sync,
                    "Follow DJCraft BPM & Play/Stop",
                );
                ui.checkbox(
                    &mut self.djcraft_auto_camelot,
                    "Auto-Camelot Key-Lock Transpose",
                );
                ui.checkbox(
                    &mut self.djcraft_pad_listen,
                    "Listen for Reloop Ready Pads (DJPD)",
                );
                if ui.small_button("Pull Live BPM/Key").clicked() {
                    match synthcraft_core::query_djcraft_live_status() {
                        Ok(val) => {
                            let bpm = val
                                .pointer("/data/link_bpm")
                                .and_then(serde_json::Value::as_f64)
                                .map(|b| b as f32);
                            let before = self.engine.active_patch().clone();
                            let mut edited = before.clone();
                            if let Some(b) = bpm {
                                edited.bpm = b.clamp(40.0, 240.0);
                            }
                            self.apply_patch_edit(&before, &edited);
                            self.status_line =
                                format!("Synced from live DJCraft ({:.1} BPM)", edited.bpm);
                        }
                        Err(e) => self.status_line = format!("DJCraft IPC: {e}"),
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("Target Deck:");
                for d in ["A", "B", "C", "D"] {
                    if ui
                        .selectable_label(self.djcraft_target_deck == d, format!("Deck {d}"))
                        .clicked()
                    {
                        self.djcraft_target_deck = d.to_string();
                    }
                }
                if ui
                    .button(format!(
                        "1-Click Forge & Load -> DJCraft Deck {}",
                        self.djcraft_target_deck
                    ))
                    .clicked()
                {
                    let p = self.engine.active_patch();
                    let dur_ms = synthcraft_core::stem_duration_ms(p, self.djcraft_bars);
                    let home = std::env::var_os("USERPROFILE")
                        .or_else(|| std::env::var_os("HOME"))
                        .map(PathBuf::from)
                        .unwrap_or_default();
                    let stem_dir = home.join("Documents").join("SynthCraft").join("Stems");
                    let _ = std::fs::create_dir_all(&stem_dir);
                    let safe_key = p.camelot_key.replace(' ', "_");
                    let out_path = stem_dir.join(format!(
                        "{}_{:.0}bpm_{}_{}bars.wav",
                        p.id, p.bpm, safe_key, self.djcraft_bars
                    ));
                    match write_wav_file(p, &out_path, self.engine.sample_rate_hz, dur_ms) {
                        Ok(samples) => {
                            match synthcraft_core::send_stem_to_djcraft_deck(
                                &self.djcraft_target_deck,
                                &out_path,
                            ) {
                                Ok(_) => {
                                    self.status_line = format!(
                                        "Forged {} samples and loaded onto DJCraft Deck {} ({})",
                                        samples,
                                        self.djcraft_target_deck,
                                        out_path.display()
                                    );
                                }
                                Err(e) => {
                                    self.status_line = format!(
                                        "Forged WAV to {} (DJCraft IPC load failed: {e})",
                                        out_path.display()
                                    );
                                }
                            }
                        }
                        Err(e) => self.status_line = format!("Stem render failed: {e}"),
                    }
                }
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label("Live Performance Pads (Key-Locked):");
                const PAD_LABELS: [&str; 8] = [
                    "1: Sub Drop",
                    "2: Root Acid",
                    "3: Minor 3rd",
                    "4: Fifth Stab",
                    "5: Octave Rave",
                    "6: Dub Siren",
                    "7: Riser Sweep",
                    "8: Panic Cut",
                ];
                for (idx, lbl) in PAD_LABELS.iter().enumerate() {
                    if ui.small_button(*lbl).clicked() {
                        self.fire_djcraft_pad(idx as u8, 0.92);
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                let prev_aux = self.djcraft_aux_in_gain;
                ui.add(
                    egui::Slider::new(&mut self.djcraft_aux_in_gain, 0.0..=1.0)
                        .text("Aux FX In from DJCraft (UDP 20810 -> 3U Rack FX)"),
                );
                if (self.djcraft_aux_in_gain - prev_aux).abs() > 1e-4 {
                    if let Some(a) = &self.audio {
                        a.send(crate::audio_io::AudioMsg::AuxInGain(
                            self.djcraft_aux_in_gain,
                        ));
                    }
                }
            });
        });
        ui.add_space(6.0);

        {
            let before = self.engine.active_patch().clone();
            let mut edited = before.clone();
            let p = &mut edited;
            ui.horizontal_wrapped(|ui| {
                ui.add(egui::Slider::new(&mut p.bpm, 70.0..=190.0).text("Deck Tempo (BPM)"));
                ui.label("Camelot Key:");
                for k in ["8A Am", "4A Fm", "2A D#m", "9A Em", "11B A", "5A Cm"] {
                    if ui.selectable_label(p.camelot_key == k, k).clicked() {
                        p.camelot_key = k.into();
                    }
                }
                ui.add(egui::Slider::new(&mut self.djcraft_bars, 1..=16).text("Loop Bars"));
            });
            self.apply_patch_edit(&before, &edited);
        }

        if self.subpage == 1 {
            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("Render 48 kHz 16-Bit RIFF PCM .wav Stem to Disk");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.wav_export_path)
                            .hint_text("Output WAV path e.g. loop_128bpm_8A.wav")
                            .desired_width(300.0),
                    );
                    if ui.button("Render .WAV Stem").clicked() {
                        let out_path = PathBuf::from(&self.wav_export_path);
                        if let Ok(samples) = write_wav_file(
                            self.engine.active_patch(),
                            &out_path,
                            self.engine.sample_rate_hz,
                            1000,
                        ) {
                            self.status_line = format!(
                                "Rendered {} PCM samples to {}",
                                samples,
                                out_path.display()
                            );
                        }
                    }
                });
            });
        }

        ui.add_space(6.0);
        let stem = export_djcraft_stem_manifest(
            self.engine.active_patch(),
            self.djcraft_bars,
            self.engine.sample_rate_hz,
        );
        let pretty = serde_json::to_string_pretty(&stem).unwrap_or_default();
        ui.add(
            egui::TextEdit::multiline(&mut pretty.as_str())
                .hint_text("DJCraft Stem Manifest JSON")
                .code_editor()
                .desired_width(f32::INFINITY)
                .desired_rows(14),
        );
    }

    fn render_chat(&mut self, ui: &mut egui::Ui) {
        ui.heading("SynthCraft Sound-Designer Chat");
        ui.label("no model selected -- connect an MCP 2026-07-28 client or local Ollama endpoint to synthesize patches from natural language.");
        ui.separator();
        for (who, msg) in &self.chat_history {
            ui.horizontal_wrapped(|ui| {
                ui.strong(format!("{who}:"));
                ui.label(msg);
            });
        }
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.chat_input)
                    .hint_text("Ask about acid filters, CLAP 1.2.2 parameters, or DJCraft stems...")
                    .desired_width(420.0),
            );
            if ui.button("Send").clicked() && !self.chat_input.trim().is_empty() {
                let q = self.chat_input.trim().to_string();
                self.chat_input.clear();
                self.chat_history.push(("You".into(), q));
                self.chat_history.push((
                    "SynthCraft".into(),
                    "no model selected -- use the 23 SynthCraft MCP tools (`load_patch`, `update_filter`, `analyze_spectrum`, `export_clap_manifest`, `export_djcraft_stem`, `render_wav_file`) from your agent IDE.".into(),
                ));
            }
        });
    }

    fn render_help_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("SynthCraft Documentation & 37-Subpage Encyclopedia (F1)");
        ui.horizontal_wrapped(|ui| {
            for (tab, label) in [
                (HelpTab::SubpageEncyclopedia, "37-Subpage Encyclopedia"),
                (HelpTab::GetStarted, "Get Started"),
                (HelpTab::ModularRack, "Modular VCO & VCF"),
                (HelpTab::SequencerEnv, "Sequencer & ADSR/LFO"),
                (HelpTab::ClapDjcraft, "CLAP & DJCraft/SoundCraft"),
                (HelpTab::CliMcp, "CLI & 60 MCP Tools"),
                (HelpTab::TechnicalInfo, "Technical Info"),
            ] {
                if ui.selectable_label(self.help_tab == tab, label).clicked() {
                    self.help_tab = tab;
                }
            }
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            crate::help_docs::render_help_tab_content(self, ui);
        });
    }

    fn render_modals(&mut self, ctx: &egui::Context) {
        if self.show_palette {
            let mut open = self.show_palette;
            let mut nav_target: Option<(Page, usize)> = None;
            egui::Window::new("Command Palette")
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.palette_query)
                            .hint_text("Filter commands..."),
                    );
                    for (label, page, sub) in [
                        ("Modular Rack: Dual VCO & Ladder VCF", Page::ModularRack, 0),
                        ("Modular Rack: Factory Patch Bank", Page::ModularRack, 1),
                        ("16-Step Acid Sequencer", Page::SequencerEnv, 0),
                        ("ADSR Envelope & Tempo LFO", Page::SequencerEnv, 1),
                        ("Real-Time Waveform Oscilloscope", Page::ScopeSpectrum, 0),
                        ("32-Band DFT Spectrum & dBFS Meter", Page::ScopeSpectrum, 1),
                        ("CLAP 1.2.2 Plugin Descriptor", Page::ClapSoundcraft, 0),
                        ("SoundCraft CLAP Parameter Map", Page::ClapSoundcraft, 1),
                        ("DJCraft BPM & Camelot Loop Forge", Page::DjcraftForge, 0),
                        ("Render 48 kHz RIFF .WAV Stem", Page::DjcraftForge, 1),
                        ("Help Documentation (F1)", Page::Help, 0),
                    ] {
                        if (self.palette_query.is_empty()
                            || label
                                .to_ascii_lowercase()
                                .contains(&self.palette_query.to_ascii_lowercase()))
                            && ui.button(label).clicked()
                        {
                            nav_target = Some((page, sub));
                        }
                    }
                });
            self.show_palette = open;
            if let Some((page, sub)) = nav_target {
                self.goto(page, sub);
                self.show_palette = false;
            }
        }
        if self.show_shortcuts {
            egui::Window::new("Keyboard Shortcuts")
                .open(&mut self.show_shortcuts)
                .show(ctx, |ui| {
                    ui.label("F1: Open 6-Tab Help");
                    ui.label("Ctrl+K: Toggle Command Palette");
                    ui.label("Ctrl+MouseWheel: Zoom UI (0.75x .. 1.75x)");
                });
        }
        if self.show_licenses {
            egui::Window::new("Licenses & Attribution")
                .open(&mut self.show_licenses)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(ATTRIBUTION_MD);
                    });
                });
        }
        if self.show_about {
            egui::Window::new("About SynthCraft")
                .open(&mut self.show_about)
                .show(ctx, |ui| {
                    ui.heading(format!("SynthCraft v{APP_VERSION}"));
                    ui.label("Pure-Rust Modular DSP Synthesizer, CLAP 1.2.2 Bridge for SoundCraft & Stem Forge for DJCraft.");
                });
        }
    }
}

impl eframe::App for SynthCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.apply_theme(&ctx);
        self.handle_shortcuts(&ctx);
        self.handle_keyboard(&ctx);
        self.pump_live(&ctx);
        self.pump_saves(&ctx);
        self.pump_bank_fetch(&ctx);

        egui::Panel::top("synthcraft_top_menu").show(ui, |ui| {
            self.render_menu_bar(&ctx, ui);
        });
        egui::Panel::bottom("synthcraft_status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "[{}] [{}] {}",
                    self.layout_mode.label(),
                    self.skin.label(),
                    self.status_line
                ));
            });
        });
        if self.layout_mode == SynthLayout::WorkbenchSplit
            || matches!(self.page, Page::Help | Page::Chat | Page::Settings)
        {
            egui::Panel::left("synthcraft_sidebar")
                .default_size(225.0)
                .show(ui, |ui| {
                    self.render_sidebar(ui);
                });
        }
        if self.show_subpage_help && self.page != Page::Help {
            egui::Panel::right("synthcraft_subpage_help")
                .default_size(380.0)
                .show(ui, |ui| {
                    crate::help_docs::render_subpage_help_drawer(self, ui);
                });
        }
        egui::CentralPanel::default().show(ui, |ui| {
            self.render_page(ui);
        });

        crate::mod_wavetable_ui::handle_dropped_wav_files(self, &ctx);
        self.render_modals(&ctx);
    }
}

impl Drop for SynthCraftApp {
    fn drop(&mut self) {
        // Dropping the worker waits for queued saves, so the final write below is the newest.
        self.saver = None;
        let wanted = self.settings.auto_save_sqlite_on_tweak || self.save_requested;
        if self.persist_blocked.is_none() && self.dirty_since.is_some() && wanted {
            let job = crate::saver::SaveJob {
                path: self.db_path.clone(),
                engine: self.engine.clone(),
                settings: self.settings_blobs(),
            };
            let _ = crate::saver::run_job(&job);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    fn temp_app() -> SynthCraftApp {
        let mut p = std::env::temp_dir();
        p.push(format!("synthcraft_app_test_{}.db", std::process::id()));
        SynthCraftApp::with_db_path(p)
    }

    fn tiny_wav(secs: f32) -> Vec<u8> {
        let sr = 22_050u32;
        let n = (secs * sr as f32) as usize;
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + 2 * n as u32).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&sr.to_le_bytes());
        out.extend_from_slice(&(sr * 2).to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&(2 * n as u32).to_le_bytes());
        for i in 0..n {
            let v = ((i as f32 * 0.06).sin() * 12000.0) as i16;
            out.extend_from_slice(&v.to_le_bytes());
        }
        out
    }

    #[test]
    fn finished_bank_download_loads_through_the_command_and_reports_errors() {
        use crate::netfetch::FetchMsg;
        let ctx = egui::Context::default();
        let mut app = temp_app();
        let (tx, rx) = std::sync::mpsc::channel();
        app.bank_fetch_rx = Some(rx);
        let _ = tx.send(FetchMsg::Done(Err("offline".into())));
        app.pump_bank_fetch(&ctx);
        assert!(app.bank_status.contains("Bank not loaded: offline"));
        assert!(app.bank_fetch_rx.is_none());

        let dir = std::env::temp_dir().join(format!("sc_app_bank_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("a.wav"), tiny_wav(0.3));
        let sfz = dir.join("a.sfz");
        let _ = std::fs::write(&sfz, "<region> sample=a.wav pitch_keycenter=60\n");
        let spec = sfz.to_string_lossy().into_owned();
        let (tx, rx) = std::sync::mpsc::channel();
        app.bank_fetch_rx = Some(rx);
        let _ = tx.send(FetchMsg::Done(Ok(spec.clone())));
        app.pump_bank_fetch(&ctx);
        assert_eq!(app.bank_status, "Bank loaded.");
        assert_eq!(
            app.engine.active_patch_mut().sampler.bank.as_deref(),
            Some(spec.as_str())
        );
        assert!(app.engine.active_patch_mut().sampler.mix >= 0.8);

        let (tx, rx) = std::sync::mpsc::channel();
        app.bank_fetch_rx = Some(rx);
        let _ = tx.send(FetchMsg::Done(Ok("Z:\\nowhere\\x.sfz".into())));
        app.pump_bank_fetch(&ctx);
        assert!(
            app.bank_status.contains("Bank not loaded"),
            "{}",
            app.bank_status
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn panel_edits_run_as_engine_commands_and_are_validated() {
        let mut app = temp_app();
        let before = app.engine.active_patch().clone();
        let mut after = before.clone();
        after.cutoff_hz = 2222.0;
        after.bpm = 99.0;
        app.apply_patch_edit(&before, &after);
        assert!((app.engine.active_patch().cutoff_hz - 2222.0).abs() < 1e-3);
        assert!((app.engine.active_patch().bpm - 99.0).abs() < 1e-3);
        // Out-of-range values are clamped by the command, the way an agent would see them.
        let before = app.engine.active_patch().clone();
        let mut after = before.clone();
        after.bpm = 900.0;
        app.apply_patch_edit(&before, &after);
        assert!((app.engine.active_patch().bpm - 240.0).abs() < 1e-3);

        // Undo & Redo round-trip
        app.undo_patch();
        assert!((app.engine.active_patch().bpm - 99.0).abs() < 1e-3);
        app.redo_patch();
        assert!((app.engine.active_patch().bpm - 240.0).abs() < 1e-3);

        // A/B compare snapshots current state and toggles cleanly
        app.toggle_ab_compare(); // stores A (bpm=240), now editing B
        let b_before = app.engine.active_patch().clone();
        let mut b_after = b_before.clone();
        b_after.cutoff_hz = 4444.0;
        app.apply_patch_edit(&b_before, &b_after);
        app.toggle_ab_compare(); // back to A (cutoff=2222)
        assert!((app.engine.active_patch().cutoff_hz - 2222.0).abs() < 1e-3);
        app.toggle_ab_compare(); // back to B (cutoff=4444)
        assert!((app.engine.active_patch().cutoff_hz - 4444.0).abs() < 1e-3);
    }

    #[test]
    fn subapp_registry_exposes_every_workbench() {
        let reg = subapps::registry();
        assert_eq!(reg.len(), 15);
        let total_subpages: usize = reg.iter().map(|w| w.subpages().len()).sum();
        assert_eq!(total_subpages, 39);
    }

    #[test]
    fn headless_egui_kittest_renders_every_workbench_subpage() {
        let mut app = temp_app();
        for wb in subapps::registry() {
            for sp in wb.subpages() {
                app.goto(sp.page, sp.sub);
                let mut harness = Harness::new_ui(|ui| {
                    let ctx = ui.ctx().clone();
                    app.render_menu_bar(&ctx, ui);
                    app.render_sidebar(ui);
                    app.render_page(ui);
                });
                harness.run();
            }
        }
        let _ = std::fs::remove_file(&app.db_path);
    }

    #[test]
    fn headless_egui_kittest_renders_all_six_help_tabs_and_both_themes() {
        let mut app = temp_app();
        app.goto(Page::Help, 0);
        for tab in [
            HelpTab::GetStarted,
            HelpTab::ModularRack,
            HelpTab::SequencerEnv,
            HelpTab::ClapDjcraft,
            HelpTab::CliMcp,
            HelpTab::TechnicalInfo,
        ] {
            app.help_tab = tab;
            app.dark_mode = !app.dark_mode;
            let mut harness = Harness::new_ui(|ui| {
                let ctx = ui.ctx().clone();
                app.apply_theme(&ctx);
                app.render_page(ui);
            });
            harness.run();
        }
        let _ = std::fs::remove_file(&app.db_path);
    }

    #[test]
    fn headless_egui_kittest_renders_all_five_skins_and_four_layouts() {
        let mut app = temp_app();
        app.goto(Page::ModularRack, 0);
        for skin in SynthSkin::all() {
            app.set_skin(*skin);
            for layout in SynthLayout::all() {
                app.layout_mode = *layout;
                let mut harness = Harness::new_ui(|ui| {
                    let ctx = ui.ctx().clone();
                    app.apply_theme(&ctx);
                    app.render_menu_bar(&ctx, ui);
                    app.render_page(ui);
                });
                harness.run();
            }
        }
        let _ = std::fs::remove_file(&app.db_path);
    }
}
