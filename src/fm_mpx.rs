use anyhow::Result;

use crate::audio::AudioSource;
use crate::rds::RdsGenerator;

const PI: f32 = 3.141592654;
const MPX_SAMPLE_RATE: f32 = 228000.0;

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

pub struct FmMpx {
    pub rds: RdsGenerator,

    audio: Option<AudioSource>,
    downsample_factor: f32,
    audio_pos: f32,
    audio_index: usize,

    low_pass_fir: [f32; FIR_HALF_SIZE],
    fir_buffer_mono: [f32; FIR_SIZE],
    fir_buffer_stereo: [f32; FIR_SIZE],
    fir_index: usize,

    channels: usize,
    phase_38: usize,
    phase_19: usize,

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

impl FmMpx {
    pub fn new(audio: Option<AudioSource>) -> Self {
        let mut low_pass_fir = [0.0f32; FIR_HALF_SIZE];

        let (downsample_factor, channels) = if let Some(ref audio) = audio {
            let in_samplerate = audio.sample_rate as f32;
            let downsample_factor = MPX_SAMPLE_RATE / in_samplerate;

            let mut cutoff_freq = 15000.0 * 0.8;
            if in_samplerate / 2.0 < cutoff_freq {
                cutoff_freq = (in_samplerate / 2.0) * 0.8;
            }

            low_pass_fir[FIR_HALF_SIZE - 1] = 2.0 * cutoff_freq / MPX_SAMPLE_RATE / 2.0;

            for i in 1..FIR_HALF_SIZE {
                let idx = FIR_HALF_SIZE - 1 - i;
                let sinc = (2.0 * PI * cutoff_freq * i as f32 / MPX_SAMPLE_RATE).sin()
                    / (PI * i as f32);
                let window = 0.54 - 0.46 * (2.0 * PI * (i + FIR_HALF_SIZE) as f32
                    / (2.0 * FIR_HALF_SIZE as f32))
                    .cos();
                low_pass_fir[idx] = sinc * window;
            }

            (downsample_factor, audio.channels)
        } else {
            (1.0, 0)
        };

        FmMpx {
            rds: RdsGenerator::new(),
            audio,
            downsample_factor,
            audio_pos: downsample_factor,
            audio_index: 0,
            low_pass_fir,
            fir_buffer_mono: [0.0; FIR_SIZE],
            fir_buffer_stereo: [0.0; FIR_SIZE],
            fir_index: 0,
            channels,
            phase_38: 0,
            phase_19: 0,

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

    pub fn set_rds_ps(&mut self, ps: &str) {
        self.rds.set_ps(ps);
    }

    pub fn set_rds_rt(&mut self, rt: &str) {
        self.rds.set_rt(rt);
    }

    pub fn set_rds_pi(&mut self, pi: u16) {
        self.rds.set_pi(pi);
    }

    pub fn set_rds_tp(&mut self, tp: bool) {
        self.rds.set_tp(tp);
    }

    pub fn set_rds_ta(&mut self, ta: bool) {
        self.rds.set_ta(ta);
    }

    pub fn set_rds_pty(&mut self, pty: u8) {
        self.rds.set_pty(pty);
    }

    pub fn set_rds_ms(&mut self, ms: bool) {
        self.rds.set_ms(ms);
    }

    pub fn set_rds_di(&mut self, di: u8) {
        self.rds.set_di(di);
    }

    pub fn set_rds_ab(&mut self, ab: bool) {
        self.rds.set_rt_ab(ab);
    }

    pub fn set_rds_ab_auto(&mut self, ab_auto: bool) {
        self.rds.set_rt_ab_auto(ab_auto);
    }

    pub fn set_rds_ct_enabled(&mut self, enabled: bool) {
        self.rds.set_ct_enabled(enabled);
    }

    pub fn set_pilot_level(&mut self, level: f32) {
        self.pilot_level = level.clamp(0.0, 2.0);
    }

    pub fn set_rds_level(&mut self, level: f32) {
        self.rds_level = level.clamp(0.0, 2.0);
    }

    pub fn set_stereo_separation(&mut self, level: f32) {
        self.stereo_separation = level.clamp(0.0, 2.0);
    }

    pub fn set_preemphasis(&mut self, tau: Option<f32>) {
        self.preemphasis_tau = tau;
        self.preemph_prev_mono = 0.0;
        self.preemph_prev_stereo = 0.0;
        self.preemph_state_mono = 0.0;
        self.preemph_state_stereo = 0.0;
    }

    pub fn set_compressor(&mut self, enabled: bool, threshold_db: f32, ratio: f32, attack: f32, release: f32) {
        self.compressor_enabled = enabled;
        self.comp_threshold_db = threshold_db;
        self.comp_ratio = ratio.max(1.0);
        self.comp_attack = attack.max(0.001);
        self.comp_release = release.max(0.01);
        self.comp_gain_db = 0.0;
    }

    pub fn set_rds_af_list(&mut self, freqs: &[f32]) {
        self.rds.set_af_list_mhz(freqs);
    }

    pub fn set_rds_ps_scroll(&mut self, enabled: bool, text: &str, cps: f32) {
        self.rds.enable_ps_scroll(enabled, text, cps);
    }

    pub fn set_rds_rt_scroll(&mut self, enabled: bool, text: &str, cps: f32) {
        self.rds.enable_rt_scroll(enabled, text, cps);
    }

    pub fn set_rds_group_mix(&mut self, count_0a: usize, count_2a: usize, count_4a: usize) {
        self.rds.set_group_mix(count_0a, count_2a, count_4a);
    }

    pub fn set_rds_ct_interval(&mut self, interval_groups: usize) {
        self.rds.set_ct_interval_groups(interval_groups);
    }

    pub fn set_rds_ps_alternates(&mut self, list: Vec<String>, interval_groups: usize) {
        self.rds.set_ps_alternates(list, interval_groups);
    }

    pub fn get_samples(&mut self, mpx_buffer: &mut [f32]) -> Result<()> {
        self.rds.get_rds_samples(mpx_buffer);
        if (self.rds_level - 1.0).abs() > f32::EPSILON {
            for v in mpx_buffer.iter_mut() {
                *v *= self.rds_level;
            }
        }

        if self.audio.is_none() {
            return Ok(());
        }

        let audio = self.audio.as_ref().unwrap();
        let total_samples = audio.samples.len();
        let channels = self.channels;

        for i in 0..mpx_buffer.len() {
            if self.audio_pos >= self.downsample_factor {
                self.audio_pos -= self.downsample_factor;
                if total_samples > 0 {
                    self.audio_index = (self.audio_index + channels) % total_samples;
                }
            }

            let mono_sample;
            let stereo_sample;
            if channels <= 1 {
                mono_sample = audio.samples.get(self.audio_index).copied().unwrap_or(0.0);
                stereo_sample = 0.0;
            } else {
                let left = audio.samples.get(self.audio_index).copied().unwrap_or(0.0);
                let right = audio
                    .samples
                    .get(self.audio_index + 1)
                    .copied()
                    .unwrap_or(0.0);
                mono_sample = left + right;
                stereo_sample = left - right;
            }

            self.fir_buffer_mono[self.fir_index] = mono_sample;
            if channels > 1 {
                self.fir_buffer_stereo[self.fir_index] = stereo_sample;
            }
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

                if channels > 1 {
                    out_stereo += self.low_pass_fir[fi]
                        * (self.fir_buffer_stereo[ifbi] + self.fir_buffer_stereo[dfbi]);
                }

                ifbi += 1;
                if ifbi >= FIR_SIZE {
                    ifbi = 0;
                }
            }

        let mut mono = out_mono;
        let mut stereo = out_stereo;

        if let Some(tau) = self.preemphasis_tau {
            let a = (-1.0 / (tau * MPX_SAMPLE_RATE)).exp();
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
                (-1.0 / (self.comp_attack * MPX_SAMPLE_RATE)).exp()
            } else {
                (-1.0 / (self.comp_release * MPX_SAMPLE_RATE)).exp()
            };
            self.comp_gain_db = target_gain_db + coeff * (self.comp_gain_db - target_gain_db);
            let gain = 10f32.powf(self.comp_gain_db / 20.0);
            mono *= gain;
            stereo *= gain;
        }

        mpx_buffer[i] += 4.05 * mono;

            if channels > 1 {
            mpx_buffer[i] += (4.05 * self.stereo_separation) * CARRIER_38[self.phase_38] * stereo
                + self.pilot_level * CARRIER_19[self.phase_19];

                self.phase_19 += 1;
                self.phase_38 += 1;
                if self.phase_19 >= CARRIER_19.len() {
                    self.phase_19 = 0;
                }
                if self.phase_38 >= CARRIER_38.len() {
                    self.phase_38 = 0;
                }
            }

            self.audio_pos += 1.0;
        }

        Ok(())
    }
}
