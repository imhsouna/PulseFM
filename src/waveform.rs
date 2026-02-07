use std::sync::OnceLock;

static WAVEFORM: OnceLock<Vec<f32>> = OnceLock::new();

pub fn waveform_biphase() -> &'static [f32] {
    WAVEFORM.get_or_init(|| {
        let raw = include_str!("../assets/waveform_biphase.txt");
        raw.lines()
            .filter_map(|line| {
                let t = line.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.parse::<f32>().expect("invalid waveform sample"))
                }
            })
            .collect::<Vec<f32>>()
    })
}
