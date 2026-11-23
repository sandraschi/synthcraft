//! SynthCraft L3 Native Desktop Entry Point.
#![forbid(unsafe_code)]

mod app;
mod audio_io;
mod controls;
mod help_docs;
mod live;
mod midi_io;
mod rack_ui;
mod settings_ui;
mod subapps;

use app::SynthCraftApp;
use std::path::PathBuf;
use synthcraft_core::{default_db_path, Command, SynthEngine, APP_VERSION};

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!(
            "SynthCraft Desktop Studio v{APP_VERSION}\n\nUsage: synthcraft-app [OPTIONS]\n\nOptions:\n  --db <PATH>   Override SQLite database path\n  --smoke       Run non-interactive smoke verification and exit 0\n  --audio-check Open the audio device, play a note and verify the callback output, then exit\n  -h, --help    Print help\n  -V, --version Print version"
        );
        return Ok(());
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("synthcraft-app {APP_VERSION}");
        return Ok(());
    }

    let mut db_path: Option<PathBuf> = None;
    let mut idx = 1;
    while idx < args.len() {
        if args[idx] == "--db" && idx + 1 < args.len() {
            db_path = Some(PathBuf::from(&args[idx + 1]));
            idx += 2;
        } else {
            idx += 1;
        }
    }
    let resolved_db = db_path.unwrap_or_else(default_db_path);

    if args.iter().any(|a| a == "--smoke" || a == "--smoke-test") {
        let mut eng = SynthEngine::open_or_seed(&resolved_db).unwrap_or_default();
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
            app.start_audio(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}

/// Headless end-to-end check of the live path: device, callback, engine and meters.
fn audio_check(db: PathBuf) -> eframe::Result<()> {
    use audio_io::{AudioHandle, AudioMsg};
    let eng = SynthEngine::open_or_seed(&db).unwrap_or_default();
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
