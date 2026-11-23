//! SynthCraft L3 Native Desktop Entry Point.
#![forbid(unsafe_code)]

mod app;
mod help_docs;
mod subapps;

use app::SynthCraftApp;
use std::path::PathBuf;
use synthcraft_core::{default_db_path, Command, SynthEngine, APP_VERSION};

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!(
            "SynthCraft Desktop Studio v{APP_VERSION}\n\nUsage: synthcraft-app [OPTIONS]\n\nOptions:\n  --db <PATH>   Override SQLite database path\n  --smoke       Run non-interactive smoke verification and exit 0\n  -h, --help    Print help\n  -V, --version Print version"
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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1240.0, 800.0])
            .with_title(format!("SynthCraft Studio v{APP_VERSION}")),
        ..Default::default()
    };

    eframe::run_native(
        "SynthCraft Studio",
        options,
        Box::new(move |_cc| Ok(Box::new(SynthCraftApp::with_db_path(resolved_db)))),
    )
}
