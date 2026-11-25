//! SynthCraft L2 CLI & MCP `2026-07-28` stdio server.
#![forbid(unsafe_code)]

mod fetch;

use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use synthcraft_core::sandbox::PathPolicy;
use synthcraft_core::{default_db_path, Command, SynthEngine, APP_VERSION};

#[derive(Debug, Parser)]
#[command(
    name = "synthcraft-mcp",
    version,
    about = "SynthCraft Modular DSP Synthesizer, CLAP 1.2.2 Bridge for SoundCraft & Stem Forge for DJCraft"
)]
pub struct Cli {
    /// Emit structured JSON output
    #[arg(long, global = true, default_value_t = false)]
    pub json: bool,

    /// Optional SQLite database file path override
    #[arg(long, global = true)]
    pub db: Option<PathBuf>,

    /// Launch MCP 2026-07-28 JSON-RPC server on stdio
    #[arg(long, global = true, default_value_t = false)]
    pub mcp: bool,

    /// Directory an agent may read from (repeatable). Required for the MCP server.
    #[arg(long = "automation-read-root", global = true)]
    pub automation_read_roots: Vec<PathBuf>,

    /// Directory an agent may write to (repeatable). Required for the MCP server.
    #[arg(long = "automation-write-root", global = true)]
    pub automation_write_roots: Vec<PathBuf>,

    #[command(subcommand)]
    pub subcommand: Option<CliSub>,
}

#[derive(Debug, Subcommand)]
pub enum CliSub {
    /// Run MCP 2026-07-28 JSON-RPC server on stdio
    Mcp,
    /// List the free sample banks that `fetch-bank` can download
    ListBanks,
    /// Download a free sample bank (checksum verified, SFZ generated) into a folder
    FetchBank {
        /// Bank id from `list-banks`
        id: String,
        /// Destination folder (default: Documents/SynthCraft/Banks)
        #[arg(long)]
        dest: Option<PathBuf>,
    },
    /// Show SynthCraft engine status
    Status,
    /// Show full active patch and engine summary
    Summary,
    /// List all factory and user patches
    ListPatches,
    /// Load a patch by ID
    LoadPatch {
        #[arg(long, default_value = "vienna_acid_303")]
        id: String,
    },
    /// Update dual VCO waveforms, detune, mix, and FM index
    UpdateOscillators {
        #[arg(long, default_value = "saw")]
        vco1: String,
        #[arg(long, default_value = "sub_octave")]
        vco2: String,
        #[arg(long, default_value_t = 12.0)]
        detune_cents: f32,
        #[arg(long, default_value_t = 0.35)]
        mix: f32,
        #[arg(long, default_value_t = 0.2)]
        fm_index: f32,
    },
    /// Update 4-pole ladder filter mode, cutoff Hz, resonance Q, and envelope mod depth
    UpdateFilter {
        #[arg(long, default_value = "acid_resonant")]
        mode: String,
        #[arg(long, default_value_t = 850.0)]
        cutoff_hz: f32,
        #[arg(long, default_value_t = 6.5)]
        resonance_q: f32,
        #[arg(long, default_value_t = 0.8)]
        env_mod_depth: f32,
    },
    /// Update ADSR envelope and tempo-synced LFO parameters
    UpdateEnvelopeLfo {
        #[arg(long, default_value_t = 5.0)]
        attack_ms: f32,
        #[arg(long, default_value_t = 200.0)]
        decay_ms: f32,
        #[arg(long, default_value_t = 0.3)]
        sustain_level: f32,
        #[arg(long, default_value_t = 120.0)]
        release_ms: f32,
        #[arg(long, default_value_t = 2.0)]
        lfo_rate_hz: f32,
        #[arg(long, default_value_t = 0.25)]
        lfo_depth: f32,
    },
    /// Configure one of the 16 acid sequencer steps (0..15)
    SetStep {
        #[arg(long)]
        step: usize,
        #[arg(long, default_value_t = 45)]
        midi_note: u8,
        #[arg(long, default_value_t = false)]
        accent: bool,
        #[arg(long, default_value_t = false)]
        slide: bool,
        #[arg(long, default_value_t = true)]
        active: bool,
    },
    /// Synthesize a preview audio buffer and return first 64 PCM samples
    Preview {
        #[arg(long, default_value_t = 250)]
        duration_ms: u32,
    },
    /// Analyze 32-band DFT harmonic spectrum, RMS/Peak dBFS, and pitch Hz
    AnalyzeSpectrum {
        #[arg(long, default_value_t = 300)]
        duration_ms: u32,
    },
    /// Render a 16-bit RIFF PCM .wav file to disk for SoundCraft or DJCraft
    RenderWav {
        #[arg(long)]
        path: String,
        #[arg(long, default_value_t = 2000)]
        duration_ms: u32,
    },
    /// Export CLAP 1.2.2 plugin descriptor and parameter automation map for SoundCraft
    ExportClap,
    /// Export BPM/Camelot stem manifest and cue points for DJCraft
    ExportDjcraftStem {
        #[arg(long, default_value_t = 4)]
        bars: usize,
        /// Also render the mono loop to this WAV file
        #[arg(long)]
        output: Option<String>,
    },
    /// Load a 1990-style PCM sample program (m1_00_universe, m1_01_piano16, m1_17_organ2, ...)
    LoadM1 {
        #[arg(long, default_value = "m1_01_piano16")]
        program: String,
        #[arg(long, default_value_t = 0.65)]
        mix: f32,
    },
    /// Set Extreme Modern DSP mode (karplus_strong_pluck, granular_cloud, wavetable_morph, west_coast_wavefolder, phase_distortion_cz, modal_struck_bar)
    SetExtremeDsp {
        #[arg(long, default_value = "karplus_strong_pluck")]
        mode: String,
        #[arg(long, default_value_t = 0.55)]
        morph: f32,
    },
    /// Load a MIDI pattern from the built-in MIDI Depot
    LoadMidiClip {
        #[arg(long, default_value = "90s_house_m1_piano")]
        clip_id: String,
    },
    /// Export a MIDI Depot clip as a Standard MIDI File (.mid, 480 PPQN)
    ExportMidi {
        #[arg(long, default_value = "90s_house_m1_piano")]
        clip_id: String,
        #[arg(long, default_value = "90s_house_m1_piano.mid")]
        path: String,
    },
    /// Run a 1-click Demo Showcase preset and return spectral telemetry
    RunDemo {
        #[arg(long, default_value = "demo_1990_m1_piano")]
        demo_id: String,
    },
    /// Load a sample bank (.sf2[#preset], .sfz, .wav or a folder of wavs) into the active patch
    SetSamplerBank {
        /// Bank path; omit to unload
        #[arg(long)]
        bank: Option<String>,
        #[arg(long, default_value_t = 1.0)]
        mix: f32,
    },
    /// Set master chorus and reverb of the active patch
    SetFx {
        #[arg(long, default_value_t = 0.0)]
        chorus_mix: f32,
        #[arg(long, default_value_t = 0.0)]
        reverb_mix: f32,
        #[arg(long, default_value_t = 3.0)]
        reverb_decay_s: f32,
        #[arg(long, default_value_t = 0.5)]
        reverb_damping: f32,
    },
    /// Set one scalar patch parameter (bpm, pulse_width, master_gain, drive_saturation, ...)
    SetParam { param: String, value: f32 },
    /// Set the Camelot key label
    SetCamelotKey { key: String },
    /// List connected 3U rack patch cables
    ListRackCables,
    /// Connect a 3U rack patch cable between two module jacks
    ConnectRackCable {
        #[arg(long)]
        from_module: String,
        #[arg(long)]
        from_jack: String,
        #[arg(long)]
        to_module: String,
        #[arg(long)]
        to_jack: String,
        #[arg(long)]
        color: Option<String>,
        #[arg(long)]
        gain: Option<f32>,
    },
    /// Disconnect a 3U rack patch cable by id
    DisconnectRackCable { cable_id: String },
    /// Restore default 3U rack patch cables
    ResetRackCables,
    /// List all placed 3U rack modules
    ListRackModules,
    /// Add a 3U rack module (vco1, vcf_ladder, wavefolder_lpg, sample_hold_slew, euclidean_clock, stereo_delay, attenuverter, inline_scope, ...)
    AddRackModule {
        kind: String,
        #[arg(long)]
        row: Option<usize>,
    },
    /// Remove a 3U rack module by id (also unplugs its cables)
    RemoveRackModule { module_id: String },
    /// Move a 3U rack module left/right or up/down a row
    MoveRackModule {
        module_id: String,
        #[arg(long, default_value_t = 0)]
        delta_row: i32,
        #[arg(long, default_value_t = 0)]
        delta_slot: i32,
    },
    /// Set normalized parameters (0..1) or bypass state on a 3U rack module
    SetRackModuleParam {
        module_id: String,
        #[arg(long)]
        param_a: Option<f32>,
        #[arg(long)]
        param_b: Option<f32>,
        #[arg(long)]
        bypassed: Option<bool>,
    },
    /// Restore default 2-row 3U rack layout and cables
    ResetRackLayout,
    /// List all 12 curated pre-patched 3U modular rack rigs
    ListRackPresets,
    /// Load one of the 12 curated pre-patched 3U modular rack rigs by id
    LoadRackPreset { preset_id: String },
    /// Reset all patches to factory defaults
    ResetFactory,
}

fn default_banks_dir() -> PathBuf {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join("Documents").join("SynthCraft").join("Banks")
}

fn run_list_banks(json: bool) -> i32 {
    let m = match synthcraft_core::banks::manifest() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };
    if json {
        let v: Vec<Value> = m
            .banks
            .iter()
            .map(|b| json!({"id": b.id, "name": b.name, "license": b.license, "files": b.files.len(), "bytes": b.files.iter().map(|f| f.bytes).sum::<u64>()}))
            .collect();
        println!("{}", json!({"banks": v}));
    } else {
        for b in &m.banks {
            let mb = b.files.iter().map(|f| f.bytes).sum::<u64>() / 1_000_000;
            println!("{}  {} MB  {}  {}", b.id, mb, b.license, b.name);
        }
    }
    0
}

fn run_fetch_bank(id: &str, dest: Option<&Path>, policy: &PathPolicy) -> i32 {
    let m = match synthcraft_core::banks::manifest() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };
    let Some(bank) = m.banks.iter().find(|b| b.id == id) else {
        eprintln!("error: unknown bank {id:?}; run `list-banks`");
        return 2;
    };
    let dest = dest
        .map(Path::to_path_buf)
        .unwrap_or_else(default_banks_dir);
    let dest = if policy.is_active() {
        match policy.check_write(&dest.to_string_lossy()) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: {e}");
                return 2;
            }
        }
    } else {
        dest
    };
    match synthcraft_core::banks::fetch_bank(bank, &dest, &fetch::http_get, &mut |l| {
        eprintln!("{l}")
    }) {
        Ok(sfz) => {
            println!("{}", sfz.display());
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(default_db_path);
    let mut engine = match SynthEngine::open_or_seed(&db_path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: cannot open database {}: {e}", db_path.display());
            std::process::exit(1);
        }
    };
    for w in &engine.load_warnings {
        eprintln!("warning: {w}");
    }
    let policy = match PathPolicy::new(&cli.automation_read_roots, &cli.automation_write_roots) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };

    if cli.mcp || matches!(cli.subcommand, Some(CliSub::Mcp)) {
        if !policy.is_complete() {
            eprintln!(
                "error: the MCP server refuses to start without --automation-read-root and --automation-write-root (AGENTS rule 18). It can only read and write below those folders."
            );
            std::process::exit(2);
        }
        run_mcp_stdio(&mut engine, &db_path, &policy);
        return;
    }

    let sub = cli.subcommand.unwrap_or(CliSub::Status);
    match &sub {
        CliSub::ListBanks => {
            std::process::exit(run_list_banks(cli.json));
        }
        CliSub::FetchBank { id, dest } => {
            std::process::exit(run_fetch_bank(id, dest.as_deref(), &policy));
        }
        _ => {}
    }
    let cmd = match sub {
        CliSub::ListBanks | CliSub::FetchBank { .. } => Command::Status,
        CliSub::Mcp | CliSub::Status => Command::Status,
        CliSub::Summary => Command::Summary,
        CliSub::ListPatches => Command::ListPatches,
        CliSub::LoadPatch { id } => Command::LoadPatch { patch_id: id },
        CliSub::UpdateOscillators {
            vco1,
            vco2,
            detune_cents,
            mix,
            fm_index,
        } => Command::UpdateOscillators {
            vco1_wave: vco1,
            vco2_wave: vco2,
            detune_cents,
            mix,
            fm_index,
        },
        CliSub::UpdateFilter {
            mode,
            cutoff_hz,
            resonance_q,
            env_mod_depth,
        } => Command::UpdateFilter {
            mode,
            cutoff_hz,
            resonance_q,
            env_mod_depth,
        },
        CliSub::UpdateEnvelopeLfo {
            attack_ms,
            decay_ms,
            sustain_level,
            release_ms,
            lfo_rate_hz,
            lfo_depth,
        } => Command::UpdateEnvelopeLfo {
            attack_ms,
            decay_ms,
            sustain_level,
            release_ms,
            lfo_rate_hz,
            lfo_depth,
        },
        CliSub::SetStep {
            step,
            midi_note,
            accent,
            slide,
            active,
        } => Command::SetSequencerStep {
            step_index: step,
            midi_note,
            accent,
            slide,
            active,
        },
        CliSub::Preview { duration_ms } => Command::SynthesizePreview { duration_ms },
        CliSub::AnalyzeSpectrum { duration_ms } => Command::AnalyzeSpectrum { duration_ms },
        CliSub::RenderWav { path, duration_ms } => Command::RenderWavFile {
            output_path: path,
            duration_ms,
        },
        CliSub::ExportClap => Command::ExportClapManifest,
        CliSub::ExportDjcraftStem { bars, output } => Command::ExportDjcraftStem {
            bars,
            output_path: output,
        },
        CliSub::LoadM1 { program, mix } => Command::LoadPcmProgram { program, mix },
        CliSub::SetExtremeDsp { mode, morph } => Command::SetExtremeDsp { mode, morph },
        CliSub::LoadMidiClip { clip_id } => Command::LoadMidiDepotClip { clip_id },
        CliSub::ExportMidi { clip_id, path } => Command::ExportMidiFile {
            clip_id,
            output_path: path,
        },
        CliSub::RunDemo { demo_id } => Command::RunDemoShowcase { demo_id },
        CliSub::SetSamplerBank { bank, mix } => Command::SetSamplerBank { bank, mix },
        CliSub::SetFx {
            chorus_mix,
            reverb_mix,
            reverb_decay_s,
            reverb_damping,
        } => Command::SetFx {
            chorus_mix,
            reverb_mix,
            reverb_decay_s,
            reverb_damping,
        },
        CliSub::SetParam { param, value } => Command::SetPatchParam { param, value },
        CliSub::SetCamelotKey { key } => Command::SetCamelotKey { key },
        CliSub::ListRackCables => Command::ListRackCables,
        CliSub::ConnectRackCable {
            from_module,
            from_jack,
            to_module,
            to_jack,
            color,
            gain,
        } => Command::ConnectRackCable {
            from_module,
            from_jack,
            to_module,
            to_jack,
            color_name: color,
            gain,
        },
        CliSub::DisconnectRackCable { cable_id } => Command::DisconnectRackCable { cable_id },
        CliSub::ResetRackCables => Command::ResetRackCables,
        CliSub::ListRackModules => Command::ListRackModules,
        CliSub::AddRackModule { kind, row } => Command::AddRackModule { kind, row },
        CliSub::RemoveRackModule { module_id } => Command::RemoveRackModule { module_id },
        CliSub::MoveRackModule {
            module_id,
            delta_row,
            delta_slot,
        } => Command::MoveRackModule {
            module_id,
            delta_row,
            delta_slot,
        },
        CliSub::SetRackModuleParam {
            module_id,
            param_a,
            param_b,
            bypassed,
        } => Command::SetRackModuleParam {
            module_id,
            param_a,
            param_b,
            bypassed,
        },
        CliSub::ResetRackLayout => Command::ResetRackLayout,
        CliSub::ListRackPresets => Command::ListRackPresets,
        CliSub::LoadRackPreset { preset_id } => Command::LoadRackPreset { preset_id },
        CliSub::ResetFactory => Command::ResetFactoryPatches,
    };
    let cmd = if policy.is_active() {
        match enforce_command(cmd, &policy) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    } else {
        cmd
    };

    let before = serde_json::to_string(&engine).unwrap_or_default();
    match cmd.dispatch(&mut engine) {
        Ok(val) => {
            if let Err(e) = save_if_changed(&engine, &before, &db_path) {
                eprintln!("error: command succeeded but saving failed: {e}");
                std::process::exit(1);
            }
            let out = serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string());
            println!("{out}");
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

/// Save only when the command changed something, so read-only calls never write and a
/// long-lived server does not overwrite edits made elsewhere with a stale snapshot.
fn save_if_changed(
    engine: &SynthEngine,
    before: &str,
    db_path: &Path,
) -> Result<(), synthcraft_core::SynthError> {
    let after = serde_json::to_string(engine).unwrap_or_default();
    if after == before {
        return Ok(());
    }
    engine.save_db(db_path)
}

/// Rewrite the file paths of a command to their resolved form, or refuse the command when a
/// path is outside the allowed roots.
fn enforce_command(
    cmd: Command,
    policy: &PathPolicy,
) -> Result<Command, synthcraft_core::SynthError> {
    Ok(match cmd {
        Command::RenderWavFile {
            output_path,
            duration_ms,
        } => Command::RenderWavFile {
            output_path: policy
                .check_write(&output_path)?
                .to_string_lossy()
                .into_owned(),
            duration_ms,
        },
        Command::ExportDjcraftStem { bars, output_path } => Command::ExportDjcraftStem {
            bars,
            output_path: match output_path {
                Some(p) => Some(policy.check_write(&p)?.to_string_lossy().into_owned()),
                None => None,
            },
        },
        Command::ExportMidiFile {
            clip_id,
            output_path,
        } => Command::ExportMidiFile {
            clip_id,
            output_path: policy
                .check_write(&output_path)?
                .to_string_lossy()
                .into_owned(),
        },
        Command::SetSamplerBank {
            bank: Some(spec),
            mix,
        } => {
            // A `#preset` suffix selects an SF2 preset and is not part of the path.
            let (path, suffix) = match spec.rsplit_once('#') {
                Some((p, n)) if n.trim().parse::<usize>().is_ok() => {
                    (p.to_string(), format!("#{}", n.trim()))
                }
                _ => (spec.clone(), String::new()),
            };
            let resolved = policy.check_read(&path)?;
            Command::SetSamplerBank {
                bank: Some(format!("{}{suffix}", resolved.to_string_lossy())),
                mix,
            }
        }
        other => other,
    })
}

/// Turn one MCP tool call into a `Command` and apply the path policy to it.
fn tool_command(name: &str, args: &Value) -> Option<Command> {
    let f = |k: &str, d: f64| args.get(k).and_then(Value::as_f64).unwrap_or(d) as f32;
    match name {
        "set_sampler_bank" => Some(Command::SetSamplerBank {
            bank: args
                .get("bank")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            mix: f("mix", 1.0),
        }),
        "set_fx" => Some(Command::SetFx {
            chorus_mix: f("chorus_mix", 0.0),
            reverb_mix: f("reverb_mix", 0.0),
            reverb_decay_s: f("reverb_decay_s", 3.0),
            reverb_damping: f("reverb_damping", 0.5),
        }),
        _ => None,
    }
}

pub fn handle_mcp_request(
    req: &Value,
    engine: &mut SynthEngine,
    db_path: &Path,
    policy: &PathPolicy,
) -> Option<Value> {
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let id = req.get("id").cloned();

    if method.starts_with("notifications/") {
        return None;
    }
    let req_id = id?;

    match method {
        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": "2026-07-28",
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": {
                    "name": "synthcraft-mcp",
                    "version": APP_VERSION
                }
            }
        })),
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "tools": mcp_tools_schema()
            }
        })),
        "tools/call" => {
            let params = req.get("params").cloned().unwrap_or_else(|| json!({}));
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));

            let before = serde_json::to_string(&*engine).unwrap_or_default();
            let cmd_res = match name {
                "set_sampler_bank" | "set_fx" => match tool_command(name, &args) {
                    Some(cmd) => enforce_command(cmd, policy).and_then(|c| c.dispatch(engine)),
                    None => Err(synthcraft_core::SynthError::Validation(format!(
                        "Unknown tool '{name}'"
                    ))),
                },
                "synth_status" => Command::Status.dispatch(engine),
                "synth_summary" => Command::Summary.dispatch(engine),
                "list_patches" => Command::ListPatches.dispatch(engine),
                "load_patch" => {
                    let patch_id = args
                        .get("patch_id")
                        .and_then(Value::as_str)
                        .unwrap_or("vienna_acid_303")
                        .to_string();
                    Command::LoadPatch { patch_id }.dispatch(engine)
                }
                "update_oscillators" => {
                    let vco1_wave = args
                        .get("vco1_wave")
                        .and_then(Value::as_str)
                        .unwrap_or("saw")
                        .to_string();
                    let vco2_wave = args
                        .get("vco2_wave")
                        .and_then(Value::as_str)
                        .unwrap_or("sub_octave")
                        .to_string();
                    let detune_cents = args
                        .get("detune_cents")
                        .and_then(Value::as_f64)
                        .unwrap_or(12.0) as f32;
                    let mix = args.get("mix").and_then(Value::as_f64).unwrap_or(0.35) as f32;
                    let fm_index =
                        args.get("fm_index").and_then(Value::as_f64).unwrap_or(0.2) as f32;
                    Command::UpdateOscillators {
                        vco1_wave,
                        vco2_wave,
                        detune_cents,
                        mix,
                        fm_index,
                    }
                    .dispatch(engine)
                }
                "update_filter" => {
                    let mode = args
                        .get("mode")
                        .and_then(Value::as_str)
                        .unwrap_or("acid_resonant")
                        .to_string();
                    let cutoff_hz = args
                        .get("cutoff_hz")
                        .and_then(Value::as_f64)
                        .unwrap_or(850.0) as f32;
                    let resonance_q = args
                        .get("resonance_q")
                        .and_then(Value::as_f64)
                        .unwrap_or(6.5) as f32;
                    let env_mod_depth = args
                        .get("env_mod_depth")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.8) as f32;
                    Command::UpdateFilter {
                        mode,
                        cutoff_hz,
                        resonance_q,
                        env_mod_depth,
                    }
                    .dispatch(engine)
                }
                "update_envelope_lfo" => {
                    let attack_ms =
                        args.get("attack_ms").and_then(Value::as_f64).unwrap_or(5.0) as f32;
                    let decay_ms = args
                        .get("decay_ms")
                        .and_then(Value::as_f64)
                        .unwrap_or(200.0) as f32;
                    let sustain_level = args
                        .get("sustain_level")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.3) as f32;
                    let release_ms = args
                        .get("release_ms")
                        .and_then(Value::as_f64)
                        .unwrap_or(120.0) as f32;
                    let lfo_rate_hz = args
                        .get("lfo_rate_hz")
                        .and_then(Value::as_f64)
                        .unwrap_or(2.0) as f32;
                    let lfo_depth = args
                        .get("lfo_depth")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.25) as f32;
                    Command::UpdateEnvelopeLfo {
                        attack_ms,
                        decay_ms,
                        sustain_level,
                        release_ms,
                        lfo_rate_hz,
                        lfo_depth,
                    }
                    .dispatch(engine)
                }
                "set_sequencer_step" => {
                    let step_index =
                        args.get("step_index").and_then(Value::as_u64).unwrap_or(0) as usize;
                    let midi_note =
                        args.get("midi_note").and_then(Value::as_u64).unwrap_or(45) as u8;
                    let accent = args.get("accent").and_then(Value::as_bool).unwrap_or(false);
                    let slide = args.get("slide").and_then(Value::as_bool).unwrap_or(false);
                    let active = args.get("active").and_then(Value::as_bool).unwrap_or(true);
                    Command::SetSequencerStep {
                        step_index,
                        midi_note,
                        accent,
                        slide,
                        active,
                    }
                    .dispatch(engine)
                }
                "synthesize_preview" => {
                    let duration_ms = args
                        .get("duration_ms")
                        .and_then(Value::as_u64)
                        .unwrap_or(250) as u32;
                    Command::SynthesizePreview { duration_ms }.dispatch(engine)
                }
                "analyze_spectrum" => {
                    let duration_ms = args
                        .get("duration_ms")
                        .and_then(Value::as_u64)
                        .unwrap_or(300) as u32;
                    Command::AnalyzeSpectrum { duration_ms }.dispatch(engine)
                }
                "render_wav_file" => {
                    let output_path = args
                        .get("output_path")
                        .and_then(Value::as_str)
                        .unwrap_or("synthcraft_out.wav")
                        .to_string();
                    let duration_ms = args
                        .get("duration_ms")
                        .and_then(Value::as_u64)
                        .unwrap_or(2000) as u32;
                    enforce_command(
                        Command::RenderWavFile {
                            output_path,
                            duration_ms,
                        },
                        policy,
                    )
                    .and_then(|c| c.dispatch(engine))
                }
                "export_clap_manifest" => Command::ExportClapManifest.dispatch(engine),
                "export_djcraft_stem" => {
                    let bars = args.get("bars").and_then(Value::as_u64).unwrap_or(4) as usize;
                    let output_path = args
                        .get("output_path")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    enforce_command(Command::ExportDjcraftStem { bars, output_path }, policy)
                        .and_then(|c| c.dispatch(engine))
                }
                "load_pcm_program" => {
                    let program = args
                        .get("program")
                        .and_then(Value::as_str)
                        .unwrap_or("m1_01_piano16")
                        .to_string();
                    let mix = args.get("mix").and_then(Value::as_f64).unwrap_or(0.65) as f32;
                    Command::LoadPcmProgram { program, mix }.dispatch(engine)
                }
                "set_extreme_dsp" => {
                    let mode = args
                        .get("mode")
                        .and_then(Value::as_str)
                        .unwrap_or("karplus_strong_pluck")
                        .to_string();
                    let morph = args.get("morph").and_then(Value::as_f64).unwrap_or(0.55) as f32;
                    Command::SetExtremeDsp { mode, morph }.dispatch(engine)
                }
                "load_midi_depot_clip" => {
                    let clip_id = args
                        .get("clip_id")
                        .and_then(Value::as_str)
                        .unwrap_or("90s_house_m1_piano")
                        .to_string();
                    Command::LoadMidiDepotClip { clip_id }.dispatch(engine)
                }
                "export_midi_file" => {
                    let clip_id = args
                        .get("clip_id")
                        .and_then(Value::as_str)
                        .unwrap_or("90s_house_m1_piano")
                        .to_string();
                    let output_path = args
                        .get("output_path")
                        .and_then(Value::as_str)
                        .unwrap_or("synthcraft_clip.mid")
                        .to_string();
                    enforce_command(
                        Command::ExportMidiFile {
                            clip_id,
                            output_path,
                        },
                        policy,
                    )
                    .and_then(|c| c.dispatch(engine))
                }
                "run_demo_showcase" => {
                    let demo_id = args
                        .get("demo_id")
                        .and_then(Value::as_str)
                        .unwrap_or("demo_1990_m1_piano")
                        .to_string();
                    Command::RunDemoShowcase { demo_id }.dispatch(engine)
                }
                "set_patch_param" => {
                    let param = args
                        .get("param")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let value = args.get("value").and_then(Value::as_f64).unwrap_or(0.0) as f32;
                    Command::SetPatchParam { param, value }.dispatch(engine)
                }
                "set_camelot_key" => {
                    let key = args
                        .get("key")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    Command::SetCamelotKey { key }.dispatch(engine)
                }
                "list_rack_cables" => Command::ListRackCables.dispatch(engine),
                "connect_rack_cable" => Command::ConnectRackCable {
                    from_module: args
                        .get("from_module")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    from_jack: args
                        .get("from_jack")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    to_module: args
                        .get("to_module")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    to_jack: args
                        .get("to_jack")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    color_name: args
                        .get("color_name")
                        .and_then(Value::as_str)
                        .map(String::from),
                    gain: args.get("gain").and_then(Value::as_f64).map(|v| v as f32),
                }
                .dispatch(engine),
                "disconnect_rack_cable" => Command::DisconnectRackCable {
                    cable_id: args
                        .get("cable_id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                }
                .dispatch(engine),
                "reset_rack_cables" => Command::ResetRackCables.dispatch(engine),
                "list_rack_modules" => Command::ListRackModules.dispatch(engine),
                "add_rack_module" => Command::AddRackModule {
                    kind: args
                        .get("kind")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    row: args.get("row").and_then(Value::as_u64).map(|v| v as usize),
                }
                .dispatch(engine),
                "remove_rack_module" => Command::RemoveRackModule {
                    module_id: args
                        .get("module_id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                }
                .dispatch(engine),
                "move_rack_module" => Command::MoveRackModule {
                    module_id: args
                        .get("module_id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    delta_row: args.get("delta_row").and_then(Value::as_i64).unwrap_or(0) as i32,
                    delta_slot: args.get("delta_slot").and_then(Value::as_i64).unwrap_or(0) as i32,
                }
                .dispatch(engine),
                "set_rack_module_param" => Command::SetRackModuleParam {
                    module_id: args
                        .get("module_id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    param_a: args
                        .get("param_a")
                        .and_then(Value::as_f64)
                        .map(|v| v as f32),
                    param_b: args
                        .get("param_b")
                        .and_then(Value::as_f64)
                        .map(|v| v as f32),
                    bypassed: args.get("bypassed").and_then(Value::as_bool),
                }
                .dispatch(engine),
                "reset_rack_layout" => Command::ResetRackLayout.dispatch(engine),
                "list_rack_presets" => Command::ListRackPresets.dispatch(engine),
                "load_rack_preset" => Command::LoadRackPreset {
                    preset_id: args
                        .get("preset_id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                }
                .dispatch(engine),
                "reset_factory_patches" => Command::ResetFactoryPatches.dispatch(engine),
                other => Err(synthcraft_core::SynthError::Validation(format!(
                    "Unknown tool '{other}'"
                ))),
            };

            match cmd_res {
                Ok(val) => {
                    if let Err(e) = save_if_changed(engine, &before, db_path) {
                        return Some(json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "result": {
                                "content": [{ "type": "text", "text": format!("saving failed: {e}") }],
                                "isError": true
                            }
                        }));
                    }
                    let text = serde_json::to_string_pretty(&val).unwrap_or_default();
                    Some(json!({
                        "jsonrpc": "2.0",
                        "id": req_id,
                        "result": {
                            "content": [{ "type": "text", "text": text }],
                            "isError": false
                        }
                    }))
                }
                Err(e) => Some(json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": {
                        "content": [{ "type": "text", "text": e.to_string() }],
                        "isError": true
                    }
                })),
            }
        }
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "error": { "code": -32601, "message": format!("Method not found: {method}") }
        })),
    }
}

fn mcp_tools_schema() -> Value {
    json!([
        { "name": "synth_status", "description": "Get SynthCraft status, active patch, BPM, Camelot key, sample rate, and CLAP plugin ID.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "synth_summary", "description": "Return full active patch parameters and 16-step sequencer state.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "list_patches", "description": "List all 6 factory and user patches in the SynthCraft library.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "load_patch", "description": "Select and activate a patch by ID (e.g. vienna_acid_303, goliath_reese_sub, danube_dub_chord, stephansdom_bell_fm, djcraft_riser_sweep, soundcraft_warm_pad).", "inputSchema": { "type": "object", "properties": { "patch_id": { "type": "string" } }, "required": ["patch_id"] } },
        { "name": "update_oscillators", "description": "Update VCO1 and VCO2 waveforms (sine, saw, square, triangle, pulse_pwm, supersaw, white_noise, sub_octave), detune cents, mix, and FM index.", "inputSchema": { "type": "object", "properties": { "vco1_wave": { "type": "string" }, "vco2_wave": { "type": "string" }, "detune_cents": { "type": "number" }, "mix": { "type": "number" }, "fm_index": { "type": "number" } }, "required": ["vco1_wave", "vco2_wave"] } },
        { "name": "update_filter", "description": "Configure the 4-pole ladder filter mode (lowpass_24db, bandpass_12db, highpass_12db, acid_resonant), cutoff_hz, resonance_q, and env_mod_depth.", "inputSchema": { "type": "object", "properties": { "mode": { "type": "string" }, "cutoff_hz": { "type": "number" }, "resonance_q": { "type": "number" }, "env_mod_depth": { "type": "number" } }, "required": ["mode", "cutoff_hz", "resonance_q"] } },
        { "name": "update_envelope_lfo", "description": "Update ADSR envelope (attack_ms, decay_ms, sustain_level, release_ms) and LFO (lfo_rate_hz, lfo_depth).", "inputSchema": { "type": "object", "properties": { "attack_ms": { "type": "number" }, "decay_ms": { "type": "number" }, "sustain_level": { "type": "number" }, "release_ms": { "type": "number" }, "lfo_rate_hz": { "type": "number" }, "lfo_depth": { "type": "number" } } } },
        { "name": "set_sequencer_step", "description": "Update a step (0..15) in the 16-step acid sequencer (midi_note, accent, slide, active).", "inputSchema": { "type": "object", "properties": { "step_index": { "type": "integer" }, "midi_note": { "type": "integer" }, "accent": { "type": "boolean" }, "slide": { "type": "boolean" }, "active": { "type": "boolean" } }, "required": ["step_index", "midi_note"] } },
        { "name": "synthesize_preview", "description": "Synthesize audio for duration_ms and return sample metadata plus the first 64 PCM samples.", "inputSchema": { "type": "object", "properties": { "duration_ms": { "type": "integer" } } } },
        { "name": "analyze_spectrum", "description": "Compute 32-band DFT harmonic spectrum, 128-sample oscilloscope buffer, RMS/Peak dBFS, crest factor, and fundamental pitch Hz.", "inputSchema": { "type": "object", "properties": { "duration_ms": { "type": "integer" } } } },
        { "name": "render_wav_file", "description": "Render the active patch to a 16-bit RIFF PCM .wav file on disk for SoundCraft or DJCraft.", "inputSchema": { "type": "object", "properties": { "output_path": { "type": "string" }, "duration_ms": { "type": "integer" } }, "required": ["output_path"] } },
        { "name": "export_clap_manifest", "description": "Export CLAP 1.2.2 plugin descriptor (dev.sandraschi.synthcraft) and polyphonic parameter automation table for SoundCraft.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "export_djcraft_stem", "description": "Export BPM-locked, Camelot-keyed loop/stem metadata and bar-grid cue points for DJCraft. With output_path (inside the write root) the mono loop is also rendered to a WAV at the engine sample rate.", "inputSchema": { "type": "object", "properties": { "bars": { "type": "integer" }, "output_path": { "type": "string" } } } },
        { "name": "load_pcm_program", "description": "Layer a 1990-style PCM sample program (m1_00_universe, m1_01_piano16, m1_11_fretless, m1_12_slapbass, m1_17_organ2, m1_24_panflute, m1_36_choir, m1_48_orchhit, or off).", "inputSchema": { "type": "object", "properties": { "program": { "type": "string" }, "mix": { "type": "number" } }, "required": ["program"] } },
        { "name": "set_extreme_dsp", "description": "Activate an Extreme Modern DSP generator (off, karplus_strong_pluck, granular_cloud, wavetable_morph, west_coast_wavefolder, phase_distortion_cz, modal_struck_bar).", "inputSchema": { "type": "object", "properties": { "mode": { "type": "string" }, "morph": { "type": "number" } }, "required": ["mode"] } },
        { "name": "set_sampler_bank", "description": "Load a sample bank into the active patch: an .sf2 (add #N for preset N), .sfz, .wav or a folder of wavs named by note. Must be inside the read root. Omit bank to unload.", "inputSchema": { "type": "object", "properties": { "bank": { "type": "string" }, "mix": { "type": "number" } } } },
        { "name": "set_fx", "description": "Set the master chorus and reverb of the active patch (chorus_mix, reverb_mix 0..1, reverb_decay_s 0.2..20, reverb_damping 0..0.95).", "inputSchema": { "type": "object", "properties": { "chorus_mix": { "type": "number" }, "reverb_mix": { "type": "number" }, "reverb_decay_s": { "type": "number" }, "reverb_damping": { "type": "number" } } } },
        { "name": "load_midi_depot_clip", "description": "Load one of the 8 built-in MIDI Depot patterns into the 16-step sequencer.", "inputSchema": { "type": "object", "properties": { "clip_id": { "type": "string" } }, "required": ["clip_id"] } },
        { "name": "export_midi_file", "description": "Export a MIDI Depot clip as a Standard MIDI File (.mid, SMF Format 0, 480 PPQN).", "inputSchema": { "type": "object", "properties": { "clip_id": { "type": "string" }, "output_path": { "type": "string" } }, "required": ["clip_id", "output_path"] } },
        { "name": "run_demo_showcase", "description": "Configure and audition one of the 6 curated 1-click Demo Showcases (demo_1990_m1_piano, demo_club_organ_bass, demo_m1_universe_granular, demo_buchla_wavefolder, demo_karplus_panflute, demo_modal_glass_choir).", "inputSchema": { "type": "object", "properties": { "demo_id": { "type": "string" } }, "required": ["demo_id"] } },
        { "name": "set_patch_param", "description": "Set one scalar parameter of the active patch: bpm, pulse_width, master_gain, drive_saturation, m1_layer_mix, extreme_morph, stepN.velocity or stepN.gate_pct (N = 0..15). Values are clamped to the parameter range.", "inputSchema": { "type": "object", "properties": { "param": { "type": "string" }, "value": { "type": "number" } }, "required": ["param", "value"] } },
        { "name": "set_camelot_key", "description": "Set the Camelot key label of the active patch, for example 8A Am.", "inputSchema": { "type": "object", "properties": { "key": { "type": "string" } }, "required": ["key"] } },
        { "name": "list_rack_cables", "description": "List all connected 3U Eurorack patch cables.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "connect_rack_cable", "description": "Connect a 3U Eurorack patch cable between an output jack and an input jack.", "inputSchema": { "type": "object", "properties": { "from_module": { "type": "string" }, "from_jack": { "type": "string" }, "to_module": { "type": "string" }, "to_jack": { "type": "string" }, "color_name": { "type": "string" }, "gain": { "type": "number" } }, "required": ["from_module", "from_jack", "to_module", "to_jack"] } },
        { "name": "disconnect_rack_cable", "description": "Disconnect a 3U Eurorack patch cable by its cable_id.", "inputSchema": { "type": "object", "properties": { "cable_id": { "type": "string" } }, "required": ["cable_id"] } },
        { "name": "reset_rack_cables", "description": "Restore the default 3U Eurorack patch cables.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "list_rack_modules", "description": "List all placed 3U Eurorack modules across rows.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "add_rack_module", "description": "Add a 3U Eurorack module (vco1, vco2_xdsp, pcm_workstation, vcf_ladder, lfo_adsr, vca_master, wavefolder_lpg, sample_hold_slew, euclidean_clock, stereo_delay, attenuverter, inline_scope) to a row (0..2).", "inputSchema": { "type": "object", "properties": { "kind": { "type": "string" }, "row": { "type": "integer", "minimum": 0, "maximum": 2 } }, "required": ["kind"] } },
        { "name": "remove_rack_module", "description": "Remove a 3U Eurorack module by module_id (also unplugs attached cables).", "inputSchema": { "type": "object", "properties": { "module_id": { "type": "string" } }, "required": ["module_id"] } },
        { "name": "move_rack_module", "description": "Move a 3U Eurorack module horizontally (delta_slot) or between rows (delta_row).", "inputSchema": { "type": "object", "properties": { "module_id": { "type": "string" }, "delta_row": { "type": "integer" }, "delta_slot": { "type": "integer" } }, "required": ["module_id"] } },
        { "name": "set_rack_module_param", "description": "Set normalized parameters param_a/param_b (0..1) or bypassed state on a 3U rack module.", "inputSchema": { "type": "object", "properties": { "module_id": { "type": "string" }, "param_a": { "type": "number" }, "param_b": { "type": "number" }, "bypassed": { "type": "boolean" } }, "required": ["module_id"] } },
        { "name": "reset_rack_layout", "description": "Restore the default 2-row 3U Eurorack module layout and default cables.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "list_rack_presets", "description": "List all 12 curated pre-patched 3U modular rack rigs (modules + cables + voicing).", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "load_rack_preset", "description": "Load one of the 12 curated pre-patched 3U modular rack rigs by preset_id (e.g. berlin_school_sequence, west_coast_wavefolder, acid_squelch_303, random_robot_bleeps, ambient_cathedral_drift, industrial_filter_bypass, dub_techno_chord_echo, fm_metallic_bells, sub_earthquake_reese, pcm_shimmer_workstation, polyrhythmic_glitch_lab, clean_studio_init).", "inputSchema": { "type": "object", "properties": { "preset_id": { "type": "string" } }, "required": ["preset_id"] } },
        { "name": "reset_factory_patches", "description": "Reset all factory patches to their default state.", "inputSchema": { "type": "object", "properties": {} } }
    ])
}

fn run_mcp_stdio(engine: &mut SynthEngine, db_path: &Path, policy: &PathPolicy) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let Ok(raw) = line else { break };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(req) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        if let Some(resp) = handle_mcp_request(&req, engine, db_path, policy) {
            if let Ok(s) = serde_json::to_string(&resp) {
                let _ = writeln!(stdout, "{s}");
                let _ = stdout.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("synthcraft_mcp_test_{}.db", std::process::id()));
        p
    }

    #[test]
    fn mcp_initialize_returns_2026_07_28_and_23_tools() {
        let mut eng = SynthEngine::default();
        let db = temp_db();
        let init = handle_mcp_request(
            &json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
            &mut eng,
            &db,
            &PathPolicy::default(),
        )
        .unwrap_or_default();
        assert_eq!(init["result"]["protocolVersion"], "2026-07-28");

        let list = handle_mcp_request(
            &json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
            &mut eng,
            &db,
            &PathPolicy::default(),
        )
        .unwrap_or_default();
        let tools = list["result"]["tools"]
            .as_array()
            .map(Vec::len)
            .unwrap_or(0);
        assert_eq!(tools, 35);
        let _ = std::fs::remove_file(&db);
    }

    #[test]
    fn mcp_tools_call_executes_spectrum_clap_and_djcraft_stem() {
        let mut eng = SynthEngine::default();
        let db = temp_db();

        let spec = handle_mcp_request(
            &json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "analyze_spectrum",
                    "arguments": { "duration_ms": 150 }
                }
            }),
            &mut eng,
            &db,
            &PathPolicy::default(),
        )
        .unwrap_or_default();
        assert_eq!(spec["result"]["isError"], false);

        let clap = handle_mcp_request(
            &json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "export_clap_manifest",
                    "arguments": {}
                }
            }),
            &mut eng,
            &db,
            &PathPolicy::default(),
        )
        .unwrap_or_default();
        assert_eq!(clap["result"]["isError"], false);

        let stem = handle_mcp_request(
            &json!({
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {
                    "name": "export_djcraft_stem",
                    "arguments": { "bars": 4 }
                }
            }),
            &mut eng,
            &db,
            &PathPolicy::default(),
        )
        .unwrap_or_default();
        assert_eq!(stem["result"]["isError"], false);
        let _ = std::fs::remove_file(&db);
    }
}
