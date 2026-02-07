use anyhow::{anyhow, Result};
use hound::{SampleFormat, WavReader};

pub struct AudioSource {
    pub samples: Vec<f32>,
    pub channels: usize,
    pub sample_rate: u32,
}

pub fn load_wav(path: &str) -> Result<AudioSource> {
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();
    let channels = spec.channels as usize;

    if channels == 0 {
        return Err(anyhow!("invalid channel count"));
    }

    let mut samples = Vec::new();

    match spec.sample_format {
        SampleFormat::Float => {
            for sample in reader.samples::<f32>() {
                samples.push(sample?);
            }
        }
        SampleFormat::Int => {
            let bits = spec.bits_per_sample as i32;
            let max = (1i64 << (bits - 1)) - 1;
            if bits <= 16 {
                for sample in reader.samples::<i16>() {
                    let v = sample? as i64;
                    samples.push(v as f32 / max as f32);
                }
            } else {
                for sample in reader.samples::<i32>() {
                    let v = sample? as i64;
                    samples.push(v as f32 / max as f32);
                }
            }
        }
    }

    Ok(AudioSource {
        samples,
        channels,
        sample_rate: spec.sample_rate,
    })
}
