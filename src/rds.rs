use chrono::{Datelike, Timelike, Offset};
use chrono::NaiveDate;

use crate::rds_strings::fill_rds_string;
use crate::waveform::waveform_biphase;

const RT_LENGTH: usize = 64;
const PS_LENGTH: usize = 8;
const GROUP_LENGTH: usize = 4;

const POLY: u16 = 0x1B9;
const POLY_DEG: usize = 10;
const MSB_BIT: u16 = 0x8000;
const BLOCK_SIZE: usize = 16;

const BITS_PER_GROUP: usize = GROUP_LENGTH * (BLOCK_SIZE + POLY_DEG);
const SAMPLES_PER_BIT: usize = 192;

const OFFSET_WORDS: [u16; 4] = [0x0FC, 0x198, 0x168, 0x1B4];

#[derive(Clone)]
pub struct RdsParams {
    pub pi: u16,
    pub tp: bool,
    pub ta: bool,
    pub pty: u8,
    pub ms: bool,
    pub di: u8,
    pub ab: bool,
    pub ab_auto: bool,
    pub ct_enabled: bool,
    pub af_stream: Vec<u8>,
    pub ps: [u8; PS_LENGTH],
    pub rt: [u8; RT_LENGTH],
}

impl Default for RdsParams {
    fn default() -> Self {
        let params = RdsParams {
            pi: 0,
            tp: false,
            ta: false,
            pty: 0,
            ms: true,
            di: 0b1000,
            ab: false,
            ab_auto: true,
            ct_enabled: true,
            af_stream: Vec::new(),
            ps: [0x20; PS_LENGTH],
            rt: [0x20; RT_LENGTH],
        };
        params
    }
}

pub struct RdsGenerator {
    params: RdsParams,
    state: usize,
    ps_state: usize,
    rt_state: usize,
    latest_minutes: i32,

    bit_buffer: [u8; BITS_PER_GROUP],
    bit_pos: usize,

    sample_buffer: Vec<f32>,
    in_sample_index: usize,
    out_sample_index: usize,

    prev_output: u8,
    cur_output: u8,
    cur_bit: u8,
    sample_count: usize,
    inverting: bool,
    phase: usize,

    af_pos: usize,
    ps_scroll: Option<String>,
    rt_scroll: Option<String>,
    ps_scroll_pos: usize,
    rt_scroll_pos: usize,
    ps_scroll_interval_samples: usize,
    rt_scroll_interval_samples: usize,
    sample_ticks: usize,

    group_cycle: Vec<u8>,
    group_index: usize,
    ct_interval_groups: usize,
    ct_counter: usize,
    ps_alt_list: Vec<String>,
    ps_alt_index: usize,
    ps_alt_interval: usize,
    ps_alt_counter: usize,
}

impl RdsGenerator {
    pub fn new() -> Self {
        let filter_size = waveform_biphase().len();
        let sample_buffer_size = SAMPLES_PER_BIT + filter_size;

        RdsGenerator {
            params: RdsParams::default(),
            state: 0,
            ps_state: 0,
            rt_state: 0,
            latest_minutes: -1,

            bit_buffer: [0u8; BITS_PER_GROUP],
            bit_pos: BITS_PER_GROUP,

            sample_buffer: vec![0.0; sample_buffer_size],
            in_sample_index: 0,
            out_sample_index: sample_buffer_size - 1,

            prev_output: 0,
            cur_output: 0,
            cur_bit: 0,
            sample_count: SAMPLES_PER_BIT,
            inverting: false,
            phase: 0,

            af_pos: 0,
            ps_scroll: None,
            rt_scroll: None,
            ps_scroll_pos: 0,
            rt_scroll_pos: 0,
            ps_scroll_interval_samples: 228000 / 2,
            rt_scroll_interval_samples: 228000 / 2,
            sample_ticks: 0,

            group_cycle: vec![0, 0, 0, 0, 2],
            group_index: 0,
            ct_interval_groups: 0,
            ct_counter: 0,
            ps_alt_list: Vec::new(),
            ps_alt_index: 0,
            ps_alt_interval: 0,
            ps_alt_counter: 0,
        }
    }

    pub fn set_pi(&mut self, pi_code: u16) {
        self.params.pi = pi_code;
    }

    pub fn set_tp(&mut self, tp: bool) {
        self.params.tp = tp;
    }

    pub fn set_rt(&mut self, rt: &str) {
        let mut next = [0u8; RT_LENGTH];
        fill_rds_string(&mut next, rt);
        if next != self.params.rt {
            if self.params.ab_auto {
                self.params.ab = !self.params.ab;
            }
            self.params.rt = next;
        }
    }

    pub fn set_rt_ab(&mut self, ab: bool) {
        self.params.ab = ab;
    }

    pub fn set_rt_ab_auto(&mut self, ab_auto: bool) {
        self.params.ab_auto = ab_auto;
    }

    pub fn set_pty(&mut self, pty: u8) {
        self.params.pty = pty.min(31);
    }

    pub fn set_ms(&mut self, ms: bool) {
        self.params.ms = ms;
    }

    pub fn set_di(&mut self, di: u8) {
        self.params.di = di & 0x0F;
    }

    pub fn set_ct_enabled(&mut self, enabled: bool) {
        self.params.ct_enabled = enabled;
    }

    pub fn set_ps(&mut self, ps: &str) {
        fill_rds_string(&mut self.params.ps, ps);
    }

    pub fn set_ta(&mut self, ta: bool) {
        self.params.ta = ta;
    }

    pub fn set_defaults_tunisia(&mut self) {
        self.set_ps("BOUZIDFM");
        self.set_rt("BOUZIDFM Sidi Bouzid 98.0 MHz");
        self.params.pty = 10;
        self.params.tp = false;
        self.params.ta = false;
        self.params.ms = true;
        self.params.di = 0b1000;
        self.params.ab = false;
        self.params.ab_auto = true;
        self.params.ct_enabled = true;
        self.params.pi = 0x7200;
    }

    pub fn set_group_mix(&mut self, count_0a: usize, count_2a: usize, count_4a: usize) {
        let mut cycle = Vec::new();
        cycle.extend(std::iter::repeat(0).take(count_0a.max(1)));
        cycle.extend(std::iter::repeat(2).take(count_2a.max(1)));
        if count_4a > 0 {
            cycle.extend(std::iter::repeat(4).take(count_4a));
        }
        self.group_cycle = cycle;
        self.group_index = 0;
    }

    pub fn set_ct_interval_groups(&mut self, interval: usize) {
        self.ct_interval_groups = interval;
        self.ct_counter = 0;
    }

    pub fn set_ps_alternates(&mut self, list: Vec<String>, interval_groups: usize) {
        self.ps_alt_list = list;
        self.ps_alt_interval = interval_groups;
        self.ps_alt_index = 0;
        self.ps_alt_counter = 0;
    }

    pub fn set_af_list_mhz(&mut self, freqs: &[f32]) {
        let mut codes = Vec::new();
        for &mhz in freqs {
            if mhz < 87.6 || mhz > 107.9 {
                continue;
            }
            let code = ((mhz - 87.6) * 10.0).round() as i32 + 1;
            if code >= 1 && code <= 204 {
                codes.push(code as u8);
            }
        }

        codes.sort();
        codes.dedup();

        if codes.is_empty() {
            self.params.af_stream.clear();
            self.af_pos = 0;
            return;
        }

        let count = codes.len().min(25);
        let mut stream = Vec::with_capacity(count + 1);
        stream.push(0xE0 + count as u8);
        stream.extend(codes.into_iter().take(count));
        if stream.len() % 2 != 0 {
            stream.push(0x00);
        }
        self.params.af_stream = stream;
        self.af_pos = 0;
    }

    pub fn enable_ps_scroll(&mut self, enabled: bool, text: &str, chars_per_sec: f32) {
        if !enabled {
            self.ps_scroll = None;
            return;
        }
        self.ps_scroll = Some(text.to_string());
        self.ps_scroll_pos = 0;
        let cps = if chars_per_sec <= 0.1 { 0.1 } else { chars_per_sec };
        self.ps_scroll_interval_samples = (228000.0 / cps) as usize;
        if self.ps_scroll_interval_samples == 0 {
            self.ps_scroll_interval_samples = 1;
        }
    }

    pub fn enable_rt_scroll(&mut self, enabled: bool, text: &str, chars_per_sec: f32) {
        if !enabled {
            self.rt_scroll = None;
            return;
        }
        self.rt_scroll = Some(text.to_string());
        self.rt_scroll_pos = 0;
        let cps = if chars_per_sec <= 0.1 { 0.1 } else { chars_per_sec };
        self.rt_scroll_interval_samples = (228000.0 / cps) as usize;
        if self.rt_scroll_interval_samples == 0 {
            self.rt_scroll_interval_samples = 1;
        }
    }

    fn crc(block: u16) -> u16 {
        let mut crc: u16 = 0;
        let mut block = block;
        for _ in 0..BLOCK_SIZE {
            let bit = (block & MSB_BIT) != 0;
            block <<= 1;

            let msb = (crc >> (POLY_DEG - 1)) & 1;
            crc <<= 1;
            if (msb != 0) ^ bit {
                crc ^= POLY;
            }
        }
        crc
    }

    fn fill_rds_ct_group(&mut self, blocks: &mut [u16; GROUP_LENGTH]) {
        let now_utc = chrono::Utc::now();
        let now_local = chrono::Local::now();

        let date = NaiveDate::from_ymd_opt(now_utc.year(), now_utc.month(), now_utc.day())
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        let mjd_base = NaiveDate::from_ymd_opt(1858, 11, 17).unwrap();
        let mjd = (date - mjd_base).num_days() as i32;

        let base = (4u16 << 12)
            | ((self.params.tp as u16) << 10)
            | ((self.params.pty as u16) << 5);
        blocks[1] = base | ((mjd >> 15) as u16);
        blocks[2] = ((mjd << 1) as u16) | ((now_utc.hour() as u16) >> 4);
        blocks[3] = ((now_utc.hour() as u16 & 0xF) << 12) | ((now_utc.minute() as u16) << 6);

        let offset_minutes = now_local.offset().fix().local_minus_utc();
        let offset = offset_minutes / (30 * 60);

        let abs_offset = offset.abs() as u16;
        blocks[3] |= abs_offset;
        if offset < 0 {
            blocks[3] |= 0x20;
        }
    }

    fn get_rds_ct_group(&mut self, blocks: &mut [u16; GROUP_LENGTH]) -> bool {
        if !self.params.ct_enabled {
            return false;
        }
        let minute = chrono::Utc::now().minute() as i32;
        if minute == self.latest_minutes {
            return false;
        }
        self.latest_minutes = minute;
        self.fill_rds_ct_group(blocks);
        true
    }

    fn get_rds_group(&mut self, buffer: &mut [u8; BITS_PER_GROUP]) {
        let mut blocks: [u16; GROUP_LENGTH] = [self.params.pi, 0, 0, 0];

        if self.ps_alt_interval > 0 && !self.ps_alt_list.is_empty() {
            self.ps_alt_counter += 1;
            if self.ps_alt_counter >= self.ps_alt_interval {
                self.ps_alt_counter = 0;
                self.ps_alt_index = (self.ps_alt_index + 1) % self.ps_alt_list.len();
                let ps = self.ps_alt_list[self.ps_alt_index].clone();
                self.set_ps(&ps);
            }
        }

        let mut sent_ct = false;
        if self.ct_interval_groups > 0 {
            self.ct_counter += 1;
            if self.ct_counter >= self.ct_interval_groups {
                self.ct_counter = 0;
                self.fill_rds_ct_group(&mut blocks);
                sent_ct = true;
            }
        }

        if !sent_ct && !self.get_rds_ct_group(&mut blocks) {
            let group_type = if self.group_cycle.is_empty() {
                0
            } else {
                let g = self.group_cycle[self.group_index % self.group_cycle.len()];
                self.group_index = (self.group_index + 1) % self.group_cycle.len();
                g
            };

            if group_type == 0 && self.state < 4 {
                let di_bit = (self.params.di >> (3 - self.ps_state)) & 0x01;
                blocks[1] = (0u16 << 12)
                    | ((self.params.tp as u16) << 10)
                    | ((self.params.pty as u16) << 5)
                    | ((self.params.ta as u16) << 4)
                    | ((self.params.ms as u16) << 3)
                    | ((di_bit as u16) << 2)
                    | (self.ps_state as u16);
                if self.params.af_stream.is_empty() {
                    blocks[2] = 0xCDCD;
                } else {
                    let af1 = self.params.af_stream[self.af_pos % self.params.af_stream.len()];
                    let af2 = self.params.af_stream[(self.af_pos + 1) % self.params.af_stream.len()];
                    blocks[2] = ((af1 as u16) << 8) | (af2 as u16);
                    self.af_pos = (self.af_pos + 2) % self.params.af_stream.len();
                }
                let p = self.ps_state * 2;
                blocks[3] = ((self.params.ps[p] as u16) << 8) | (self.params.ps[p + 1] as u16);
                self.ps_state += 1;
                if self.ps_state >= 4 {
                    self.ps_state = 0;
                }
            } else if group_type == 2 {
                blocks[1] = (2u16 << 12)
                    | ((self.params.tp as u16) << 10)
                    | ((self.params.pty as u16) << 5)
                    | ((self.params.ab as u16) << 4)
                    | (self.rt_state as u16);
                let p = self.rt_state * 4;
                blocks[2] = ((self.params.rt[p] as u16) << 8) | (self.params.rt[p + 1] as u16);
                blocks[3] = ((self.params.rt[p + 2] as u16) << 8)
                    | (self.params.rt[p + 3] as u16);
                self.rt_state += 1;
                if self.rt_state >= 16 {
                    self.rt_state = 0;
                }
            } else if group_type == 4 {
                self.fill_rds_ct_group(&mut blocks);
            }

            self.state += 1;
            if self.state >= 5 {
                self.state = 0;
            }
        }

        let mut out_index = 0;
        for i in 0..GROUP_LENGTH {
            let mut block = blocks[i];
            let mut check = Self::crc(block) ^ OFFSET_WORDS[i];
            for _ in 0..BLOCK_SIZE {
                buffer[out_index] = if (block & (1 << (BLOCK_SIZE - 1))) != 0 { 1 } else { 0 };
                out_index += 1;
                block <<= 1;
            }
            for _ in 0..POLY_DEG {
                buffer[out_index] = if (check & (1 << (POLY_DEG - 1))) != 0 { 1 } else { 0 };
                out_index += 1;
                check <<= 1;
            }
        }
    }

    pub fn get_rds_samples(&mut self, buffer: &mut [f32]) {
        let filter = waveform_biphase();
        let sample_buffer_size = self.sample_buffer.len();

        for sample in buffer.iter_mut() {
            self.sample_ticks += 1;
            if let Some(ref text) = self.ps_scroll {
                if self.sample_ticks % self.ps_scroll_interval_samples == 0 {
                    let mut window = String::new();
                    let padded = format!("{}   ", text);
                    for i in 0..PS_LENGTH {
                        let idx = (self.ps_scroll_pos + i) % padded.len();
                        window.push(padded.as_bytes()[idx] as char);
                    }
                    self.ps_scroll_pos = (self.ps_scroll_pos + 1) % padded.len();
                    self.set_ps(&window);
                }
            }
            if let Some(ref text) = self.rt_scroll {
                if self.sample_ticks % self.rt_scroll_interval_samples == 0 {
                    let mut window = String::new();
                    let padded = format!("{}   ", text);
                    for i in 0..RT_LENGTH {
                        let idx = (self.rt_scroll_pos + i) % padded.len();
                        window.push(padded.as_bytes()[idx] as char);
                    }
                    self.rt_scroll_pos = (self.rt_scroll_pos + 1) % padded.len();
                    self.set_rt(&window);
                }
            }
            if self.sample_count >= SAMPLES_PER_BIT {
                if self.bit_pos >= BITS_PER_GROUP {
                    let mut buffer = [0u8; BITS_PER_GROUP];
                    self.get_rds_group(&mut buffer);
                    self.bit_buffer = buffer;
                    self.bit_pos = 0;
                }

                self.cur_bit = self.bit_buffer[self.bit_pos];
                self.prev_output = self.cur_output;
                self.cur_output = self.prev_output ^ self.cur_bit;
                self.inverting = self.cur_output == 1;

                let mut idx = self.in_sample_index;
                for &val in filter {
                    let mut v = val;
                    if self.inverting {
                        v = -v;
                    }
                    self.sample_buffer[idx] += v;
                    idx += 1;
                    if idx >= sample_buffer_size {
                        idx = 0;
                    }
                }

                self.in_sample_index += SAMPLES_PER_BIT;
                if self.in_sample_index >= sample_buffer_size {
                    self.in_sample_index -= sample_buffer_size;
                }

                self.bit_pos += 1;
                self.sample_count = 0;
            }

            let mut out = self.sample_buffer[self.out_sample_index];
            self.sample_buffer[self.out_sample_index] = 0.0;
            self.out_sample_index += 1;
            if self.out_sample_index >= sample_buffer_size {
                self.out_sample_index = 0;
            }

            match self.phase {
                0 | 2 => out = 0.0,
                1 => {}
                3 => out = -out,
                _ => {}
            }
            self.phase += 1;
            if self.phase >= 4 {
                self.phase = 0;
            }

            *sample = out;
            self.sample_count += 1;
        }
    }
}
