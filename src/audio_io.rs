use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;
use rustfft::{FftPlanner, num_complex::Complex};

use crate::rds::RdsGenerator;

const INTERNAL_SAMPLE_RATE: u32 = 228_000;
const OUTPUT_SAMPLE_RATE: u32 = 192_000;
const SPECTRUM_BANDS: usize = 48;
const SPECTRUM_BINS: usize = 256;
const SPECTRUM_MIN_DB: f32 = -60.0;
const SPECTRUM_MAX_DB: f32 = 0.0;

const FIR_HALF_SIZE: usize = 30;
const FIR_SIZE: usize = 2 * FIR_HALF_SIZE - 1;

const CARRIER_38: [f32; 6] = [
    0.0,
    0.8660254037844386,
    0.8660254037844388,
    1.2246467991473532e-16,
    -0.8660254037844384,
    -0.8660254037844386,
];

const CARRIER_19: [f32; 12] = [
    0.0,
    0.5,
    0.8660254037844386,
    1.0,
    0.8660254037844388,
    0.5,
    1.2246467991473532e-16,
    -0.5,
    -0.8660254037844384,
    -1.0,
    -0.8660254037844386,
    -0.5,
];

#[derive(Clone, Copy)]
struct Frame {
    left: f32,
    right: f32,
}

struct OutputResampler {
    phase: f32,
    step: f32,
    prev: f32,
    next: f32,
    has_next: bool,
}

impl OutputResampler {
    fn new(internal_rate: u32, output_rate: u32) -> Self {
        OutputResampler {
            phase: 0.0,
            step: internal_rate as f32 / output_rate as f32,
            prev: 0.0,
            next: 0.0,
            has_next: false,
        }
    }

    fn next_sample<F>(&mut self, mut fetch: F) -> f32
    where
        F: FnMut() -> f32,
    {
        if !self.has_next {
            self.next = fetch();
            self.has_next = true;
        }

        while self.phase >= 1.0 {
            self.phase -= 1.0;
            self.prev = self.next;
            self.next = fetch();
        }

        let t = self.phase;
        let sample = self.prev + (self.next - self.prev) * t;
        self.phase += self.step;
        sample
    }
}

struct LiveMpx {
    rds: RdsGenerator,
    low_pass_fir: [f32; FIR_HALF_SIZE],
    fir_buffer_mono: [f32; FIR_SIZE],
    fir_buffer_stereo: [f32; FIR_SIZE],
    fir_index: usize,
    phase_38: usize,
    phase_19: usize,

    gain: f32,
    limiter_enabled: bool,
    limiter_threshold: f32,
    limiter_lookahead: usize,
    limiter_buffer: VecDeque<f32>,

    pilot_level: f32,
    rds_level: f32,
    stereo_separation: f32,

    preemphasis_tau: Option<f32>,
    preemph_prev_mono: f32,
    preemph_prev_stereo: f32,
    preemph_state_mono: f32,
    preemph_state_stereo: f32,

    compressor_enabled: bool,
    comp_threshold_db: f32,
    comp_ratio: f32,
    comp_attack: f32,
    comp_release: f32,
    comp_gain_db: f32,
}

impl LiveMpx {
    fn new() -> Self {
        let mut low_pass_fir = [0.0f32; FIR_HALF_SIZE];
        let cutoff_freq = 15000.0 * 0.8;

        low_pass_fir[FIR_HALF_SIZE - 1] = 2.0 * cutoff_freq / INTERNAL_SAMPLE_RATE as f32 / 2.0;

        for i in 1..FIR_HALF_SIZE {
            let idx = FIR_HALF_SIZE - 1 - i;
            let sinc = (2.0 * std::f32::consts::PI * cutoff_freq * i as f32
                / INTERNAL_SAMPLE_RATE as f32)
                .sin()
                / (std::f32::consts::PI * i as f32);
            let window = 0.54
                - 0.46
                    * (2.0 * std::f32::consts::PI * (i + FIR_HALF_SIZE) as f32
                        / (2.0 * FIR_HALF_SIZE as f32))
                        .cos();
            low_pass_fir[idx] = sinc * window;
        }

        LiveMpx {
            rds: RdsGenerator::new(),
            low_pass_fir,
            fir_buffer_mono: [0.0; FIR_SIZE],
            fir_buffer_stereo: [0.0; FIR_SIZE],
            fir_index: 0,
            phase_38: 0,
            phase_19: 0,

            gain: 1.0,
            limiter_enabled: true,
            limiter_threshold: 0.95,
            limiter_lookahead: 256,
            limiter_buffer: VecDeque::with_capacity(512),

            pilot_level: 0.9,
            rds_level: 1.0,
            stereo_separation: 1.0,

            preemphasis_tau: None,
            preemph_prev_mono: 0.0,
            preemph_prev_stereo: 0.0,
            preemph_state_mono: 0.0,
            preemph_state_stereo: 0.0,

            compressor_enabled: false,
            comp_threshold_db: -18.0,
            comp_ratio: 3.0,
            comp_attack: 0.01,
            comp_release: 0.2,
            comp_gain_db: 0.0,
        }
    }

    fn set_ps(&mut self, ps: &str) {
        self.rds.set_ps(ps);
    }

    fn set_rt(&mut self, rt: &str) {
        self.rds.set_rt(rt);
    }

    fn set_pi(&mut self, pi: u16) {
        self.rds.set_pi(pi);
    }

    fn set_tp(&mut self, tp: bool) {
        self.rds.set_tp(tp);
    }

    fn set_ta(&mut self, ta: bool) {
        self.rds.set_ta(ta);
    }

    fn set_pty(&mut self, pty: u8) {
        self.rds.set_pty(pty);
    }

    fn set_ms(&mut self, ms: bool) {
        self.rds.set_ms(ms);
    }

    fn set_di(&mut self, di: u8) {
        self.rds.set_di(di);
    }

    fn set_ab(&mut self, ab: bool) {
        self.rds.set_rt_ab(ab);
    }

    fn set_ab_auto(&mut self, ab_auto: bool) {
        self.rds.set_rt_ab_auto(ab_auto);
    }

    fn set_ct_enabled(&mut self, enabled: bool) {
        self.rds.set_ct_enabled(enabled);
    }

    fn set_af_list_mhz(&mut self, freqs: &[f32]) {
        self.rds.set_af_list_mhz(freqs);
    }

    fn set_ps_scroll(&mut self, enabled: bool, text: &str, cps: f32) {
        self.rds.enable_ps_scroll(enabled, text, cps);
    }

    fn set_rt_scroll(&mut self, enabled: bool, text: &str, cps: f32) {
        self.rds.enable_rt_scroll(enabled, text, cps);
    }

    fn set_group_mix(&mut self, count_0a: usize, count_2a: usize, count_4a: usize) {
        self.rds.set_group_mix(count_0a, count_2a, count_4a);
    }

    fn set_ct_interval(&mut self, interval_groups: usize) {
        self.rds.set_ct_interval_groups(interval_groups);
    }

    fn set_ps_alternates(&mut self, list: Vec<String>, interval_groups: usize) {
        self.rds.set_ps_alternates(list, interval_groups);
    }

    fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    fn set_limiter(&mut self, enabled: bool, threshold: f32) {
        self.limiter_enabled = enabled;
        self.limiter_threshold = threshold;
    }

    fn set_limiter_lookahead(&mut self, samples: usize) {
        self.limiter_lookahead = samples.max(1).min(2048);
        self.limiter_buffer.clear();
    }

    fn set_pilot_level(&mut self, level: f32) {
        self.pilot_level = level.clamp(0.0, 2.0);
    }

    fn set_rds_level(&mut self, level: f32) {
        self.rds_level = level.clamp(0.0, 2.0);
    }

    fn set_stereo_separation(&mut self, level: f32) {
        self.stereo_separation = level.clamp(0.0, 2.0);
    }

    fn set_preemphasis(&mut self, tau_seconds: Option<f32>) {
        self.preemphasis_tau = tau_seconds;
        self.preemph_prev_mono = 0.0;
        self.preemph_prev_stereo = 0.0;
        self.preemph_state_mono = 0.0;
        self.preemph_state_stereo = 0.0;
    }

    fn set_compressor(&mut self, enabled: bool, threshold_db: f32, ratio: f32, attack: f32, release: f32) {
        self.compressor_enabled = enabled;
        self.comp_threshold_db = threshold_db;
        self.comp_ratio = ratio.max(1.0);
        self.comp_attack = attack.max(0.001);
        self.comp_release = release.max(0.01);
        self.comp_gain_db = 0.0;
    }

    fn next_sample(&mut self, frame: Frame) -> f32 {
        let mut rds_sample = 0.0f32;
        self.rds.get_rds_samples(std::slice::from_mut(&mut rds_sample));

        let mono_sample = frame.left + frame.right;
        let stereo_sample = frame.left - frame.right;

        self.fir_buffer_mono[self.fir_index] = mono_sample;
        self.fir_buffer_stereo[self.fir_index] = stereo_sample;

        self.fir_index += 1;
        if self.fir_index >= FIR_SIZE {
            self.fir_index = 0;
        }

        let mut out_mono = 0.0;
        let mut out_stereo = 0.0;
        let mut ifbi = self.fir_index;
        let mut dfbi = self.fir_index;

        for fi in 0..FIR_HALF_SIZE {
            if dfbi == 0 {
                dfbi = FIR_SIZE - 1;
            } else {
                dfbi -= 1;
            }

            out_mono += self.low_pass_fir[fi]
                * (self.fir_buffer_mono[ifbi] + self.fir_buffer_mono[dfbi]);
            out_stereo += self.low_pass_fir[fi]
                * (self.fir_buffer_stereo[ifbi] + self.fir_buffer_stereo[dfbi]);

            ifbi += 1;
            if ifbi >= FIR_SIZE {
                ifbi = 0;
            }
        }

        let mut mono = out_mono;
        let mut stereo = out_stereo;

        if let Some(tau) = self.preemphasis_tau {
            let a = (-1.0 / (tau * INTERNAL_SAMPLE_RATE as f32)).exp();
            let y_mono = mono - self.preemph_prev_mono + a * self.preemph_state_mono;
            self.preemph_prev_mono = mono;
            self.preemph_state_mono = y_mono;
            mono = y_mono;

            let y_stereo = stereo - self.preemph_prev_stereo + a * self.preemph_state_stereo;
            self.preemph_prev_stereo = stereo;
            self.preemph_state_stereo = y_stereo;
            stereo = y_stereo;
        }

        if self.compressor_enabled {
            let level = mono.abs().max(stereo.abs()).max(1e-6);
            let level_db = 20.0 * level.log10();
            let mut target_gain_db = 0.0;
            if level_db > self.comp_threshold_db {
                let compressed = self.comp_threshold_db + (level_db - self.comp_threshold_db) / self.comp_ratio;
                target_gain_db = compressed - level_db;
            }
            let coeff = if target_gain_db < self.comp_gain_db {
                (-1.0 / (self.comp_attack * INTERNAL_SAMPLE_RATE as f32)).exp()
            } else {
                (-1.0 / (self.comp_release * INTERNAL_SAMPLE_RATE as f32)).exp()
            };
            self.comp_gain_db = target_gain_db + coeff * (self.comp_gain_db - target_gain_db);
            let gain = 10f32.powf(self.comp_gain_db / 20.0);
            mono *= gain;
            stereo *= gain;
        }

        let mut mpx = self.rds_level * rds_sample + 4.05 * mono;
        mpx += (4.05 * self.stereo_separation) * CARRIER_38[self.phase_38] * stereo
            + self.pilot_level * CARRIER_19[self.phase_19];

        self.phase_19 += 1;
        self.phase_38 += 1;
        if self.phase_19 >= CARRIER_19.len() {
            self.phase_19 = 0;
        }
        if self.phase_38 >= CARRIER_38.len() {
            self.phase_38 = 0;
        }

        let mut out = mpx * 0.1 * self.gain;
        if self.limiter_enabled {
            self.limiter_buffer.push_back(out);
            if self.limiter_buffer.len() < self.limiter_lookahead {
                return 0.0;
            }
            if self.limiter_buffer.len() > self.limiter_lookahead {
                let _ = self.limiter_buffer.pop_front();
            }
            let mut max = 0.0f32;
            for v in self.limiter_buffer.iter() {
                let a = v.abs();
                if a > max {
                    max = a;
                }
            }
            let threshold = self.limiter_threshold.max(0.1);
            let gain = if max > threshold { threshold / max } else { 1.0 };
            if let Some(sample) = self.limiter_buffer.front() {
                out = *sample * gain;
            }
        }
        out
    }
}

pub struct AudioEngine {
    _input_stream: Option<cpal::Stream>,
    _output_stream: cpal::Stream,
    shared: Arc<Mutex<LiveMpx>>,
    meter: Arc<MeterState>,
    scope: Arc<Mutex<VecDeque<f32>>>,
    spectrum: Arc<Mutex<Vec<f32>>>,
    spectrum_peak: Arc<Mutex<Vec<f32>>>,
    spectrum_avg: Arc<Mutex<Vec<f32>>>,
    xrun_count: Arc<AtomicU32>,
    buffer_fill: Arc<AtomicU32>,
    latency_ms: f32,
}

pub struct AudioEngineConfig {
    pub input_device: Option<String>,
    pub output_device: String,
    pub ps: String,
    pub rt: String,
    pub pi: u16,
    pub tp: bool,
    pub ta: bool,
    pub pty: u8,
    pub ms: bool,
    pub di: u8,
    pub ab: bool,
    pub ab_auto: bool,
    pub ct_enabled: bool,
    pub af_list_mhz: Vec<f32>,
    pub ps_scroll_enabled: bool,
    pub ps_scroll_text: String,
    pub ps_scroll_cps: f32,
    pub rt_scroll_enabled: bool,
    pub rt_scroll_text: String,
    pub rt_scroll_cps: f32,
    pub output_gain: f32,
    pub limiter_enabled: bool,
    pub limiter_threshold: f32,
    pub limiter_lookahead: usize,
    pub pilot_level: f32,
    pub rds_level: f32,
    pub stereo_separation: f32,
    pub preemphasis_tau: Option<f32>,
    pub compressor_enabled: bool,
    pub comp_threshold_db: f32,
    pub comp_ratio: f32,
    pub comp_attack: f32,
    pub comp_release: f32,
    pub group_0a: usize,
    pub group_2a: usize,
    pub group_4a: usize,
    pub ct_interval_groups: usize,
    pub ps_alt_list: Vec<String>,
    pub ps_alt_interval: usize,
}

pub struct MeterSnapshot {
    pub rms: f32,
    pub peak: f32,
    pub pilot: f32,
    pub rds: f32,
    pub bands_db: [f32; SPECTRUM_BANDS],
    pub scope: Vec<f32>,
    pub spectrum_db: Vec<f32>,
    pub spectrum_peak_db: Vec<f32>,
    pub spectrum_avg_db: Vec<f32>,
    pub xrun_count: u32,
    pub buffer_fill: f32,
    pub latency_ms: f32,
}

struct MeterState {
    rms: AtomicU32,
    peak: AtomicU32,
    pilot: AtomicU32,
    rds: AtomicU32,
    bands_db: [AtomicU32; SPECTRUM_BANDS],
}

impl MeterState {
    fn new() -> Self {
        MeterState {
            rms: AtomicU32::new(0),
            peak: AtomicU32::new(0),
            pilot: AtomicU32::new(0),
            rds: AtomicU32::new(0),
            bands_db: std::array::from_fn(|_| AtomicU32::new(f32_to_u32(SPECTRUM_MIN_DB))),
        }
    }
}

fn f32_to_u32(v: f32) -> u32 {
    v.to_bits()
}

fn u32_to_f32(v: u32) -> f32 {
    f32::from_bits(v)
}

fn db_to_unit(db: f32) -> f32 {
    let clamped = db.clamp(SPECTRUM_MIN_DB, SPECTRUM_MAX_DB);
    (clamped - SPECTRUM_MIN_DB) / (SPECTRUM_MAX_DB - SPECTRUM_MIN_DB)
}

pub fn list_input_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let mut devices = Vec::new();
    for device in host.input_devices()? {
        if let Ok(name) = device.name() {
            devices.push(name);
        }
    }
    devices.sort();
    Ok(devices)
}

pub fn list_output_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let mut devices = Vec::new();
    for device in host.output_devices()? {
        if let Ok(name) = device.name() {
            devices.push(name);
        }
    }
    devices.sort();
    Ok(devices)
}

fn find_device_by_name(devices: Vec<cpal::Device>, name: &str) -> Option<cpal::Device> {
    devices.into_iter().find(|d| d.name().map(|n| n == name).unwrap_or(false))
}

fn pick_config(
    device: &cpal::Device,
    is_input: bool,
) -> Result<cpal::SupportedStreamConfig> {
    let configs = if is_input {
        device.supported_input_configs()?.collect::<Vec<_>>()
    } else {
        device.supported_output_configs()?.collect::<Vec<_>>()
    };

    for cfg in configs {
        if cfg.sample_format() != cpal::SampleFormat::F32 {
            continue;
        }
        let min = cfg.min_sample_rate().0;
        let max = cfg.max_sample_rate().0;
        if min <= OUTPUT_SAMPLE_RATE && max >= OUTPUT_SAMPLE_RATE {
            return Ok(cfg.with_sample_rate(cpal::SampleRate(OUTPUT_SAMPLE_RATE)));
        }
    }

    Err(anyhow!("Device does not support 192 kHz float32"))
}

pub fn start_engine(config: AudioEngineConfig) -> Result<AudioEngine> {
    let host = cpal::default_host();

    let output_devices = host.output_devices()?.collect::<Vec<_>>();
    let output_device = find_device_by_name(output_devices, &config.output_device)
        .ok_or_else(|| anyhow!("Output device not found"))?;

    let output_supported = pick_config(&output_device, false)?;
    let output_config: cpal::StreamConfig = output_supported.clone().into();

    let input_device = if let Some(ref name) = config.input_device {
        let input_devices = host.input_devices()?.collect::<Vec<_>>();
        Some(find_device_by_name(input_devices, name).ok_or_else(|| anyhow!("Input device not found"))?)
    } else {
        None
    };

    let input_supported = if let Some(ref device) = input_device {
        Some(pick_config(device, true)?)
    } else {
        None
    };

    let ring = HeapRb::<Frame>::new(OUTPUT_SAMPLE_RATE as usize * 2);
    let (mut prod, mut cons) = ring.split();

    let xrun_count = Arc::new(AtomicU32::new(0));
    let buffer_fill = Arc::new(AtomicU32::new(0));

    let xrun_for_input = Arc::clone(&xrun_count);
    let fill_for_input = Arc::clone(&buffer_fill);
    let input_stream = if let (Some(device), Some(cfg)) = (input_device, input_supported) {
        let input_config: cpal::StreamConfig = cfg.clone().into();
        let channels = input_config.channels as usize;
        let err_fn = |err| eprintln!("input stream error: {}", err);
        let stream = device.build_input_stream(
            &input_config,
            move |data: &[f32], _| {
                let mut i = 0;
                while i + channels <= data.len() {
                    let left = data[i];
                    let right = if channels > 1 { data[i + 1] } else { data[i] };
                    if prod.push(Frame { left, right }).is_err() {
                        xrun_for_input.fetch_add(1, Ordering::Relaxed);
                    } else {
                        let prev = fill_for_input.load(Ordering::Relaxed);
                        fill_for_input.store(prev.saturating_add(1), Ordering::Relaxed);
                    }
                    i += channels;
                }
            },
            err_fn,
            None,
        )?;
        Some(stream)
    } else {
        None
    };

    let shared = Arc::new(Mutex::new(LiveMpx::new()));
    {
        let mut engine = shared.lock().unwrap();
        engine.set_ps(&config.ps);
        engine.set_rt(&config.rt);
        engine.set_pi(config.pi);
        engine.set_tp(config.tp);
        engine.set_ta(config.ta);
        engine.set_pty(config.pty);
        engine.set_ms(config.ms);
        engine.set_di(config.di);
        engine.set_ab(config.ab);
        engine.set_ab_auto(config.ab_auto);
        engine.set_ct_enabled(config.ct_enabled);
        engine.set_af_list_mhz(&config.af_list_mhz);
        engine.set_ps_scroll(config.ps_scroll_enabled, &config.ps_scroll_text, config.ps_scroll_cps);
        engine.set_rt_scroll(config.rt_scroll_enabled, &config.rt_scroll_text, config.rt_scroll_cps);
        engine.set_gain(config.output_gain);
        engine.set_limiter(config.limiter_enabled, config.limiter_threshold);
        engine.set_limiter_lookahead(config.limiter_lookahead);
        engine.set_pilot_level(config.pilot_level);
        engine.set_rds_level(config.rds_level);
        engine.set_stereo_separation(config.stereo_separation);
        engine.set_preemphasis(config.preemphasis_tau);
        engine.set_compressor(
            config.compressor_enabled,
            config.comp_threshold_db,
            config.comp_ratio,
            config.comp_attack,
            config.comp_release,
        );
        engine.set_group_mix(config.group_0a, config.group_2a, config.group_4a);
        engine.set_ct_interval(config.ct_interval_groups);
        engine.set_ps_alternates(config.ps_alt_list.clone(), config.ps_alt_interval);
    }

    let mut output_resampler = OutputResampler::new(INTERNAL_SAMPLE_RATE, OUTPUT_SAMPLE_RATE);

    let meter = Arc::new(MeterState::new());
    let meter_for_output = Arc::clone(&meter);
    let scope = Arc::new(Mutex::new(VecDeque::with_capacity(2048)));
    let scope_for_output = Arc::clone(&scope);

    let mut fft_planner = FftPlanner::<f32>::new();
    let fft = fft_planner.plan_fft_forward(1024);
    let mut fft_buf: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); 1024];
    let mut fft_pos: usize = 0;
    let spectrum = Arc::new(Mutex::new(vec![SPECTRUM_MIN_DB; SPECTRUM_BINS]));
    let spectrum_peak = Arc::new(Mutex::new(vec![SPECTRUM_MIN_DB; SPECTRUM_BINS]));
    let spectrum_avg = Arc::new(Mutex::new(vec![SPECTRUM_MIN_DB; SPECTRUM_BINS]));
    let spectrum_for_output = Arc::clone(&spectrum);
    let spectrum_peak_for_output = Arc::clone(&spectrum_peak);
    let spectrum_avg_for_output = Arc::clone(&spectrum_avg);

    let err_fn = |err| eprintln!("output stream error: {}", err);
    let xrun_for_output = Arc::clone(&xrun_count);
    let fill_for_output = Arc::clone(&buffer_fill);
    let latency_ms = match output_config.buffer_size {
        cpal::BufferSize::Fixed(frames) => frames as f32 / OUTPUT_SAMPLE_RATE as f32 * 1000.0,
        cpal::BufferSize::Default => 0.0,
    };
    let output_channels = output_config.channels as usize;
    let shared_for_output = Arc::clone(&shared);
    let output_stream = output_device.build_output_stream(
        &output_config,
        move |data: &mut [f32], _| {
            let mut engine = shared_for_output.lock().unwrap();
            let mut index = 0;
            let mut sum_sq = 0.0f32;
            let mut peak = 0.0f32;
            while index + output_channels <= data.len() {
                let out = output_resampler.next_sample(|| {
                    let frame = match cons.pop() {
                        Some(f) => {
                            let prev = fill_for_output.load(Ordering::Relaxed);
                            fill_for_output.store(prev.saturating_sub(1), Ordering::Relaxed);
                            f
                        }
                        None => {
                            xrun_for_output.fetch_add(1, Ordering::Relaxed);
                            Frame { left: 0.0, right: 0.0 }
                        }
                    };
                    engine.next_sample(frame)
                });
                for ch in 0..output_channels {
                    data[index + ch] = out;
                }
                sum_sq += out * out;
                if out.abs() > peak {
                    peak = out.abs();
                }

                fft_buf[fft_pos].re = out;
                fft_buf[fft_pos].im = 0.0;
                fft_pos += 1;
                if fft_pos >= fft_buf.len() {
                    fft_pos = 0;
                    let mut windowed = fft_buf.clone();
                    let window_len = windowed.len() as f32;
                    for (i, v) in windowed.iter_mut().enumerate() {
                        let w = 0.5 - 0.5 * ((2.0 * std::f32::consts::PI * i as f32) / window_len).cos();
                        v.re *= w;
                    }
                    fft.process(&mut windowed);
                    let mut bands = [SPECTRUM_MIN_DB; SPECTRUM_BANDS];
                    let mut pilot = 0.0f32;
                    let mut rds = 0.0f32;
                    let n = windowed.len() as f32;
                    let mut spec = vec![SPECTRUM_MIN_DB; SPECTRUM_BINS];
                    for (k, v) in windowed.iter().enumerate().take(windowed.len() / 2) {
                        let freq = k as f32 * OUTPUT_SAMPLE_RATE as f32 / n;
                        let mag = (v.re * v.re + v.im * v.im).sqrt() / n;
                        let db = 20.0 * (mag + 1e-9).log10();
                        let unit = db_to_unit(db);
                        if (freq - 19000.0).abs() < 100.0 {
                            pilot = pilot.max(unit);
                        }
                        if (freq - 57000.0).abs() < 150.0 {
                            rds = rds.max(unit);
                        }
                        if k < SPECTRUM_BINS {
                            spec[k] = db;
                        }
                        let band = ((freq / (OUTPUT_SAMPLE_RATE as f32 / 2.0)) * SPECTRUM_BANDS as f32)
                            .floor() as usize;
                        if band < SPECTRUM_BANDS && db > bands[band] {
                            bands[band] = db;
                        }
                    }
                    meter_for_output.pilot.store(f32_to_u32(pilot), Ordering::Relaxed);
                    meter_for_output.rds.store(f32_to_u32(rds), Ordering::Relaxed);
                    for i in 0..SPECTRUM_BANDS {
                        meter_for_output.bands_db[i].store(f32_to_u32(bands[i]), Ordering::Relaxed);
                    }
                    if let Ok(mut spectrum_guard) = spectrum_for_output.lock() {
                        *spectrum_guard = spec.clone();
                    }
                    if let Ok(mut peak_guard) = spectrum_peak_for_output.lock() {
                        if peak_guard.len() != spec.len() {
                            *peak_guard = spec.clone();
                        } else {
                            for i in 0..spec.len() {
                                peak_guard[i] = peak_guard[i].max(spec[i]);
                            }
                        }
                    }
                    if let Ok(mut avg_guard) = spectrum_avg_for_output.lock() {
                        if avg_guard.len() != spec.len() {
                            *avg_guard = spec.clone();
                        } else {
                            for i in 0..spec.len() {
                                avg_guard[i] = avg_guard[i] * 0.9 + spec[i] * 0.1;
                            }
                        }
                    }
                }
                index += output_channels;
            }
            let rms = (sum_sq / (data.len() as f32 / output_channels as f32)).sqrt();
            meter_for_output.rms.store(f32_to_u32(rms), Ordering::Relaxed);
            meter_for_output.peak.store(f32_to_u32(peak), Ordering::Relaxed);

            if let Ok(mut scope_buf) = scope_for_output.lock() {
                for &sample in data.iter().step_by(output_channels) {
                    if scope_buf.len() >= 2048 {
                        scope_buf.pop_front();
                    }
                    scope_buf.push_back(sample);
                }
            }
        },
        err_fn,
        None,
    )?;

    if let Some(ref stream) = input_stream {
        stream.play()?;
    }
    output_stream.play()?;

    Ok(AudioEngine {
        _input_stream: input_stream,
        _output_stream: output_stream,
        shared,
        meter,
        scope,
        spectrum,
        spectrum_peak,
        spectrum_avg,
        xrun_count,
        buffer_fill,
        latency_ms,
    })
}

impl AudioEngine {
    pub fn meter_snapshot(&self) -> MeterSnapshot {
        let mut bands = [0.0f32; SPECTRUM_BANDS];
        for i in 0..SPECTRUM_BANDS {
            bands[i] = u32_to_f32(self.meter.bands_db[i].load(Ordering::Relaxed));
        }
        let scope = self.scope.lock().map(|buf| buf.iter().copied().collect()).unwrap_or_default();
        let spectrum = self.spectrum.lock().map(|v| v.clone()).unwrap_or_default();
        let spectrum_peak = self.spectrum_peak.lock().map(|v| v.clone()).unwrap_or_default();
        let spectrum_avg = self.spectrum_avg.lock().map(|v| v.clone()).unwrap_or_default();
        MeterSnapshot {
            rms: u32_to_f32(self.meter.rms.load(Ordering::Relaxed)),
            peak: u32_to_f32(self.meter.peak.load(Ordering::Relaxed)),
            pilot: u32_to_f32(self.meter.pilot.load(Ordering::Relaxed)),
            rds: u32_to_f32(self.meter.rds.load(Ordering::Relaxed)),
            bands_db: bands,
            scope,
            spectrum_db: spectrum,
            spectrum_peak_db: spectrum_peak,
            spectrum_avg_db: spectrum_avg,
            xrun_count: self.xrun_count.load(Ordering::Relaxed),
            buffer_fill: self.buffer_fill.load(Ordering::Relaxed) as f32 / (OUTPUT_SAMPLE_RATE as f32 * 2.0),
            latency_ms: self.latency_ms,
        }
    }

    pub fn update_ps(&self, ps: &str) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_ps(ps);
        }
    }

    pub fn update_rt(&self, rt: &str) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_rt(rt);
        }
    }

    pub fn update_pi(&self, pi: u16) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_pi(pi);
        }
    }

    pub fn update_tp(&self, tp: bool) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_tp(tp);
        }
    }

    pub fn update_ta(&self, ta: bool) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_ta(ta);
        }
    }

    pub fn update_pty(&self, pty: u8) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_pty(pty);
        }
    }

    pub fn update_ms(&self, ms: bool) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_ms(ms);
        }
    }

    pub fn update_di(&self, di: u8) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_di(di);
        }
    }

    pub fn update_ab(&self, ab: bool) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_ab(ab);
        }
    }

    pub fn update_ab_auto(&self, ab_auto: bool) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_ab_auto(ab_auto);
        }
    }

    pub fn update_ct_enabled(&self, enabled: bool) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_ct_enabled(enabled);
        }
    }

    pub fn update_af_list(&self, freqs: &[f32]) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_af_list_mhz(freqs);
        }
    }

    pub fn update_ps_scroll(&self, enabled: bool, text: &str, cps: f32) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_ps_scroll(enabled, text, cps);
        }
    }

    pub fn update_rt_scroll(&self, enabled: bool, text: &str, cps: f32) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_rt_scroll(enabled, text, cps);
        }
    }

    pub fn update_gain(&self, gain: f32) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_gain(gain);
        }
    }

    pub fn update_limiter(&self, enabled: bool, threshold: f32) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_limiter(enabled, threshold);
        }
    }

    pub fn update_limiter_lookahead(&self, samples: usize) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_limiter_lookahead(samples);
        }
    }

    pub fn update_pilot_level(&self, level: f32) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_pilot_level(level);
        }
    }

    pub fn update_rds_level(&self, level: f32) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_rds_level(level);
        }
    }

    pub fn update_stereo_separation(&self, level: f32) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_stereo_separation(level);
        }
    }

    pub fn update_preemphasis(&self, tau: Option<f32>) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_preemphasis(tau);
        }
    }

    pub fn update_compressor(&self, enabled: bool, threshold_db: f32, ratio: f32, attack: f32, release: f32) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_compressor(enabled, threshold_db, ratio, attack, release);
        }
    }

    pub fn update_group_mix(&self, count_0a: usize, count_2a: usize, count_4a: usize) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_group_mix(count_0a, count_2a, count_4a);
        }
    }

    pub fn update_ct_interval(&self, interval_groups: usize) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_ct_interval(interval_groups);
        }
    }

    pub fn update_ps_alternates(&self, list: Vec<String>, interval_groups: usize) {
        if let Ok(mut engine) = self.shared.lock() {
            engine.set_ps_alternates(list, interval_groups);
        }
    }
}
