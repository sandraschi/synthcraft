//! Live audio output (cpal) for the desktop studio.
//!
//! The UI thread owns the `SynthEngine` patch; the audio callback owns a `SynthRt`. They talk
//! through a channel of small messages. Patches travel in a `Box` that is swapped in place, so
//! the callback never frees or allocates a patch; the old patch goes back to the UI thread to
//! be dropped there. The callback also copies finished audio into a short ring for the scope.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, FromSample, SampleFormat, SizedSample, StreamConfig};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use synthcraft_core::sampler::SampleBank;
use synthcraft_core::{ExternalClapProcessor, RackRouting, RtOptions, SynthPatch, SynthRt};

pub const SCOPE_LEN: usize = 8192;

/// Messages from the UI and MIDI threads to the audio callback.
pub enum AudioMsg {
    NoteOn(u8, f32),
    NoteOff(u8),
    AllNotesOff,
    PitchBend(f32),
    ModWheel(f32),
    Play(bool),
    Patch(Box<SynthPatch>),
    Options(RtOptions),
    Routing(RackRouting),
    /// Install a decoded sample bank. The bank cache keeps a second reference, so dropping
    /// the old one here never frees memory on the audio thread.
    Bank(Option<Arc<SampleBank>>),
    /// Install or unload an external 3U CLAP Host plugin processor (old one is returned to UI thread).
    ClapHost(Option<Box<dyn ExternalClapProcessor>>, Option<u32>),
    /// Update the CLAP parameter ID modulated by the 3U `PARAM CV` jack.
    ClapHostCvParam(Option<u32>),
}

/// Shared read-only view of what the callback is doing, for meters and the scope.
pub struct Meters {
    step: AtomicU32,
    voices: AtomicU32,
    playing: AtomicBool,
    ring: Mutex<Ring>,
}

struct Ring {
    buf: Vec<f32>,
    pos: usize,
}

impl Meters {
    fn new() -> Self {
        Self {
            step: AtomicU32::new(0),
            voices: AtomicU32::new(0),
            playing: AtomicBool::new(false),
            ring: Mutex::new(Ring {
                buf: vec![0.0; SCOPE_LEN],
                pos: 0,
            }),
        }
    }

    pub fn step(&self) -> usize {
        self.step.load(Ordering::Relaxed) as usize
    }

    pub fn voices(&self) -> usize {
        self.voices.load(Ordering::Relaxed) as usize
    }

    pub fn playing(&self) -> bool {
        self.playing.load(Ordering::Relaxed)
    }

    /// The most recent `n` samples, oldest first.
    pub fn recent(&self, n: usize) -> Vec<f32> {
        let Ok(ring) = self.ring.lock() else {
            return Vec::new();
        };
        let n = n.min(ring.buf.len());
        let start = (ring.pos + ring.buf.len() - n) % ring.buf.len();
        (0..n)
            .map(|i| ring.buf[(start + i) % ring.buf.len()])
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct AudioConfigInfo {
    pub device: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_frames: Option<u32>,
}

pub struct AudioHandle {
    _stream: cpal::Stream,
    tx: Sender<AudioMsg>,
    returned: Receiver<Box<SynthPatch>>,
    returned_clap: Receiver<Box<dyn ExternalClapProcessor>>,
    pub meters: Arc<Meters>,
    pub info: AudioConfigInfo,
    last_error: Arc<Mutex<Option<String>>>,
}

/// Output devices the default host can see, as display names.
pub fn list_output_devices() -> Vec<String> {
    let host = cpal::default_host();
    let Ok(devices) = host.output_devices() else {
        return Vec::new();
    };
    devices
        .filter_map(|d| d.description().ok().map(|desc| desc.name().to_string()))
        .collect()
}

fn find_device(name: Option<&str>) -> Option<cpal::Device> {
    let host = cpal::default_host();
    if let Some(want) = name.filter(|n| !n.is_empty()) {
        if let Ok(devices) = host.output_devices() {
            for d in devices {
                if d.description().map(|x| x.name() == want).unwrap_or(false) {
                    return Some(d);
                }
            }
        }
    }
    host.default_output_device()
}

impl AudioHandle {
    pub fn sender(&self) -> Sender<AudioMsg> {
        self.tx.clone()
    }

    pub fn send(&self, msg: AudioMsg) {
        let _ = self.tx.send(msg);
    }

    /// Drop patches and unloaded CLAP hosts the callback has handed back (keeps deallocation off the audio thread).
    pub fn reclaim(&self) {
        while self.returned.try_recv().is_ok() {}
        while self.returned_clap.try_recv().is_ok() {}
    }

    pub fn take_error(&self) -> Option<String> {
        self.last_error.lock().ok().and_then(|mut e| e.take())
    }

    /// Open the chosen output device and start the callback.
    pub fn start(
        device_name: Option<&str>,
        sample_rate: Option<u32>,
        buffer_frames: Option<u32>,
        patch: SynthPatch,
        options: RtOptions,
    ) -> Result<Self, String> {
        let device = find_device(device_name).ok_or("no audio output device found")?;
        let dev_name = device
            .description()
            .map(|d| d.name().to_string())
            .unwrap_or_else(|_| "output".into());
        let supported = device
            .default_output_config()
            .map_err(|e| format!("cannot query output config: {e}"))?;
        let format = supported.sample_format();
        let mut config: StreamConfig = supported.into();
        if let Some(sr) = sample_rate {
            config.sample_rate = sr;
        }
        if let Some(b) = buffer_frames {
            config.buffer_size = BufferSize::Fixed(b);
        }

        // If the requested rate or buffer is refused, fall back to the device defaults.
        match Self::build(&device, &dev_name, format, config, &patch, options) {
            Ok(h) => Ok(h),
            Err(first) => {
                let fallback = device
                    .default_output_config()
                    .map_err(|e| format!("{first}; default config also failed: {e}"))?;
                let fmt = fallback.sample_format();
                Self::build(&device, &dev_name, fmt, fallback.into(), &patch, options)
                    .map_err(|e| format!("{first}; default config also failed: {e}"))
            }
        }
    }

    fn build(
        device: &cpal::Device,
        dev_name: &str,
        format: SampleFormat,
        config: StreamConfig,
        patch: &SynthPatch,
        options: RtOptions,
    ) -> Result<Self, String> {
        let (tx, rx) = channel::<AudioMsg>();
        let (ret_tx, returned) = channel::<Box<SynthPatch>>();
        let (ret_clap_tx, returned_clap) = channel::<Box<dyn ExternalClapProcessor>>();
        let meters = Arc::new(Meters::new());
        let last_error = Arc::new(Mutex::new(None));
        let info = AudioConfigInfo {
            device: dev_name.to_string(),
            sample_rate: config.sample_rate,
            channels: config.channels,
            buffer_frames: match config.buffer_size {
                BufferSize::Fixed(n) => Some(n),
                BufferSize::Default => None,
            },
        };
        let mut rt = SynthRt::new(config.sample_rate as f32, patch.clone());
        rt.set_options(options);
        let stream = match format {
            SampleFormat::F32 => Self::stream::<f32>(
                device,
                config,
                rt,
                rx,
                ret_tx,
                ret_clap_tx,
                &meters,
                &last_error,
            ),
            SampleFormat::I16 => Self::stream::<i16>(
                device,
                config,
                rt,
                rx,
                ret_tx,
                ret_clap_tx,
                &meters,
                &last_error,
            ),
            SampleFormat::I32 => Self::stream::<i32>(
                device,
                config,
                rt,
                rx,
                ret_tx,
                ret_clap_tx,
                &meters,
                &last_error,
            ),
            SampleFormat::U16 => Self::stream::<u16>(
                device,
                config,
                rt,
                rx,
                ret_tx,
                ret_clap_tx,
                &meters,
                &last_error,
            ),
            other => Err(format!("unsupported sample format {other}")),
        }?;
        stream
            .play()
            .map_err(|e| format!("cannot start stream: {e}"))?;
        Ok(Self {
            _stream: stream,
            tx,
            returned,
            returned_clap,
            meters,
            info,
            last_error,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn stream<T>(
        device: &cpal::Device,
        config: StreamConfig,
        mut rt: SynthRt,
        rx: Receiver<AudioMsg>,
        ret_tx: Sender<Box<SynthPatch>>,
        ret_clap_tx: Sender<Box<dyn ExternalClapProcessor>>,
        meters: &Arc<Meters>,
        last_error: &Arc<Mutex<Option<String>>>,
    ) -> Result<cpal::Stream, String>
    where
        T: SizedSample + FromSample<f32>,
    {
        let channels = usize::from(config.channels.max(1));
        let meters_cb = Arc::clone(meters);
        let err_slot = Arc::clone(last_error);
        let mut mono = vec![0.0_f32; 4096];
        let mut right_buf = vec![0.0_f32; 4096];
        device
            .build_output_stream(
                config,
                move |data: &mut [T], _| {
                    while let Ok(msg) = rx.try_recv() {
                        match msg {
                            AudioMsg::NoteOn(n, v) => rt.note_on(n, v),
                            AudioMsg::NoteOff(n) => rt.note_off(n),
                            AudioMsg::AllNotesOff => rt.all_notes_off(),
                            AudioMsg::PitchBend(s) => rt.pitch_bend(s),
                            AudioMsg::ModWheel(v) => rt.set_mod_wheel(v),
                            AudioMsg::Play(p) => rt.set_playing(p),
                            AudioMsg::Options(o) => rt.set_options(o),
                            AudioMsg::Routing(r) => rt.set_routing(r),
                            AudioMsg::Bank(b) => {
                                let _ = rt.set_bank(b);
                            }
                            AudioMsg::Patch(mut boxed) => {
                                rt.swap_patch(&mut boxed);
                                let _ = ret_tx.send(boxed);
                            }
                            AudioMsg::ClapHost(host, cv_param) => {
                                rt.set_clap_host_cv_param(cv_param);
                                if let Some(old) = rt.set_clap_host(host) {
                                    let _ = ret_clap_tx.send(old);
                                }
                            }
                            AudioMsg::ClapHostCvParam(cv_param) => {
                                rt.set_clap_host_cv_param(cv_param);
                            }
                        }
                    }
                    let frames = data.len() / channels;
                    let mut done = 0;
                    while done < frames {
                        let n = (frames - done).min(mono.len());
                        rt.process_stereo(&mut mono[..n], &mut right_buf[..n]);
                        for (i, (&l, &r)) in mono[..n].iter().zip(&right_buf[..n]).enumerate() {
                            let base = (done + i) * channels;
                            for (c, slot) in data[base..base + channels].iter_mut().enumerate() {
                                *slot = T::from_sample(if c % 2 == 0 { l } else { r });
                            }
                        }
                        for (m, &r) in mono[..n].iter_mut().zip(&right_buf[..n]) {
                            *m = 0.5 * (*m + r);
                        }
                        if let Ok(mut ring) = meters_cb.ring.try_lock() {
                            let len = ring.buf.len();
                            for &s in &mono[..n] {
                                let p = ring.pos;
                                ring.buf[p] = s;
                                ring.pos = (p + 1) % len;
                            }
                        }
                        done += n;
                    }
                    meters_cb
                        .step
                        .store(rt.current_step() as u32, Ordering::Relaxed);
                    meters_cb
                        .voices
                        .store(rt.active_voices() as u32, Ordering::Relaxed);
                    meters_cb.playing.store(rt.is_playing(), Ordering::Relaxed);
                },
                move |err| {
                    if let Ok(mut slot) = err_slot.lock() {
                        *slot = Some(err.to_string());
                    }
                },
                None,
            )
            .map_err(|e| format!("cannot open stream: {e}"))
    }
}
