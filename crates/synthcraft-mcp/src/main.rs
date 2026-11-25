//! SynthCraft L2 CLI & MCP `2026-07-28` stdio server.
#![forbid(unsafe_code)]
#![recursion_limit = "256"]

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
    /// Set or add a General Modulation Matrix route (slot 0..7)
    SetModRoute {
        #[arg(long, default_value_t = 0)]
        index: usize,
        #[arg(long)]
        source: String,
        #[arg(long)]
        dest: String,
        #[arg(long, default_value_t = 0.5)]
        depth: f32,
        #[arg(long, default_value_t = true)]
        enabled: bool,
    },
    /// Clear all routes in the active patch's Modulation Matrix
    ClearModMatrix,
    /// Configure the real-time Arpeggiator (off, up, down, up_down, random)
    SetArpeggiator {
        #[arg(long)]
        mode: String,
        #[arg(long, default_value_t = 4)]
        rate_div: u8,
        #[arg(long, default_value_t = 2)]
        octaves: u8,
        #[arg(long, default_value_t = 0.65)]
        gate: f32,
        #[arg(long, default_value_t = false)]
        latch: bool,
    },
    /// Set microtonal tuning preset (equal_12, just_5_limit, pythagorean, equal_19, equal_24, equal_31, bohlen_pierce) or inline Scala .scl text
    SetTuning {
        #[arg(long, default_value = "equal_12")]
        preset: String,
        #[arg(long)]
        scala_scl: Option<String>,
    },
    /// Load a built-in morphing wavetable (analog_morph, formant_vowel, metallic_bell, acid_fold) or a .wav file
    LoadWavetable { spec: String },
    /// Import a Standard MIDI File (.mid) into the 16-step sequencer
    ImportMidi { path: String },
    /// Manage the 3U CLAP Host module (scan, inspect, configure, status)
    #[command(subcommand)]
    ClapHost(ClapHostSub),
    /// Live DJCraft bridge (query running DJCraft or forge & load a stem directly onto a deck)
    #[command(subcommand)]
    Djcraft(DjcraftSub),
    /// List all built-in morphing wavetables and the active patch's wavetable
    ListWavetables,
    /// Record a sequence of MIDI notes directly into the 16-step sequencer
    RecordSeqNotes {
        /// Comma-separated MIDI notes (0..127), e.g. "36,36,48,39,43"
        notes: String,
        #[arg(long, default_value_t = 0)]
        start_step: usize,
        #[arg(long, default_value_t = 0.85)]
        velocity: f32,
    },
    /// Export the Web Audio AudioWorklet JS bridge (and compiled .wasm if built) to a directory
    ExportWasmBundle { output_dir: String },
    /// Set sequencer pattern length (16, 32, 48, or 64 steps = 1..4 bars)
    SetSequencerLength {
        #[arg(long, default_value_t = 32)]
        steps: usize,
    },
    /// Load a built-in 8-voice TR-808/909 drum machine kit preset
    LoadDrumPreset {
        /// Preset id: tr909_berlin_techno, tr808_electro, house_1990_classic, dnb_amen_174, acid_303_groove, polyrhythm_64
        preset_id: String,
    },
    /// Configure Drawable MSEG & Dual Morphing Filter B (Formant/Comb/MS-20)
    SetMsegFilter2 {
        #[arg(long, default_value = "tempo_loop")]
        mseg_mode: String,
        #[arg(long, default_value = "acid_pluck")]
        mseg_preset: String,
        #[arg(long, default_value = "formant_vowel")]
        filter2_mode: String,
        #[arg(long, default_value_t = 1400.0)]
        filter2_cutoff_hz: f32,
        #[arg(long, default_value_t = 1.0)]
        vowel_morph: f32,
        #[arg(long, default_value_t = 0.75)]
        filter2_mix: f32,
    },
    /// Configure 24-Grain Granular Cloud & 64-Partial Spectral Resynthesis
    SetGranularSpectral {
        #[arg(long, default_value_t = true)]
        granular_enabled: bool,
        #[arg(long, default_value_t = 0.45)]
        granular_mix: f32,
        #[arg(long, default_value_t = 90.0)]
        grain_size_ms: f32,
        #[arg(long, default_value_t = false)]
        freeze: bool,
        #[arg(long, default_value_t = true)]
        spectral_enabled: bool,
        #[arg(long, default_value = "vocal_choir")]
        spectral_preset: String,
        #[arg(long, default_value_t = 0.40)]
        spectral_mix: f32,
    },
    /// Run multi-core Genopatch Audio-to-Patch evolutionary search against a .wav or active patch
    RunGenopatch {
        #[arg(long)]
        wav_path: Option<String>,
        #[arg(long, default_value_t = 6)]
        generations: usize,
        #[arg(long, default_value_t = 32)]
        population: usize,
        #[arg(long, default_value_t = false)]
        apply_best: bool,
    },
    /// Configure 6-Op Thru-Zero FM Matrix & 16-Voice Hyper-Unison Stack
    SetFm6Unison {
        #[arg(long, default_value = "dx7_epiano")]
        fm6_preset: String,
        #[arg(long, default_value_t = true)]
        fm6_enabled: bool,
        #[arg(long, default_value_t = 0.65)]
        fm6_mix: f32,
        #[arg(long, default_value_t = true)]
        unison_enabled: bool,
        #[arg(long, default_value_t = 7)]
        unison_voices: u32,
        #[arg(long, default_value_t = 22.0)]
        unison_detune_cents: f32,
    },
    /// Configure Reorderable Studio FX Rack (OTT, Convolution IR, Shimmer, Tape/Bitcrusher)
    SetStudioFx {
        #[arg(long, default_value_t = true)]
        enabled: bool,
        #[arg(long, default_value_t = 0.45)]
        ott_depth: f32,
        #[arg(long, default_value_t = 0.30)]
        conv_mix: f32,
        #[arg(long, default_value_t = 0.35)]
        shimmer_mix: f32,
        #[arg(long, default_value_t = 0.25)]
        tape_wow: f32,
        #[arg(long, default_value_t = 16.0)]
        crush_bits: f32,
    },
    /// Configure Goliath 2D FDTD Physical Modeling Mesh & 8x Oversampled WDF Circuit Sim
    SetPhysicalWdf {
        #[arg(long, default_value_t = true)]
        mesh_enabled: bool,
        #[arg(long, default_value = "titanium_plate")]
        material: String,
        #[arg(long, default_value_t = 0.28)]
        tension: f32,
        #[arg(long, default_value_t = 0.45)]
        mesh_mix: f32,
        #[arg(long, default_value = "buchla292")]
        wdf_mode: String,
        #[arg(long, default_value_t = 1.2)]
        wdf_drive: f32,
    },
    /// Configure Neural DDSP Timbre Transfer (32-Harmonic + Filtered-Noise Decoder)
    SetDdspTimbre {
        #[arg(long, default_value_t = true)]
        enabled: bool,
        #[arg(long, default_value = "solo_violin_bow")]
        timbre_a: String,
        #[arg(long, default_value = "vienna_brass_horn")]
        timbre_b: String,
        #[arg(long, default_value_t = 0.35)]
        morph: f32,
        #[arg(long, default_value_t = 0.65)]
        bow_pressure: f32,
        #[arg(long, default_value_t = 0.25)]
        breath_noise: f32,
        #[arg(long, default_value_t = 0.50)]
        mix: f32,
    },
    /// Configure 1-Finger Chord Memory, Scale Lock, 16-Band Vocoder, 8-Scene Crossfader & Mastering Bus
    SetPerfMaster {
        #[arg(long, default_value = "house_stab_m9")]
        chord_mode: String,
        #[arg(long, default_value = "dorian")]
        scale_lock: String,
        #[arg(long, default_value_t = false)]
        vocoder_enabled: bool,
        #[arg(long, default_value_t = 0.0)]
        scene_crossfade: f32,
        #[arg(long, default_value_t = true)]
        mastering_enabled: bool,
        #[arg(long, default_value_t = 1.0)]
        stereo_width: f32,
        #[arg(long, default_value_t = -0.3)]
        limiter_ceiling_db: f32,
    },
    /// Reset all patches to factory defaults
    ResetFactory,
}

#[derive(Debug, Subcommand)]
pub enum ClapHostSub {
    /// Scan standard system CLAP plugin directories for installed .clap files
    Scan,
    /// Inspect an external .clap plugin binary (descriptors + automatable parameters)
    Inspect {
        path: String,
        #[arg(long, default_value_t = 0)]
        plugin_index: u32,
    },
    /// Configure the 3U CLAP Host rack module (plugin path, CV target param, mix, CV depth)
    Configure {
        #[arg(long)]
        plugin_path: Option<String>,
        #[arg(long, default_value_t = 0)]
        plugin_index: u32,
        #[arg(long)]
        cv_param_id: Option<u32>,
        #[arg(long, default_value_t = 1.0)]
        mix: f32,
        #[arg(long, default_value_t = 0.5)]
        cv_depth: f32,
    },
    /// Show the current 3U CLAP Host configuration and rack slot state
    Status,
}

#[derive(Debug, Subcommand)]
pub enum DjcraftSub {
    /// Query a running DJCraft GUI's live 4-deck BPM, playing state, and Camelot keys
    Status,
    /// Render a BPM/Camelot-locked WAV stem and load it directly onto DJCraft Deck A/B/C/D
    ForgeAndLoad {
        #[arg(long, default_value = "B")]
        deck: String,
        #[arg(long, default_value_t = 4)]
        bars: usize,
        #[arg(long)]
        output_path: String,
    },
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
    // Debug builds of `clap::Parser` with 40+ subcommands can exceed the 1 MiB default Windows
    // main-thread stack; run on an 8 MiB stack so debug and release behave identically.
    let handle = std::thread::Builder::new()
        .name("synthcraft-mcp".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(real_main);
    match handle {
        Ok(h) => {
            if h.join().is_err() {
                std::process::exit(1);
            }
        }
        Err(_) => real_main(),
    }
}

fn real_main() {
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
    run_cli_subcommand(sub, cli.json, &mut engine, &db_path, &policy);
}

#[inline(never)]
fn run_cli_subcommand(
    sub: CliSub,
    json_flag: bool,
    engine: &mut SynthEngine,
    db_path: &Path,
    policy: &PathPolicy,
) {
    match &sub {
        CliSub::ListBanks => {
            std::process::exit(run_list_banks(json_flag));
        }
        CliSub::FetchBank { id, dest } => {
            std::process::exit(run_fetch_bank(id, dest.as_deref(), policy));
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
        CliSub::SetModRoute {
            index,
            source,
            dest,
            depth,
            enabled,
        } => Command::SetModRoute {
            index,
            source,
            dest,
            depth,
            enabled,
        },
        CliSub::ClearModMatrix => Command::ClearModMatrix,
        CliSub::SetArpeggiator {
            mode,
            rate_div,
            octaves,
            gate,
            latch,
        } => Command::SetArpeggiator {
            mode,
            rate_div,
            octaves,
            gate,
            latch,
        },
        CliSub::SetTuning { preset, scala_scl } => Command::SetTuning { preset, scala_scl },
        CliSub::LoadWavetable { spec } => Command::LoadWavetable { spec },
        CliSub::ImportMidi { path } => Command::ImportMidiFile { path },
        CliSub::ClapHost(sub) => match sub {
            ClapHostSub::Scan => {
                let files: Vec<String> = synthcraft_clap_host::scan_clap_files(&[])
                    .into_iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                let out = serde_json::to_string_pretty(&json!({
                    "clap_dirs": synthcraft_clap_host::standard_clap_dirs()
                        .into_iter()
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect::<Vec<_>>(),
                    "plugins": files,
                }))
                .unwrap_or_else(|_| "{}".to_string());
                println!("{out}");
                return;
            }
            ClapHostSub::Inspect { path, plugin_index } => {
                let resolved = if policy.is_active() {
                    match policy.check_read(&path) {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("error: {e}");
                            std::process::exit(1);
                        }
                    }
                } else {
                    PathBuf::from(&path)
                };
                match synthcraft_clap_host::inspect_clap_file(&resolved, plugin_index) {
                    Ok((plugins, params)) => {
                        let out = serde_json::to_string_pretty(&json!({
                            "path": resolved.to_string_lossy(),
                            "plugins": plugins,
                            "params": params,
                        }))
                        .unwrap_or_else(|_| "{}".to_string());
                        println!("{out}");
                        return;
                    }
                    Err(e) => {
                        eprintln!("error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            ClapHostSub::Configure {
                plugin_path,
                plugin_index,
                cv_param_id,
                mix,
                cv_depth,
            } => Command::ConfigureClapHost {
                plugin_path,
                plugin_index,
                cv_param_id,
                mix,
                cv_depth,
            },
            ClapHostSub::Status => {
                let val = clap_host_status_json(engine);
                let out = serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string());
                println!("{out}");
                return;
            }
        },
        CliSub::Djcraft(sub) => match sub {
            DjcraftSub::Status => match synthcraft_core::query_djcraft_live_status() {
                Ok(val) => {
                    let out =
                        serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string());
                    println!("{out}");
                    return;
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            },
            DjcraftSub::ForgeAndLoad {
                deck,
                bars,
                output_path,
            } => {
                let resolved = if policy.is_active() {
                    match policy.check_write(&output_path) {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("error: {e}");
                            std::process::exit(1);
                        }
                    }
                } else {
                    PathBuf::from(&output_path)
                };
                match forge_and_load_djcraft(engine, &deck, bars, &resolved) {
                    Ok(val) => {
                        let out =
                            serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string());
                        println!("{out}");
                        return;
                    }
                    Err(e) => {
                        eprintln!("error: {e}");
                        std::process::exit(1);
                    }
                }
            }
        },
        CliSub::ListWavetables => {
            let val = list_wavetables_json(engine);
            let out = serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string());
            println!("{out}");
            return;
        }
        CliSub::RecordSeqNotes {
            notes,
            start_step,
            velocity,
        } => {
            let parsed: Vec<u8> = notes
                .split(',')
                .filter_map(|s| s.trim().parse::<u8>().ok())
                .collect();
            let before = serde_json::to_string(engine).unwrap_or_default();
            match record_seq_notes(engine, &parsed, start_step, velocity) {
                Ok(val) => {
                    if let Err(e) = save_if_changed(engine, &before, db_path) {
                        eprintln!("error: saving failed: {e}");
                        std::process::exit(1);
                    }
                    let out =
                        serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string());
                    println!("{out}");
                    return;
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        CliSub::ExportWasmBundle { output_dir } => {
            let resolved = if policy.is_active() {
                match policy.check_write(&output_dir) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("error: {e}");
                        std::process::exit(1);
                    }
                }
            } else {
                PathBuf::from(&output_dir)
            };
            match export_wasm_bundle(&resolved) {
                Ok(val) => {
                    let out =
                        serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string());
                    println!("{out}");
                    return;
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        CliSub::SetSequencerLength { steps } => Command::SetSequencerLength { steps },
        CliSub::LoadDrumPreset { preset_id } => Command::LoadDrumPreset { preset_id },
        CliSub::SetMsegFilter2 {
            mseg_mode,
            mseg_preset,
            filter2_mode,
            filter2_cutoff_hz,
            vowel_morph,
            filter2_mix,
        } => {
            let mut p = engine.active_patch().clone();
            p.mseg.mode = parse_mseg_mode(&mseg_mode);
            p.mseg.points = synthcraft_core::MsegParams::preset_points(&mseg_preset);
            p.filter2.mode = parse_filter2_mode(&filter2_mode);
            p.filter2.cutoff_hz = filter2_cutoff_hz.clamp(40.0, 16000.0);
            p.filter2.vowel_morph = vowel_morph.clamp(0.0, 4.0);
            p.filter2.mix = filter2_mix.clamp(0.0, 1.0);
            Command::SetNextGenParams {
                mseg: p.mseg,
                filter2: p.filter2,
                granular: p.granular,
                spectral: p.spectral,
            }
        }
        CliSub::SetGranularSpectral {
            granular_enabled,
            granular_mix,
            grain_size_ms,
            freeze,
            spectral_enabled,
            spectral_preset,
            spectral_mix,
        } => {
            let mut p = engine.active_patch().clone();
            p.granular.enabled = granular_enabled;
            p.granular.mix = granular_mix.clamp(0.0, 1.0);
            p.granular.grain_size_ms = grain_size_ms.clamp(10.0, 400.0);
            p.granular.freeze = freeze;
            p.spectral.enabled = spectral_enabled;
            p.spectral.load_preset(&spectral_preset);
            p.spectral.mix = spectral_mix.clamp(0.0, 1.0);
            Command::SetNextGenParams {
                mseg: p.mseg,
                filter2: p.filter2,
                granular: p.granular,
                spectral: p.spectral,
            }
        }
        CliSub::RunGenopatch {
            wav_path,
            generations,
            population,
            apply_best,
        } => {
            let before = serde_json::to_string(engine).unwrap_or_default();
            match run_genopatch_tool(
                engine,
                wav_path.as_deref(),
                generations,
                population,
                apply_best,
                policy,
            ) {
                Ok(val) => {
                    if let Err(e) = save_if_changed(engine, &before, db_path) {
                        eprintln!("error: saving failed: {e}");
                        std::process::exit(1);
                    }
                    let out =
                        serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string());
                    println!("{out}");
                    return;
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        CliSub::SetFm6Unison {
            fm6_preset,
            fm6_enabled,
            fm6_mix,
            unison_enabled,
            unison_voices,
            unison_detune_cents,
        } => {
            let mut p = engine.active_patch().clone();
            p.fm6 = synthcraft_core::Fm6Params::preset(&fm6_preset);
            p.fm6.enabled = fm6_enabled;
            p.fm6.mix = fm6_mix.clamp(0.0, 1.0);
            p.unison.enabled = unison_enabled;
            p.unison.voices = unison_voices.clamp(2, 16);
            p.unison.detune_cents = unison_detune_cents.clamp(0.0, 100.0);
            Command::SetProLabParams {
                fm6: Box::new(p.fm6),
                unison: p.unison,
                studio_fx: Box::new(p.studio_fx),
                physical_mesh: p.physical_mesh,
                wdf_circuit: p.wdf_circuit,
            }
        }
        CliSub::SetStudioFx {
            enabled,
            ott_depth,
            conv_mix,
            shimmer_mix,
            tape_wow,
            crush_bits,
        } => {
            let mut p = engine.active_patch().clone();
            p.studio_fx.enabled = enabled;
            p.studio_fx.ott_enabled = ott_depth > 0.001;
            p.studio_fx.ott_depth = ott_depth.clamp(0.0, 1.0);
            p.studio_fx.conv_enabled = conv_mix > 0.001;
            p.studio_fx.conv_mix = conv_mix.clamp(0.0, 1.0);
            p.studio_fx.shimmer_enabled = shimmer_mix > 0.001;
            p.studio_fx.shimmer_mix = shimmer_mix.clamp(0.0, 1.0);
            p.studio_fx.tape_enabled = tape_wow > 0.001 || crush_bits < 15.9;
            p.studio_fx.tape_wow = tape_wow.clamp(0.0, 1.0);
            p.studio_fx.crush_bits = crush_bits.clamp(4.0, 16.0);
            Command::SetProLabParams {
                fm6: Box::new(p.fm6),
                unison: p.unison,
                studio_fx: Box::new(p.studio_fx),
                physical_mesh: p.physical_mesh,
                wdf_circuit: p.wdf_circuit,
            }
        }
        CliSub::SetPhysicalWdf {
            mesh_enabled,
            material,
            tension,
            mesh_mix,
            wdf_mode,
            wdf_drive,
        } => {
            let mut p = engine.active_patch().clone();
            p.physical_mesh.enabled = mesh_enabled;
            p.physical_mesh.material = parse_physical_material(&material);
            p.physical_mesh.tension = tension.clamp(0.05, 0.44);
            p.physical_mesh.mix = mesh_mix.clamp(0.0, 1.0);
            p.wdf_circuit.mode = parse_wdf_mode(&wdf_mode);
            p.wdf_circuit.drive = wdf_drive.clamp(0.0, 3.0);
            Command::SetProLabParams {
                fm6: Box::new(p.fm6),
                unison: p.unison,
                studio_fx: Box::new(p.studio_fx),
                physical_mesh: p.physical_mesh,
                wdf_circuit: p.wdf_circuit,
            }
        }
        CliSub::SetDdspTimbre {
            enabled,
            timbre_a,
            timbre_b,
            morph,
            bow_pressure,
            breath_noise,
            mix,
        } => {
            let mut ddsp = engine.active_patch().ddsp;
            ddsp.enabled = enabled;
            ddsp.timbre_a = parse_ddsp_timbre(&timbre_a);
            ddsp.timbre_b = parse_ddsp_timbre(&timbre_b);
            ddsp.morph = morph.clamp(0.0, 1.0);
            ddsp.bow_pressure = bow_pressure.clamp(0.0, 1.0);
            ddsp.breath_noise = breath_noise.clamp(0.0, 1.0);
            ddsp.mix = mix.clamp(0.0, 1.0);
            Command::SetDdspParams { ddsp }
        }
        CliSub::SetPerfMaster {
            chord_mode,
            scale_lock,
            vocoder_enabled,
            scene_crossfade,
            mastering_enabled,
            stereo_width,
            limiter_ceiling_db,
        } => {
            let mut p = engine.active_patch().clone();
            p.chord_mem.chord_mode = parse_chord_mode(&chord_mode);
            p.chord_mem.scale_lock = parse_scale_lock(&scale_lock);
            p.vocoder.enabled = vocoder_enabled;
            p.scenes.ensure_len();
            p.scenes.crossfade = scene_crossfade.clamp(0.0, 1.0);
            if scene_crossfade > 0.001 {
                p.scenes.blended_crossfade().apply_to(&mut p);
            }
            p.mastering.enabled = mastering_enabled;
            p.mastering.stereo_width = stereo_width.clamp(0.0, 2.0);
            p.mastering.limiter_ceiling_db = limiter_ceiling_db.clamp(-6.0, -0.1);
            Command::SetPerfMasterParams {
                chord_mem: p.chord_mem,
                vocoder: p.vocoder,
                scenes: Box::new(p.scenes),
                mastering: p.mastering,
            }
        }
        CliSub::ResetFactory => Command::ResetFactoryPatches,
    };
    let cmd = if policy.is_active() {
        match enforce_command(cmd, policy) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    } else {
        cmd
    };

    let before = serde_json::to_string(engine).unwrap_or_default();
    match cmd.dispatch(engine) {
        Ok(val) => {
            if let Err(e) = save_if_changed(engine, &before, db_path) {
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
        Command::ImportMidiFile { path } => Command::ImportMidiFile {
            path: policy.check_read(&path)?.to_string_lossy().into_owned(),
        },
        Command::LoadWavetable { spec } => {
            if synthcraft_core::Wavetable::builtin(&spec).is_some() {
                Command::LoadWavetable { spec }
            } else {
                Command::LoadWavetable {
                    spec: policy.check_read(&spec)?.to_string_lossy().into_owned(),
                }
            }
        }
        Command::ConfigureClapHost {
            plugin_path,
            plugin_index,
            cv_param_id,
            mix,
            cv_depth,
        } => Command::ConfigureClapHost {
            plugin_path: match plugin_path {
                Some(p) => Some(policy.check_read(&p)?.to_string_lossy().into_owned()),
                None => None,
            },
            plugin_index,
            cv_param_id,
            mix,
            cv_depth,
        },
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
                "set_mod_route" => Command::SetModRoute {
                    index: args.get("index").and_then(Value::as_u64).unwrap_or(0) as usize,
                    source: args
                        .get("source")
                        .and_then(Value::as_str)
                        .unwrap_or("lfo")
                        .to_string(),
                    dest: args
                        .get("dest")
                        .and_then(Value::as_str)
                        .unwrap_or("cutoff")
                        .to_string(),
                    depth: args.get("depth").and_then(Value::as_f64).unwrap_or(0.5) as f32,
                    enabled: args.get("enabled").and_then(Value::as_bool).unwrap_or(true),
                }
                .dispatch(engine),
                "clear_mod_matrix" => Command::ClearModMatrix.dispatch(engine),
                "set_arpeggiator" => Command::SetArpeggiator {
                    mode: args
                        .get("mode")
                        .and_then(Value::as_str)
                        .unwrap_or("off")
                        .to_string(),
                    rate_div: args.get("rate_div").and_then(Value::as_u64).unwrap_or(4) as u8,
                    octaves: args.get("octaves").and_then(Value::as_u64).unwrap_or(2) as u8,
                    gate: args.get("gate").and_then(Value::as_f64).unwrap_or(0.65) as f32,
                    latch: args.get("latch").and_then(Value::as_bool).unwrap_or(false),
                }
                .dispatch(engine),
                "set_tuning" => Command::SetTuning {
                    preset: args
                        .get("preset")
                        .and_then(Value::as_str)
                        .unwrap_or("equal_12")
                        .to_string(),
                    scala_scl: args
                        .get("scala_scl")
                        .and_then(Value::as_str)
                        .map(String::from),
                }
                .dispatch(engine),
                "load_wavetable" => {
                    let spec = args
                        .get("spec")
                        .and_then(Value::as_str)
                        .unwrap_or("analog_morph")
                        .to_string();
                    enforce_command(Command::LoadWavetable { spec }, policy)
                        .and_then(|c| c.dispatch(engine))
                }
                "import_midi_file" => {
                    let path = args
                        .get("path")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    enforce_command(Command::ImportMidiFile { path }, policy)
                        .and_then(|c| c.dispatch(engine))
                }
                "scan_clap_plugins" => {
                    let files: Vec<String> = synthcraft_clap_host::scan_clap_files(&[])
                        .into_iter()
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect();
                    let dirs: Vec<String> = synthcraft_clap_host::standard_clap_dirs()
                        .into_iter()
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect();
                    Ok(json!({ "clap_dirs": dirs, "plugins": files }))
                }
                "inspect_clap_plugin" => {
                    let path = args.get("path").and_then(Value::as_str).unwrap_or_default();
                    let plugin_index = args
                        .get("plugin_index")
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as u32;
                    policy.check_read(path).and_then(|resolved| {
                        let (plugins, params) =
                            synthcraft_clap_host::inspect_clap_file(&resolved, plugin_index)?;
                        Ok(json!({
                            "path": resolved.to_string_lossy(),
                            "plugins": plugins,
                            "params": params,
                        }))
                    })
                }
                "configure_clap_host" => {
                    let plugin_path = args
                        .get("plugin_path")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    let plugin_index = args
                        .get("plugin_index")
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as u32;
                    let cv_param_id = args
                        .get("cv_param_id")
                        .and_then(Value::as_u64)
                        .map(|v| v as u32);
                    let mix = args.get("mix").and_then(Value::as_f64).unwrap_or(1.0) as f32;
                    let cv_depth =
                        args.get("cv_depth").and_then(Value::as_f64).unwrap_or(0.5) as f32;
                    enforce_command(
                        Command::ConfigureClapHost {
                            plugin_path,
                            plugin_index,
                            cv_param_id,
                            mix,
                            cv_depth,
                        },
                        policy,
                    )
                    .and_then(|c| c.dispatch(engine))
                }
                "get_clap_host_status" => Ok(clap_host_status_json(engine)),
                "list_wavetables" => Ok(list_wavetables_json(engine)),
                "record_sequencer_notes" => {
                    let notes: Vec<u8> = args
                        .get("notes")
                        .and_then(Value::as_array)
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_u64().map(|n| n.min(127) as u8))
                                .collect()
                        })
                        .unwrap_or_default();
                    let start_step =
                        args.get("start_step").and_then(Value::as_u64).unwrap_or(0) as usize;
                    let velocity =
                        args.get("velocity").and_then(Value::as_f64).unwrap_or(0.85) as f32;
                    record_seq_notes(engine, &notes, start_step, velocity)
                }
                "export_wasm_bundle" => {
                    let output_dir = args
                        .get("output_dir")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    policy
                        .check_write(output_dir)
                        .and_then(|resolved| export_wasm_bundle(&resolved))
                }
                "djcraft_live_status" => synthcraft_core::query_djcraft_live_status(),
                "forge_and_load_djcraft_deck" => {
                    let deck = args.get("deck").and_then(Value::as_str).unwrap_or("B");
                    let bars = args.get("bars").and_then(Value::as_u64).unwrap_or(4) as usize;
                    let output_path = args
                        .get("output_path")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    policy
                        .check_write(output_path)
                        .and_then(|resolved| forge_and_load_djcraft(engine, deck, bars, &resolved))
                }
                "reset_factory_patches" => Command::ResetFactoryPatches.dispatch(engine),
                "set_sequencer_length" => {
                    let steps = args.get("steps").and_then(Value::as_u64).unwrap_or(32) as usize;
                    Command::SetSequencerLength { steps }.dispatch(engine)
                }
                "load_drum_preset" => {
                    let preset_id = args
                        .get("preset_id")
                        .and_then(Value::as_str)
                        .unwrap_or("tr909_berlin_techno")
                        .to_string();
                    Command::LoadDrumPreset { preset_id }.dispatch(engine)
                }
                "set_mseg_filter2" => {
                    let mut p = engine.active_patch().clone();
                    if let Some(m) = args.get("mseg_mode").and_then(Value::as_str) {
                        p.mseg.mode = parse_mseg_mode(m);
                    }
                    if let Some(pr) = args.get("mseg_preset").and_then(Value::as_str) {
                        p.mseg.points = synthcraft_core::MsegParams::preset_points(pr);
                    }
                    if let Some(fm) = args.get("filter2_mode").and_then(Value::as_str) {
                        p.filter2.mode = parse_filter2_mode(fm);
                    }
                    if let Some(c) = args.get("filter2_cutoff_hz").and_then(Value::as_f64) {
                        p.filter2.cutoff_hz = (c as f32).clamp(40.0, 16000.0);
                    }
                    if let Some(v) = args.get("vowel_morph").and_then(Value::as_f64) {
                        p.filter2.vowel_morph = (v as f32).clamp(0.0, 4.0);
                    }
                    if let Some(mx) = args.get("filter2_mix").and_then(Value::as_f64) {
                        p.filter2.mix = (mx as f32).clamp(0.0, 1.0);
                    }
                    Command::SetNextGenParams {
                        mseg: p.mseg,
                        filter2: p.filter2,
                        granular: p.granular,
                        spectral: p.spectral,
                    }
                    .dispatch(engine)
                }
                "set_granular_spectral" => {
                    let mut p = engine.active_patch().clone();
                    if let Some(b) = args.get("granular_enabled").and_then(Value::as_bool) {
                        p.granular.enabled = b;
                    }
                    if let Some(v) = args.get("granular_mix").and_then(Value::as_f64) {
                        p.granular.mix = (v as f32).clamp(0.0, 1.0);
                    }
                    if let Some(v) = args.get("grain_size_ms").and_then(Value::as_f64) {
                        p.granular.grain_size_ms = (v as f32).clamp(10.0, 400.0);
                    }
                    if let Some(b) = args.get("freeze").and_then(Value::as_bool) {
                        p.granular.freeze = b;
                    }
                    if let Some(b) = args.get("spectral_enabled").and_then(Value::as_bool) {
                        p.spectral.enabled = b;
                    }
                    if let Some(pr) = args.get("spectral_preset").and_then(Value::as_str) {
                        p.spectral.load_preset(pr);
                    }
                    if let Some(v) = args.get("spectral_mix").and_then(Value::as_f64) {
                        p.spectral.mix = (v as f32).clamp(0.0, 1.0);
                    }
                    Command::SetNextGenParams {
                        mseg: p.mseg,
                        filter2: p.filter2,
                        granular: p.granular,
                        spectral: p.spectral,
                    }
                    .dispatch(engine)
                }
                "run_genopatch" => {
                    let wav = args.get("wav_path").and_then(Value::as_str);
                    let gens =
                        args.get("generations").and_then(Value::as_u64).unwrap_or(5) as usize;
                    let pop = args.get("population").and_then(Value::as_u64).unwrap_or(24) as usize;
                    let apply = args
                        .get("apply_best")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    run_genopatch_tool(engine, wav, gens, pop, apply, policy)
                }
                "set_fm6_unison" => {
                    let mut p = engine.active_patch().clone();
                    if let Some(pr) = args.get("fm6_preset").and_then(Value::as_str) {
                        p.fm6 = synthcraft_core::Fm6Params::preset(pr);
                    }
                    if let Some(b) = args.get("fm6_enabled").and_then(Value::as_bool) {
                        p.fm6.enabled = b;
                    }
                    if let Some(v) = args.get("fm6_mix").and_then(Value::as_f64) {
                        p.fm6.mix = (v as f32).clamp(0.0, 1.0);
                    }
                    if let Some(b) = args.get("unison_enabled").and_then(Value::as_bool) {
                        p.unison.enabled = b;
                    }
                    if let Some(v) = args.get("unison_voices").and_then(Value::as_u64) {
                        p.unison.voices = (v as u32).clamp(2, 16);
                    }
                    if let Some(v) = args.get("unison_detune_cents").and_then(Value::as_f64) {
                        p.unison.detune_cents = (v as f32).clamp(0.0, 100.0);
                    }
                    Command::SetProLabParams {
                        fm6: Box::new(p.fm6),
                        unison: p.unison,
                        studio_fx: Box::new(p.studio_fx),
                        physical_mesh: p.physical_mesh,
                        wdf_circuit: p.wdf_circuit,
                    }
                    .dispatch(engine)
                }
                "set_studio_fx" => {
                    let mut p = engine.active_patch().clone();
                    if let Some(b) = args.get("enabled").and_then(Value::as_bool) {
                        p.studio_fx.enabled = b;
                    }
                    if let Some(v) = args.get("ott_depth").and_then(Value::as_f64) {
                        p.studio_fx.ott_depth = (v as f32).clamp(0.0, 1.0);
                        p.studio_fx.ott_enabled = p.studio_fx.ott_depth > 0.001;
                    }
                    if let Some(v) = args.get("conv_mix").and_then(Value::as_f64) {
                        p.studio_fx.conv_mix = (v as f32).clamp(0.0, 1.0);
                        p.studio_fx.conv_enabled = p.studio_fx.conv_mix > 0.001;
                    }
                    if let Some(v) = args.get("shimmer_mix").and_then(Value::as_f64) {
                        p.studio_fx.shimmer_mix = (v as f32).clamp(0.0, 1.0);
                        p.studio_fx.shimmer_enabled = p.studio_fx.shimmer_mix > 0.001;
                    }
                    if let Some(v) = args.get("tape_wow").and_then(Value::as_f64) {
                        p.studio_fx.tape_wow = (v as f32).clamp(0.0, 1.0);
                        p.studio_fx.tape_enabled = p.studio_fx.tape_wow > 0.001;
                    }
                    Command::SetProLabParams {
                        fm6: Box::new(p.fm6),
                        unison: p.unison,
                        studio_fx: Box::new(p.studio_fx),
                        physical_mesh: p.physical_mesh,
                        wdf_circuit: p.wdf_circuit,
                    }
                    .dispatch(engine)
                }
                "set_physical_wdf" => {
                    let mut p = engine.active_patch().clone();
                    if let Some(b) = args.get("mesh_enabled").and_then(Value::as_bool) {
                        p.physical_mesh.enabled = b;
                    }
                    if let Some(m) = args.get("material").and_then(Value::as_str) {
                        p.physical_mesh.material = parse_physical_material(m);
                    }
                    if let Some(v) = args.get("tension").and_then(Value::as_f64) {
                        p.physical_mesh.tension = (v as f32).clamp(0.05, 0.44);
                    }
                    if let Some(v) = args.get("mesh_mix").and_then(Value::as_f64) {
                        p.physical_mesh.mix = (v as f32).clamp(0.0, 1.0);
                    }
                    if let Some(w) = args.get("wdf_mode").and_then(Value::as_str) {
                        p.wdf_circuit.mode = parse_wdf_mode(w);
                    }
                    if let Some(v) = args.get("wdf_drive").and_then(Value::as_f64) {
                        p.wdf_circuit.drive = (v as f32).clamp(0.0, 3.0);
                    }
                    Command::SetProLabParams {
                        fm6: Box::new(p.fm6),
                        unison: p.unison,
                        studio_fx: Box::new(p.studio_fx),
                        physical_mesh: p.physical_mesh,
                        wdf_circuit: p.wdf_circuit,
                    }
                    .dispatch(engine)
                }
                "set_ddsp_timbre" => {
                    let mut ddsp = engine.active_patch().ddsp;
                    if let Some(b) = args.get("enabled").and_then(Value::as_bool) {
                        ddsp.enabled = b;
                    }
                    if let Some(s) = args.get("timbre_a").and_then(Value::as_str) {
                        ddsp.timbre_a = parse_ddsp_timbre(s);
                    }
                    if let Some(s) = args.get("timbre_b").and_then(Value::as_str) {
                        ddsp.timbre_b = parse_ddsp_timbre(s);
                    }
                    if let Some(v) = args.get("morph").and_then(Value::as_f64) {
                        ddsp.morph = (v as f32).clamp(0.0, 1.0);
                    }
                    if let Some(v) = args.get("bow_pressure").and_then(Value::as_f64) {
                        ddsp.bow_pressure = (v as f32).clamp(0.0, 1.0);
                    }
                    if let Some(v) = args.get("breath_noise").and_then(Value::as_f64) {
                        ddsp.breath_noise = (v as f32).clamp(0.0, 1.0);
                    }
                    if let Some(v) = args.get("mix").and_then(Value::as_f64) {
                        ddsp.mix = (v as f32).clamp(0.0, 1.0);
                    }
                    Command::SetDdspParams { ddsp }.dispatch(engine)
                }
                "set_perf_master" => {
                    let mut p = engine.active_patch().clone();
                    if let Some(s) = args.get("chord_mode").and_then(Value::as_str) {
                        p.chord_mem.chord_mode = parse_chord_mode(s);
                    }
                    if let Some(s) = args.get("scale_lock").and_then(Value::as_str) {
                        p.chord_mem.scale_lock = parse_scale_lock(s);
                    }
                    if let Some(b) = args.get("vocoder_enabled").and_then(Value::as_bool) {
                        p.vocoder.enabled = b;
                    }
                    if let Some(v) = args.get("scene_crossfade").and_then(Value::as_f64) {
                        p.scenes.ensure_len();
                        p.scenes.crossfade = (v as f32).clamp(0.0, 1.0);
                        p.scenes.blended_crossfade().apply_to(&mut p);
                    }
                    if let Some(b) = args.get("mastering_enabled").and_then(Value::as_bool) {
                        p.mastering.enabled = b;
                    }
                    if let Some(v) = args.get("stereo_width").and_then(Value::as_f64) {
                        p.mastering.stereo_width = (v as f32).clamp(0.0, 2.0);
                    }
                    if let Some(v) = args.get("limiter_ceiling_db").and_then(Value::as_f64) {
                        p.mastering.limiter_ceiling_db = (v as f32).clamp(-6.0, -0.1);
                    }
                    Command::SetPerfMasterParams {
                        chord_mem: p.chord_mem,
                        vocoder: p.vocoder,
                        scenes: Box::new(p.scenes),
                        mastering: p.mastering,
                    }
                    .dispatch(engine)
                }
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
        { "name": "update_oscillators", "description": "Update VCO1 and VCO2 waveforms (sine, saw, square, triangle, pulse_pwm, supersaw, white_noise, sub_octave, atmo_pad, wavetable), detune cents, mix, and FM index.", "inputSchema": { "type": "object", "properties": { "vco1_wave": { "type": "string" }, "vco2_wave": { "type": "string" }, "detune_cents": { "type": "number" }, "mix": { "type": "number" }, "fm_index": { "type": "number" } }, "required": ["vco1_wave", "vco2_wave"] } },
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
        { "name": "add_rack_module", "description": "Add a 3U Eurorack module (vco1, vco2_xdsp, pcm_workstation, vcf_ladder, lfo_adsr, vca_master, wavefolder_lpg, sample_hold_slew, euclidean_clock, stereo_delay, attenuverter, inline_scope, clap_host) to a row (0..2).", "inputSchema": { "type": "object", "properties": { "kind": { "type": "string" }, "row": { "type": "integer", "minimum": 0, "maximum": 2 } }, "required": ["kind"] } },
        { "name": "remove_rack_module", "description": "Remove a 3U Eurorack module by module_id (also unplugs attached cables).", "inputSchema": { "type": "object", "properties": { "module_id": { "type": "string" } }, "required": ["module_id"] } },
        { "name": "move_rack_module", "description": "Move a 3U Eurorack module horizontally (delta_slot) or between rows (delta_row).", "inputSchema": { "type": "object", "properties": { "module_id": { "type": "string" }, "delta_row": { "type": "integer" }, "delta_slot": { "type": "integer" } }, "required": ["module_id"] } },
        { "name": "set_rack_module_param", "description": "Set normalized parameters param_a/param_b (0..1) or bypassed state on a 3U rack module.", "inputSchema": { "type": "object", "properties": { "module_id": { "type": "string" }, "param_a": { "type": "number" }, "param_b": { "type": "number" }, "bypassed": { "type": "boolean" } }, "required": ["module_id"] } },
        { "name": "reset_rack_layout", "description": "Restore the default 2-row 3U Eurorack module layout and default cables.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "list_rack_presets", "description": "List all 12 curated pre-patched 3U modular rack rigs (modules + cables + voicing).", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "load_rack_preset", "description": "Load one of the 12 curated pre-patched 3U modular rack rigs by preset_id (e.g. berlin_school_sequence, west_coast_wavefolder, acid_squelch_303, random_robot_bleeps, ambient_cathedral_drift, industrial_filter_bypass, dub_techno_chord_echo, fm_metallic_bells, sub_earthquake_reese, pcm_shimmer_workstation, polyrhythmic_glitch_lab, clean_studio_init).", "inputSchema": { "type": "object", "properties": { "preset_id": { "type": "string" } }, "required": ["preset_id"] } },
        { "name": "set_mod_route", "description": "Configure one General Modulation Matrix route (index 0..7): source (lfo, adsr_env, velocity, mod_wheel, sample_hold, ambient_drift) -> dest (cutoff, resonance, fm_index, pulse_width, vco2_detune, wavefold, stereo_pan, extreme_morph, reverb_mix) with bipolar depth (-1..1).", "inputSchema": { "type": "object", "properties": { "index": { "type": "integer", "minimum": 0, "maximum": 7 }, "source": { "type": "string" }, "dest": { "type": "string" }, "depth": { "type": "number" }, "enabled": { "type": "boolean" } }, "required": ["source", "dest", "depth"] } },
        { "name": "clear_mod_matrix", "description": "Clear all routes from the active patch's Modulation Matrix.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "set_arpeggiator", "description": "Configure the real-time Arpeggiator: mode (off, up, down, up_down, random), rate_div (1=1/4, 2=1/8, 4=1/16, 8=1/32), octaves (1..4), gate (0.1..1.0), latch (bool).", "inputSchema": { "type": "object", "properties": { "mode": { "type": "string" }, "rate_div": { "type": "integer" }, "octaves": { "type": "integer" }, "gate": { "type": "number" }, "latch": { "type": "boolean" } }, "required": ["mode"] } },
        { "name": "set_tuning", "description": "Set microtonal tuning preset (equal_12, just_5_limit, pythagorean, equal_19, equal_24, equal_31, bohlen_pierce) or pass custom Scala .scl text in scala_scl.", "inputSchema": { "type": "object", "properties": { "preset": { "type": "string" }, "scala_scl": { "type": "string" } } } },
        { "name": "load_wavetable", "description": "Load a built-in morphing wavetable (analog_morph, formant_vowel, metallic_bell, acid_fold) or a .wav file (inside --automation-read-root) and switch VCO1 to Wavetable mode.", "inputSchema": { "type": "object", "properties": { "spec": { "type": "string" } }, "required": ["spec"] } },
        { "name": "import_midi_file", "description": "Import a Standard MIDI File (.mid, inside --automation-read-root) and quantize the first bar into the 16-step sequencer.", "inputSchema": { "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"] } },
        { "name": "scan_clap_plugins", "description": "Scan standard system CLAP directories for installed .clap plugins.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "inspect_clap_plugin", "description": "Inspect an external .clap plugin binary (inside --automation-read-root) and return its plugin descriptors and automatable parameters.", "inputSchema": { "type": "object", "properties": { "path": { "type": "string" }, "plugin_index": { "type": "integer" } }, "required": ["path"] } },
        { "name": "configure_clap_host", "description": "Configure the 3U CLAP Host rack module (plugin_path inside --automation-read-root, plugin_index, cv_param_id, mix 0..1, cv_depth -1..1).", "inputSchema": { "type": "object", "properties": { "plugin_path": { "type": "string" }, "plugin_index": { "type": "integer" }, "cv_param_id": { "type": "integer" }, "mix": { "type": "number" }, "cv_depth": { "type": "number" } } } },
        { "name": "get_clap_host_status", "description": "Return the current 3U CLAP Host configuration and whether a CLAP HOST module is placed in the rack.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "list_wavetables", "description": "List all built-in morphing wavetables (analog_morph, formant_vowel, metallic_bell, acid_fold) and the active patch's selected wavetable.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "record_sequencer_notes", "description": "Step-record a sequence of MIDI notes (0..127) into the active patch's 16-step sequencer starting at start_step (0..15) with velocity (0.05..1.0; >=0.88 sets accent).", "inputSchema": { "type": "object", "properties": { "notes": { "type": "array", "items": { "type": "integer", "minimum": 0, "maximum": 127 } }, "start_step": { "type": "integer", "minimum": 0, "maximum": 15 }, "velocity": { "type": "number" } }, "required": ["notes"] } },
        { "name": "export_wasm_bundle", "description": "Export the Web Audio AudioWorklet bridge (synthcraft-worklet.js) and compiled synthcraft_wasm.wasm (if built) into output_dir (inside --automation-write-root).", "inputSchema": { "type": "object", "properties": { "output_dir": { "type": "string" } }, "required": ["output_dir"] } },
        { "name": "djcraft_live_status", "description": "Query a running DJCraft GUI's live 4-deck BPM, playing state, and Camelot keys via its loopback live.port IPC.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "forge_and_load_djcraft_deck", "description": "Render a BPM/Camelot-locked WAV stem (output_path inside --automation-write-root) and load it directly onto Deck A/B/C/D of a running DJCraft GUI.", "inputSchema": { "type": "object", "properties": { "deck": { "type": "string" }, "bars": { "type": "integer", "minimum": 1, "maximum": 16 }, "output_path": { "type": "string" } }, "required": ["output_path"] } },
        { "name": "reset_factory_patches", "description": "Reset all factory patches to their default state.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "set_sequencer_length", "description": "Set the acid sequencer pattern length to 16, 32, 48, or 64 steps (1..4 bars).", "inputSchema": { "type": "object", "properties": { "steps": { "type": "integer", "enum": [16, 32, 48, 64] } }, "required": ["steps"] } },
        { "name": "load_drum_preset", "description": "Load a built-in 8-voice TR-808/909 drum machine kit preset (tr909_berlin_techno, tr808_electro, house_1990_classic, dnb_amen_174, acid_303_groove, polyrhythm_64).", "inputSchema": { "type": "object", "properties": { "preset_id": { "type": "string" } }, "required": ["preset_id"] } },
        { "name": "set_mseg_filter2", "description": "Configure the drawable MSEG envelope and Dual Morphing Filter B (formant_vowel, comb_positive, comb_negative, ms20_screamer, lp24, bp12, hp12).", "inputSchema": { "type": "object", "properties": { "mseg_mode": { "type": "string" }, "mseg_preset": { "type": "string" }, "filter2_mode": { "type": "string" }, "filter2_cutoff_hz": { "type": "number" }, "vowel_morph": { "type": "number" }, "filter2_mix": { "type": "number" } } } },
        { "name": "set_granular_spectral", "description": "Configure the 24-grain live-capture Granular Cloud and 64-Partial Spectral Additive Resynthesis engine.", "inputSchema": { "type": "object", "properties": { "granular_enabled": { "type": "boolean" }, "granular_mix": { "type": "number" }, "grain_size_ms": { "type": "number" }, "freeze": { "type": "boolean" }, "spectral_enabled": { "type": "boolean" }, "spectral_preset": { "type": "string" }, "spectral_mix": { "type": "number" } } } },
        { "name": "run_genopatch", "description": "Run multi-core evolutionary Genopatch audio-to-patch search against a target .wav file (inside --automation-read-root) or the active patch, returning 6 ranked DNA branches.", "inputSchema": { "type": "object", "properties": { "wav_path": { "type": "string" }, "generations": { "type": "integer" }, "population": { "type": "integer" }, "apply_best": { "type": "boolean" } } } },
        { "name": "set_fm6_unison", "description": "Configure the 6-Operator Thru-Zero FM Matrix (dx7_epiano, berlin_fm_bell, lately_bass, neuro_growl, glass_choir) and 16-Voice Hyper-Unison stack.", "inputSchema": { "type": "object", "properties": { "fm6_preset": { "type": "string" }, "fm6_enabled": { "type": "boolean" }, "fm6_mix": { "type": "number" }, "unison_enabled": { "type": "boolean" }, "unison_voices": { "type": "integer" }, "unison_detune_cents": { "type": "number" } } } },
        { "name": "set_studio_fx", "description": "Configure the Reorderable Studio FX Rack (OTT 3-band compressor, Convolution IR reverb, Shimmer pitch reverb, Analog Tape & Bitcrusher).", "inputSchema": { "type": "object", "properties": { "enabled": { "type": "boolean" }, "ott_depth": { "type": "number" }, "conv_mix": { "type": "number" }, "shimmer_mix": { "type": "number" }, "tape_wow": { "type": "number" } } } },
        { "name": "set_physical_wdf", "description": "Configure the Goliath 2D FDTD Physical Modeling Mesh (titanium_plate, glass_marimba, kettle_membrane, bowed_cello) and 8x Oversampled WDF Circuit Sim (moog_cp3, buchla292, tb303).", "inputSchema": { "type": "object", "properties": { "mesh_enabled": { "type": "boolean" }, "material": { "type": "string" }, "tension": { "type": "number" }, "mesh_mix": { "type": "number" }, "wdf_mode": { "type": "string" }, "wdf_drive": { "type": "number" } } } },
        { "name": "set_ddsp_timbre", "description": "Configure the Neural DDSP Timbre Transfer decoder (solo_violin_bow, vienna_brass_horn, breath_flute, gamelan_metal, alien_vocal_choir, cello_sul_ponticello).", "inputSchema": { "type": "object", "properties": { "enabled": { "type": "boolean" }, "timbre_a": { "type": "string" }, "timbre_b": { "type": "string" }, "morph": { "type": "number" }, "bow_pressure": { "type": "number" }, "breath_noise": { "type": "number" }, "mix": { "type": "number" } } } },
        { "name": "set_perf_master", "description": "Configure 1-Finger Chord Memory, Scale Lock, 16-Band Analog Vocoder, 8-Scene Crossfader, and the Studio Mastering Chain (4-band EQ, M/S width, brickwall limiter).", "inputSchema": { "type": "object", "properties": { "chord_mode": { "type": "string" }, "scale_lock": { "type": "string" }, "vocoder_enabled": { "type": "boolean" }, "scene_crossfade": { "type": "number" }, "mastering_enabled": { "type": "boolean" }, "stereo_width": { "type": "number" }, "limiter_ceiling_db": { "type": "number" } } } }
    ])
}

fn parse_mseg_mode(s: &str) -> synthcraft_core::MsegMode {
    match s.trim().to_ascii_lowercase().as_str() {
        "off" => synthcraft_core::MsegMode::Off,
        "free_hz" | "free" => synthcraft_core::MsegMode::FreeHz,
        "one_shot" | "oneshot" => synthcraft_core::MsegMode::OneShot,
        _ => synthcraft_core::MsegMode::TempoLoop,
    }
}

fn parse_filter2_mode(s: &str) -> synthcraft_core::Filter2Mode {
    match s.trim().to_ascii_lowercase().as_str() {
        "off" => synthcraft_core::Filter2Mode::Off,
        "lp24" | "lowpass" => synthcraft_core::Filter2Mode::LowPass24dB,
        "bp12" | "bandpass" => synthcraft_core::Filter2Mode::BandPass12dB,
        "hp12" | "highpass" => synthcraft_core::Filter2Mode::HighPass12dB,
        "comb_positive" | "comb+" => synthcraft_core::Filter2Mode::CombPositive,
        "comb_negative" | "comb-" => synthcraft_core::Filter2Mode::CombNegative,
        "ms20_screamer" | "ms20" => synthcraft_core::Filter2Mode::Ms20Screamer,
        _ => synthcraft_core::Filter2Mode::FormantVowel,
    }
}

fn parse_physical_material(s: &str) -> synthcraft_core::PhysicalMaterial {
    match s.trim().to_ascii_lowercase().as_str() {
        "glass_marimba" | "glass" => synthcraft_core::PhysicalMaterial::GlassMarimba,
        "kettle_membrane" | "kettle" => synthcraft_core::PhysicalMaterial::KettleMembrane,
        "bowed_cello" | "cello" => synthcraft_core::PhysicalMaterial::BowedCelloBody,
        _ => synthcraft_core::PhysicalMaterial::TitaniumPlate,
    }
}

fn parse_wdf_mode(s: &str) -> synthcraft_core::WdfCircuitMode {
    match s.trim().to_ascii_lowercase().as_str() {
        "off" => synthcraft_core::WdfCircuitMode::Off,
        "moog_cp3" | "cp3" => synthcraft_core::WdfCircuitMode::MoogCp3Overdrive,
        "tb303" | "tb303_diode" => synthcraft_core::WdfCircuitMode::Tb303DiodeLadder,
        _ => synthcraft_core::WdfCircuitMode::Buchla292VactrolLpg,
    }
}

fn parse_ddsp_timbre(s: &str) -> synthcraft_core::DdspTimbre {
    match s.trim().to_ascii_lowercase().as_str() {
        "vienna_brass_horn" | "brass" => synthcraft_core::DdspTimbre::ViennaBrassHorn,
        "breath_flute" | "flute" => synthcraft_core::DdspTimbre::BreathFlute,
        "gamelan_metal" | "gamelan" => synthcraft_core::DdspTimbre::GamelanMetal,
        "alien_vocal_choir" | "choir" => synthcraft_core::DdspTimbre::AlienVocalChoir,
        "cello_sul_ponticello" | "cello" => synthcraft_core::DdspTimbre::CelloSulPonticello,
        _ => synthcraft_core::DdspTimbre::SoloViolinBow,
    }
}

fn parse_chord_mode(s: &str) -> synthcraft_core::ChordMode {
    match s.trim().to_ascii_lowercase().as_str() {
        "off" => synthcraft_core::ChordMode::Off,
        "power_5th" | "5th" => synthcraft_core::ChordMode::Power5th,
        "minor_triad" | "minor" => synthcraft_core::ChordMode::MinorTriad,
        "major_triad" | "major" => synthcraft_core::ChordMode::MajorTriad,
        "minor_7th" | "m7" => synthcraft_core::ChordMode::Minor7th,
        "major_7th" | "maj7" => synthcraft_core::ChordMode::Major7th,
        "dub_techno_m11" | "m11" => synthcraft_core::ChordMode::DubTechnoM11,
        "techno_sus4" | "sus4" => synthcraft_core::ChordMode::TechnoSus4,
        "vienna_baroque" | "baroque" => synthcraft_core::ChordMode::ViennaBaroque64,
        _ => synthcraft_core::ChordMode::HouseStabM9,
    }
}

fn parse_scale_lock(s: &str) -> synthcraft_core::ScaleLock {
    match s.trim().to_ascii_lowercase().as_str() {
        "chromatic" | "off" => synthcraft_core::ScaleLock::Chromatic,
        "natural_minor" | "minor" => synthcraft_core::ScaleLock::NaturalMinor,
        "major" => synthcraft_core::ScaleLock::Major,
        "phrygian" => synthcraft_core::ScaleLock::Phrygian,
        "harmonic_minor" => synthcraft_core::ScaleLock::HarmonicMinor,
        "hirajoshi" => synthcraft_core::ScaleLock::Hirajoshi,
        "blues" => synthcraft_core::ScaleLock::Blues,
        _ => synthcraft_core::ScaleLock::Dorian,
    }
}

fn run_genopatch_tool(
    engine: &mut SynthEngine,
    wav_path: Option<&str>,
    generations: usize,
    population: usize,
    apply_best: bool,
    policy: &PathPolicy,
) -> Result<Value, synthcraft_core::SynthError> {
    let seed = engine.active_patch().clone();
    let (target_name, samples, sr) = if let Some(wp) = wav_path {
        let resolved = if policy.is_active() {
            policy.check_read(wp)?
        } else {
            PathBuf::from(wp)
        };
        let (s, sr) = synthcraft_core::load_wav_mono_samples(&resolved)
            .map_err(synthcraft_core::SynthError::Validation)?;
        (wp.to_string(), s, sr)
    } else {
        let s = synthcraft_core::synthesize_audio_buffer(&seed, engine.sample_rate_hz, 160);
        (seed.name.clone(), s, engine.sample_rate_hz)
    };
    let report = synthcraft_core::evolve_matching_patches(
        &target_name,
        &samples,
        &seed,
        sr,
        generations,
        population,
    );
    if apply_best {
        if let Some(best) = report.candidates.first() {
            Command::ApplyEvolvedPatch {
                patch: Box::new(best.patch.clone()),
            }
            .dispatch(engine)?;
        }
    }
    let summary: Vec<Value> = report
        .candidates
        .iter()
        .map(|c| {
            json!({
                "branch_id": c.branch_id,
                "label": c.label,
                "similarity_pct": c.similarity_pct,
                "spectral_distance": c.spectral_distance,
            })
        })
        .collect();
    Ok(json!({
        "target_name": report.target_name,
        "generations": report.generations,
        "population_size": report.population_size,
        "threads_used": report.threads_used,
        "total_evaluations": report.total_evaluations,
        "elapsed_ms": report.elapsed_ms,
        "applied_best": apply_best,
        "candidates": summary,
    }))
}

fn forge_and_load_djcraft(
    engine: &SynthEngine,
    deck: &str,
    bars: usize,
    output_path: &Path,
) -> Result<Value, synthcraft_core::SynthError> {
    let p = engine.active_patch();
    let dur_ms = synthcraft_core::stem_duration_ms(p, bars.clamp(1, 16));
    let samples = synthcraft_core::write_wav_file(p, output_path, engine.sample_rate_hz, dur_ms)?;
    let ipc_resp = synthcraft_core::send_stem_to_djcraft_deck(deck, output_path)?;
    Ok(json!({
        "deck": deck,
        "bars": bars.clamp(1, 16),
        "bpm": p.bpm,
        "camelot_key": p.camelot_key,
        "samples": samples,
        "output_path": output_path.to_string_lossy(),
        "djcraft_ipc": ipc_resp,
    }))
}

const WORKLET_JS: &str = include_str!("../../synthcraft-wasm/web/synthcraft-worklet.js");

fn clap_host_status_json(engine: &SynthEngine) -> Value {
    let modules: Vec<&str> = engine
        .rack_modules
        .iter()
        .filter(|m| m.kind == synthcraft_core::RackModuleKind::ClapHost)
        .map(|m| m.id.as_str())
        .collect();
    json!({
        "config": engine.clap_host,
        "rack_slots": modules,
    })
}

fn list_wavetables_json(engine: &SynthEngine) -> Value {
    let builtins: Vec<Value> = synthcraft_core::Wavetable::BUILTINS
        .iter()
        .map(|(id, desc)| json!({ "id": id, "description": desc }))
        .collect();
    json!({
        "active_wavetable": engine.active_patch().wavetable,
        "vco1_wave": engine.active_patch().vco1_wave.as_str(),
        "vco2_wave": engine.active_patch().vco2_wave.as_str(),
        "builtins": builtins,
    })
}

fn record_seq_notes(
    engine: &mut SynthEngine,
    notes: &[u8],
    start_step: usize,
    velocity: f32,
) -> Result<Value, synthcraft_core::SynthError> {
    if notes.is_empty() {
        return Err(synthcraft_core::SynthError::Validation(
            "notes list must not be empty".into(),
        ));
    }
    let p = engine.active_patch_mut();
    let len = p.steps.len();
    if len == 0 {
        return Err(synthcraft_core::SynthError::Validation(
            "active patch has no sequencer steps".into(),
        ));
    }
    let vel = velocity.clamp(0.05, 1.0);
    let mut written = 0usize;
    for (i, &note) in notes.iter().enumerate().take(len) {
        let idx = (start_step + i) % len;
        if let Some(step) = p.steps.get_mut(idx) {
            step.midi_note = note.clamp(0, 127);
            step.active = true;
            step.velocity = vel;
            step.accent = vel >= 0.88;
            written += 1;
        }
    }
    Ok(json!({
        "recorded_steps": written,
        "start_step": start_step % len,
        "velocity": vel,
        "steps": p.steps,
    }))
}

fn export_wasm_bundle(output_dir: &Path) -> Result<Value, synthcraft_core::SynthError> {
    std::fs::create_dir_all(output_dir)
        .map_err(|e| synthcraft_core::SynthError::Io(format!("create_dir_all failed: {e}")))?;
    let js_path = output_dir.join("synthcraft-worklet.js");
    std::fs::write(&js_path, WORKLET_JS)
        .map_err(|e| synthcraft_core::SynthError::Io(format!("write worklet js failed: {e}")))?;

    // Copy compiled `synthcraft_wasm.wasm` if present next to the binary or in target/.
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("synthcraft_wasm.wasm"));
        }
    }
    candidates.push(PathBuf::from(
        "target/wasm32-unknown-unknown/release/synthcraft_wasm.wasm",
    ));
    let copied_wasm = candidates
        .into_iter()
        .find(|p| p.is_file())
        .and_then(|src| {
            let dst = output_dir.join("synthcraft_wasm.wasm");
            std::fs::copy(&src, &dst)
                .ok()
                .map(|bytes| (dst.to_string_lossy().into_owned(), bytes))
        });

    Ok(json!({
        "worklet_js": js_path.to_string_lossy(),
        "wasm_copied": copied_wasm.as_ref().map(|(p, _)| p.as_str()),
        "wasm_bytes": copied_wasm.as_ref().map(|(_, b)| *b),
    }))
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
        assert_eq!(tools, 60);
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
