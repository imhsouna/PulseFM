use std::path::Path;

use anyhow::Result;
use hound::{SampleFormat, WavSpec, WavWriter};

use crate::audio::load_wav;
use crate::fm_mpx::FmMpx;

const MPX_SAMPLE_RATE: u32 = 228000;
const SAMPLE_SCALE: f32 = 0.1;

#[derive(Clone, Debug)]
pub struct GenerateConfig {
    pub duration_secs: f32,
    pub audio_path: Option<String>,
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

pub fn generate_mpx_wav<F>(config: &GenerateConfig, output_path: &str, mut progress: F) -> Result<()>
where
    F: FnMut(f32),
{
    let audio = match config.audio_path.as_ref() {
        Some(path) => Some(load_wav(path)?),
        None => None,
    };

    let mut mpx = FmMpx::new(audio);
    mpx.set_rds_pi(config.pi);
    mpx.set_rds_ps(&config.ps);
    mpx.set_rds_rt(&config.rt);
    mpx.set_rds_tp(config.tp);
    mpx.set_rds_ta(config.ta);
    mpx.set_rds_pty(config.pty);
    mpx.set_rds_ms(config.ms);
    mpx.set_rds_di(config.di);
    mpx.set_rds_ab(config.ab);
    mpx.set_rds_ab_auto(config.ab_auto);
    mpx.set_rds_ct_enabled(config.ct_enabled);
    mpx.set_rds_af_list(&config.af_list_mhz);
    mpx.set_rds_ps_scroll(config.ps_scroll_enabled, &config.ps_scroll_text, config.ps_scroll_cps);
    mpx.set_rds_rt_scroll(config.rt_scroll_enabled, &config.rt_scroll_text, config.rt_scroll_cps);
    mpx.set_pilot_level(config.pilot_level);
    mpx.set_rds_level(config.rds_level);
    mpx.set_stereo_separation(config.stereo_separation);
    mpx.set_preemphasis(config.preemphasis_tau);
    mpx.set_compressor(
        config.compressor_enabled,
        config.comp_threshold_db,
        config.comp_ratio,
        config.comp_attack,
        config.comp_release,
    );
    mpx.set_rds_group_mix(config.group_0a, config.group_2a, config.group_4a);
    mpx.set_rds_ct_interval(config.ct_interval_groups);
    mpx.set_rds_ps_alternates(config.ps_alt_list.clone(), config.ps_alt_interval);

    let total_samples = (config.duration_secs * MPX_SAMPLE_RATE as f32) as usize;
    let chunk_size = 2048usize;

    let spec = WavSpec {
        channels: 1,
        sample_rate: MPX_SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };

    let mut writer = WavWriter::create(Path::new(output_path), spec)?;
    let mut generated = 0usize;

    while generated < total_samples {
        let remaining = total_samples - generated;
        let len = remaining.min(chunk_size);
        let mut buffer = vec![0.0f32; len];
        mpx.get_samples(&mut buffer)?;

        for sample in buffer {
            let mut out = sample * SAMPLE_SCALE * config.output_gain;
            if config.limiter_enabled {
                let threshold = config.limiter_threshold.max(0.1);
                if out > threshold {
                    out = threshold;
                } else if out < -threshold {
                    out = -threshold;
                }
            }
            writer.write_sample(out)?;
        }

        generated += len;
        progress(generated as f32 / total_samples as f32);
    }

    writer.finalize()?;
    Ok(())
}
