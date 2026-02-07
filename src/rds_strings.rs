use std::collections::HashMap;
use std::sync::OnceLock;

static RDS_MAP: OnceLock<HashMap<u32, u8>> = OnceLock::new();

fn rds_map() -> &'static HashMap<u32, u8> {
    RDS_MAP.get_or_init(|| {
        let mut map = HashMap::new();
        let raw = include_str!("../assets/rds_unicode_map.txt");
        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut parts = line.split_whitespace();
            let cp = parts
                .next()
                .and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok())
                .expect("invalid codepoint in rds map");
            let out = parts
                .next()
                .and_then(|v| u8::from_str_radix(v.trim_start_matches("0x"), 16).ok())
                .expect("invalid rds byte in rds map");
            map.insert(cp, out);
        }
        map
    })
}

pub fn fill_rds_string(target: &mut [u8], input: &str) {
    let map = rds_map();
    let mut out_index = 0;
    for ch in input.chars() {
        if out_index >= target.len() {
            break;
        }
        let cp = ch as u32;
        let rds_byte = map.get(&cp).copied().unwrap_or(0x20);
        target[out_index] = rds_byte;
        out_index += 1;
    }

    while out_index < target.len() {
        target[out_index] = 0x20;
        out_index += 1;
    }
}
