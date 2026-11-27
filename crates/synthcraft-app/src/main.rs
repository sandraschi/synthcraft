//! SynthCraft L3 Native Desktop Entry Point.
#![forbid(unsafe_code)]

mod app;
mod audio_io;
mod controls;
mod drums_ui;
mod help_docs;
mod live;
mod midi_io;
mod mod_wavetable_ui;
mod netfetch;
mod nextgen_ui;
mod perf_master_ui;
mod pipe_organ_ui;
mod preset_browser_ui;
mod pro_lab_ui;
mod rack_canvas;
mod rack_ui;
mod saver;
mod settings_ui;
mod subapps;
mod visualizers_ui;

use app::{Page, SynthCraftApp};
use controls::SynthSkin;
use std::path::PathBuf;
use synthcraft_core::{default_db_path, Command, SynthEngine, APP_VERSION};

fn parse_page(name: &str) -> Option<Page> {
    match name.to_ascii_lowercase().as_str() {
        "rack" | "modular" | "modular-rack" => Some(Page::ModularRack),
        "m1" | "pcm" | "extreme" => Some(Page::M1ExtremeDsp),
        "midi" | "keyboard" => Some(Page::MidiDepot),
        "demo" | "showcase" => Some(Page::DemoShowcase),
        "sequencer" | "drums" | "seq" => Some(Page::SequencerEnv),
        "scope" | "spectrum" => Some(Page::ScopeSpectrum),
        "clap" | "soundcraft" => Some(Page::ClapSoundcraft),
        "djcraft" | "forge" => Some(Page::DjcraftForge),
        "nextgen" | "nextgen-lab" => Some(Page::NextGenLab),
        "pro" | "pro-lab" => Some(Page::ProLab),
        "visuals" | "mpe" | "ddsp" => Some(Page::VisualsMpeDdsp),
        "perf" | "master" | "vocoder" => Some(Page::PerfMaster),
        "organ" | "pipe-organ" => Some(Page::PipeOrgan),
        "settings" => Some(Page::Settings),
        "help" => Some(Page::Help),
        _ => None,
    }
}

fn parse_skin(name: &str) -> Option<(SynthSkin, bool)> {
    match name.to_ascii_lowercase().as_str() {
        "obsidian" | "vcv" => Some((SynthSkin::ObsidianVcv, true)),
        "vintage" | "workstation" => Some((SynthSkin::WorkstationVintage, true)),
        "silver" | "acid" => Some((SynthSkin::SilverAcid, false)),
        "gold" | "westcoast" => Some((SynthSkin::WestCoastGold, true)),
        "daylight" | "light" => Some((SynthSkin::DaylightStudio, false)),
        _ => None,
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!(
            "SynthCraft Desktop Studio v{APP_VERSION}\n\nUsage: synthcraft-app [OPTIONS]\n\nOptions:\n  --db <PATH>     Override SQLite database path\n  --page <NAME>   Start on a specific workbench (rack, sequencer, organ, nextgen, pro, visuals, perf, ...)\n  --skin <NAME>   Start with a specific skin (obsidian, vintage, silver, gold, daylight)\n  --smoke         Run non-interactive smoke verification and exit 0\n  --audio-check   Open the audio device, play a note and verify the callback output, then exit\n  -h, --help      Print help\n  -V, --version   Print version"
        );
        return Ok(());
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("synthcraft-app {APP_VERSION}");
        return Ok(());
    }

    let mut db_path: Option<PathBuf> = None;
    let mut initial_page: Option<Page> = None;
    let mut initial_skin: Option<(SynthSkin, bool)> = None;
    let mut idx = 1;
    while idx < args.len() {
        if args[idx] == "--db" && idx + 1 < args.len() {
            db_path = Some(PathBuf::from(&args[idx + 1]));
            idx += 2;
        } else if args[idx] == "--page" && idx + 1 < args.len() {
            initial_page = parse_page(&args[idx + 1]);
            idx += 2;
        } else if args[idx] == "--skin" && idx + 1 < args.len() {
            initial_skin = parse_skin(&args[idx + 1]);
            idx += 2;
        } else {
            idx += 1;
        }
    }
    let resolved_db = db_path.unwrap_or_else(default_db_path);

    if args.iter().any(|a| a == "--smoke" || a == "--smoke-test") {
        let mut eng = SynthEngine::open_or_seed(&resolved_db)
            .map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;
        let status = Command::Status
            .dispatch(&mut eng)
            .map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;
        let spec = Command::AnalyzeSpectrum { duration_ms: 120 }
            .dispatch(&mut eng)
            .map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;
        let clap = Command::ExportClapManifest
            .dispatch(&mut eng)
            .map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;
        let _ = (status, spec, clap);
        let _app = SynthCraftApp::with_db_path(resolved_db);
        println!("[synthcraft] smoke ok");
        return Ok(());
    }

    if args.iter().any(|a| a == "--audio-check") {
        return audio_check(resolved_db);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1240.0, 800.0])
            .with_title(format!("SynthCraft Studio v{APP_VERSION}")),
        ..Default::default()
    };

    eframe::run_native(
        "SynthCraft Studio",
        options,
        Box::new(move |cc| {
            let mut app = SynthCraftApp::with_db_path(resolved_db);
            if let Some(page) = initial_page {
                app.page = page;
            }
            if let Some((skin, dark)) = initial_skin {
                app.skin = skin;
                app.dark_mode = dark;
            }
            app.start_audio(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}

/// Headless end-to-end check of the live path: device, callback, engine and meters.
fn audio_check(db: PathBuf) -> eframe::Result<()> {
    use audio_io::{AudioHandle, AudioMsg};
    let eng =
        SynthEngine::open_or_seed(&db).map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;
    let handle = AudioHandle::start(
        None,
        Some(eng.sample_rate_hz),
        Some(256),
        eng.active_patch().clone(),
        synthcraft_core::RtOptions::default(),
    )
    .map_err(|e| eframe::Error::AppCreation(e.into()))?;
    println!(
        "device: {} | {} Hz | {} ch | buffer {:?}",
        handle.info.device,
        handle.info.sample_rate,
        handle.info.channels,
        handle.info.buffer_frames
    );
    handle.send(AudioMsg::NoteOn(57, 0.9));
    std::thread::sleep(std::time::Duration::from_millis(900));
    let recent = handle.meters.recent(4096);
    let rms = (recent.iter().map(|s| s * s).sum::<f32>() / recent.len().max(1) as f32).sqrt();
    println!(
        "voices: {} | callback output rms: {rms:.4}",
        handle.meters.voices()
    );
    handle.send(AudioMsg::NoteOff(57));
    handle.send(AudioMsg::Play(true));
    std::thread::sleep(std::time::Duration::from_millis(1500));
    println!(
        "sequencer playing: {} | step: {}",
        handle.meters.playing(),
        handle.meters.step() + 1
    );
    if rms < 0.001 {
        return Err(eframe::Error::AppCreation(
            "callback produced silence".into(),
        ));
    }
    println!("[synthcraft] audio check ok");
    Ok(())
}
