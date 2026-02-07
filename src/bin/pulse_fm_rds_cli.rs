use std::env;

use anyhow::{anyhow, Result};

use pulse_fm_rds_encoder::wav_writer::{generate_mpx_wav, GenerateConfig};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 || args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return Ok(());
    }

    let mut out = None;
    let mut duration = 10.0f32;
    let mut ps = "BOUZIDFM".to_string();
    let mut rt = "BOUZIDFM Sidi Bouzid 98.0 MHz".to_string();
    let mut pi = 0x7200u16;
    let mut ta = false;
    let mut tp = false;
    let mut pty = 10u8;
    let mut ms = true;
    let mut di = 0b1000u8;
    let mut ab = false;
    let mut ab_auto = true;
    let mut ct_enabled = true;
    let mut af_list = vec![98.0f32];
    let mut ps_scroll_enabled = false;
    let mut ps_scroll_text = "BOUZIDFM".to_string();
    let mut ps_scroll_cps = 2.0f32;
    let mut rt_scroll_enabled = false;
    let mut rt_scroll_text = "BOUZIDFM Sidi Bouzid 98.0 MHz".to_string();
    let mut rt_scroll_cps = 2.0f32;
    let mut output_gain = 1.0f32;
    let mut limiter_enabled = true;
    let mut limiter_threshold = 0.95f32;
    let mut limiter_lookahead = 256usize;
    let mut pilot_level = 0.9f32;
    let mut rds_level = 1.0f32;
    let mut stereo_separation = 1.0f32;
    let mut preemphasis_tau = Some(50e-6f32);
    let mut compressor_enabled = false;
    let mut comp_threshold = -18.0f32;
    let mut comp_ratio = 3.0f32;
    let mut comp_attack = 0.01f32;
    let mut comp_release = 0.2f32;
    let mut group_0a = 4usize;
    let mut group_2a = 1usize;
    let mut group_4a = 0usize;
    let mut ct_interval_groups = 0usize;
    let mut ps_alt_list: Vec<String> = Vec::new();
    let mut ps_alt_interval = 0usize;
    let mut audio = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                i += 1;
                out = args.get(i).cloned();
            }
            "--duration" => {
                i += 1;
                duration = args
                    .get(i)
                    .ok_or_else(|| anyhow!("missing duration"))?
                    .parse::<f32>()?;
            }
            "--ps" => {
                i += 1;
                ps = args.get(i).cloned().ok_or_else(|| anyhow!("missing ps"))?;
            }
            "--rt" => {
                i += 1;
                rt = args.get(i).cloned().ok_or_else(|| anyhow!("missing rt"))?;
            }
            "--pi" => {
                i += 1;
                let raw = args.get(i).cloned().ok_or_else(|| anyhow!("missing pi"))?;
                let t = raw.trim_start_matches("0x");
                pi = u16::from_str_radix(t, 16)?;
            }
            "--ta" => {
                ta = true;
            }
            "--tp" => {
                tp = true;
            }
            "--pty" => {
                i += 1;
                pty = args
                    .get(i)
                    .ok_or_else(|| anyhow!("missing pty"))?
                    .parse::<u8>()?;
            }
            "--ms" => {
                ms = true;
            }
            "--speech" => {
                ms = false;
            }
            "--di" => {
                i += 1;
                let raw = args.get(i).cloned().ok_or_else(|| anyhow!("missing di"))?;
                di = u8::from_str_radix(raw.trim_start_matches("0x"), 16)?;
            }
            "--ab" => {
                ab = true;
            }
            "--no-ab-auto" => {
                ab_auto = false;
            }
            "--no-ct" => {
                ct_enabled = false;
            }
            "--audio" => {
                i += 1;
                audio = args.get(i).cloned();
            }
            "--af" => {
                i += 1;
                let raw = args.get(i).cloned().ok_or_else(|| anyhow!("missing af list"))?;
                af_list = raw
                    .split(',')
                    .filter_map(|s| s.trim().parse::<f32>().ok())
                    .collect();
            }
            "--ps-scroll" => {
                ps_scroll_enabled = true;
            }
            "--ps-scroll-text" => {
                i += 1;
                ps_scroll_text = args.get(i).cloned().ok_or_else(|| anyhow!("missing ps scroll text"))?;
            }
            "--ps-scroll-cps" => {
                i += 1;
                ps_scroll_cps = args.get(i).cloned().ok_or_else(|| anyhow!("missing ps scroll cps"))?.parse::<f32>()?;
            }
            "--rt-scroll" => {
                rt_scroll_enabled = true;
            }
            "--rt-scroll-text" => {
                i += 1;
                rt_scroll_text = args.get(i).cloned().ok_or_else(|| anyhow!("missing rt scroll text"))?;
            }
            "--rt-scroll-cps" => {
                i += 1;
                rt_scroll_cps = args.get(i).cloned().ok_or_else(|| anyhow!("missing rt scroll cps"))?.parse::<f32>()?;
            }
            "--gain" => {
                i += 1;
                output_gain = args.get(i).cloned().ok_or_else(|| anyhow!("missing gain"))?.parse::<f32>()?;
            }
            "--limiter" => {
                limiter_enabled = true;
            }
            "--no-limiter" => {
                limiter_enabled = false;
            }
            "--limiter-threshold" => {
                i += 1;
                limiter_threshold = args.get(i).cloned().ok_or_else(|| anyhow!("missing limiter threshold"))?.parse::<f32>()?;
            }
            "--lookahead" => {
                i += 1;
                limiter_lookahead = args.get(i).cloned().ok_or_else(|| anyhow!("missing lookahead"))?.parse::<usize>()?;
            }
            "--pilot" => {
                i += 1;
                pilot_level = args.get(i).cloned().ok_or_else(|| anyhow!("missing pilot level"))?.parse::<f32>()?;
            }
            "--rds-level" => {
                i += 1;
                rds_level = args.get(i).cloned().ok_or_else(|| anyhow!("missing rds level"))?.parse::<f32>()?;
            }
            "--stereo-sep" => {
                i += 1;
                stereo_separation = args.get(i).cloned().ok_or_else(|| anyhow!("missing stereo separation"))?.parse::<f32>()?;
            }
            "--preemph-50" => {
                preemphasis_tau = Some(50e-6);
            }
            "--preemph-75" => {
                preemphasis_tau = Some(75e-6);
            }
            "--preemph-off" => {
                preemphasis_tau = None;
            }
            "--comp" => {
                compressor_enabled = true;
            }
            "--comp-thr" => {
                i += 1;
                comp_threshold = args.get(i).cloned().ok_or_else(|| anyhow!("missing comp threshold"))?.parse::<f32>()?;
            }
            "--comp-ratio" => {
                i += 1;
                comp_ratio = args.get(i).cloned().ok_or_else(|| anyhow!("missing comp ratio"))?.parse::<f32>()?;
            }
            "--comp-attack" => {
                i += 1;
                comp_attack = args.get(i).cloned().ok_or_else(|| anyhow!("missing comp attack"))?.parse::<f32>()?;
            }
            "--comp-release" => {
                i += 1;
                comp_release = args.get(i).cloned().ok_or_else(|| anyhow!("missing comp release"))?.parse::<f32>()?;
            }
            "--group-mix" => {
                i += 1;
                let raw = args.get(i).cloned().ok_or_else(|| anyhow!("missing group mix"))?;
                let parts: Vec<_> = raw.split(',').collect();
                if parts.len() >= 3 {
                    group_0a = parts[0].trim().parse::<usize>().unwrap_or(4);
                    group_2a = parts[1].trim().parse::<usize>().unwrap_or(1);
                    group_4a = parts[2].trim().parse::<usize>().unwrap_or(0);
                }
            }
            "--ct-interval" => {
                i += 1;
                ct_interval_groups = args.get(i).cloned().ok_or_else(|| anyhow!("missing ct interval"))?.parse::<usize>()?;
            }
            "--ps-alt" => {
                i += 1;
                ps_alt_list = args.get(i).cloned().ok_or_else(|| anyhow!("missing ps alt list"))?
                    .split('|').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            }
            "--ps-alt-interval" => {
                i += 1;
                ps_alt_interval = args.get(i).cloned().ok_or_else(|| anyhow!("missing ps alt interval"))?.parse::<usize>()?;
            }
            other => {
                return Err(anyhow!("unknown arg: {}", other));
            }
        }
        i += 1;
    }

    let out = out.ok_or_else(|| anyhow!("--out is required"))?;

    let config = GenerateConfig {
        duration_secs: duration,
        audio_path: audio,
        ps,
        rt,
        pi,
        tp,
        ta,
        pty,
        ms,
        di,
        ab,
        ab_auto,
        ct_enabled,
        af_list_mhz: af_list,
        ps_scroll_enabled,
        ps_scroll_text,
        ps_scroll_cps,
        rt_scroll_enabled,
        rt_scroll_text,
        rt_scroll_cps,
        output_gain,
        limiter_enabled,
        limiter_threshold,
        limiter_lookahead,
        pilot_level,
        rds_level,
        stereo_separation,
        preemphasis_tau,
        compressor_enabled,
        comp_threshold_db: comp_threshold,
        comp_ratio,
        comp_attack,
        comp_release,
        group_0a,
        group_2a,
        group_4a,
        ct_interval_groups,
        ps_alt_list,
        ps_alt_interval,
    };

    generate_mpx_wav(&config, &out, |_| {})?;
    Ok(())
}

fn print_usage() {
    eprintln!("Usage: pulse-fm-rds-cli --out mpx.wav [--duration 10] [--ps text] [--rt text] [--pi 1234] [--tp] [--ta] [--pty N] [--ms|--speech] [--di 0xF] [--ab] [--no-ab-auto] [--no-ct] [--af 98.0,99.5] [--ps-scroll] [--ps-scroll-text t] [--ps-scroll-cps n] [--rt-scroll] [--rt-scroll-text t] [--rt-scroll-cps n] [--gain x] [--limiter|--no-limiter] [--limiter-threshold x] [--audio file.wav]");
}
