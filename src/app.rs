use iced::widget::{button, checkbox, column, container, pick_list, progress_bar, row, scrollable, slider, text, text_input, Column};
use iced::widget::button as button_widget;
use iced::widget::container as container_widget;
use iced::{Alignment, Background, Command, Element, Length, Theme};
use iced::theme;
use serde::{Deserialize, Serialize};
use image::{GenericImageView, Rgba, RgbaImage};
use std::fs;
use std::path::PathBuf;
use iced::widget::canvas::{Canvas, Frame, Geometry, Path, Program, Stroke, Text};
use iced::{Color, Renderer};
use std::time::Duration;

use pulse_fm_rds_encoder::audio_io::{list_input_devices, list_output_devices, start_engine, AudioEngine, AudioEngineConfig};
use pulse_fm_rds_encoder::wav_writer::{generate_mpx_wav, GenerateConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PtyItem {
    code: u8,
    label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Preemphasis {
    Off,
    Us50,
    Us75,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Tab {
    Dashboard,
    Audio,
    Rds,
    Processing,
    Meters,
    Export,
    RadioDns,
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tab::Dashboard => write!(f, "Dashboard"),
            Tab::Audio => write!(f, "Audio"),
            Tab::Rds => write!(f, "RDS"),
            Tab::Processing => write!(f, "Processing"),
            Tab::Meters => write!(f, "Meters"),
            Tab::Export => write!(f, "Export"),
            Tab::RadioDns => write!(f, "RadioDNS"),
        }
    }
}

impl std::fmt::Display for Preemphasis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Preemphasis::Off => write!(f, "Off"),
            Preemphasis::Us50 => write!(f, "50 µs"),
            Preemphasis::Us75 => write!(f, "75 µs"),
        }
    }
}

impl std::fmt::Display for PtyItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02} {}", self.code, self.label)
    }
}

fn pty_items() -> Vec<PtyItem> {
    vec![
        PtyItem { code: 0, label: "None" },
        PtyItem { code: 1, label: "News" },
        PtyItem { code: 2, label: "Current affairs" },
        PtyItem { code: 3, label: "Information" },
        PtyItem { code: 4, label: "Sport" },
        PtyItem { code: 5, label: "Education" },
        PtyItem { code: 6, label: "Drama" },
        PtyItem { code: 7, label: "Culture" },
        PtyItem { code: 8, label: "Science" },
        PtyItem { code: 9, label: "Varied" },
        PtyItem { code: 10, label: "Pop music" },
        PtyItem { code: 11, label: "Rock music" },
        PtyItem { code: 12, label: "Easy listening" },
        PtyItem { code: 13, label: "Light classical" },
        PtyItem { code: 14, label: "Serious classical" },
        PtyItem { code: 15, label: "Other music" },
        PtyItem { code: 16, label: "Weather" },
        PtyItem { code: 17, label: "Finance" },
        PtyItem { code: 18, label: "Children's programmes" },
        PtyItem { code: 19, label: "Social affairs" },
        PtyItem { code: 20, label: "Religion" },
        PtyItem { code: 21, label: "Phone-in" },
        PtyItem { code: 22, label: "Travel" },
        PtyItem { code: 23, label: "Leisure" },
        PtyItem { code: 24, label: "Jazz music" },
        PtyItem { code: 25, label: "Country music" },
        PtyItem { code: 26, label: "National music" },
        PtyItem { code: 27, label: "Oldies music" },
        PtyItem { code: 28, label: "Folk music" },
        PtyItem { code: 29, label: "Documentary" },
        PtyItem { code: 30, label: "Alarm test" },
        PtyItem { code: 31, label: "Alarm" },
    ]
}

fn preemph_items() -> Vec<Preemphasis> {
    vec![Preemphasis::Off, Preemphasis::Us50, Preemphasis::Us75]
}

fn color_bg() -> Color {
    Color::from_rgb8(10, 12, 16)
}

fn color_surface() -> Color {
    Color::from_rgb8(20, 26, 34)
}

fn color_surface_alt() -> Color {
    Color::from_rgb8(26, 34, 44)
}

fn color_border() -> Color {
    Color::from_rgb8(40, 52, 66)
}

fn color_text() -> Color {
    Color::from_rgb8(236, 242, 248)
}

fn color_muted() -> Color {
    Color::from_rgb8(150, 168, 186)
}

fn color_accent() -> Color {
    Color::from_rgb8(34, 211, 238)
}

fn color_accent_warm() -> Color {
    Color::from_rgb8(249, 115, 22)
}

fn color_live() -> Color {
    Color::from_rgb8(16, 185, 129)
}

fn color_danger() -> Color {
    Color::from_rgb8(239, 68, 68)
}

#[derive(Debug, Clone)]
pub enum Message {
    PsChanged(String),
    RtChanged(String),
    PiChanged(String),
    TaChanged(bool),
    TpChanged(bool),
    MsChanged(bool),
    DiStereoChanged(bool),
    DiArtificialChanged(bool),
    DiCompressedChanged(bool),
    DiDynamicChanged(bool),
    PtyChanged(PtyItem),
    AbChanged(bool),
    AbAutoChanged(bool),
    CtChanged(bool),
    FrequencyChanged(String),
    AfListChanged(String),
    AfBaseChanged(String),
    AfSpacingChanged(String),
    AfCountChanged(String),
    AfGenerate,
    PsScrollEnabled(bool),
    PsScrollTextChanged(String),
    PsScrollSpeedChanged(f32),
    RtScrollEnabled(bool),
    RtScrollTextChanged(String),
    RtScrollSpeedChanged(f32),
    GainChanged(f32),
    LimiterEnabled(bool),
    LimiterThresholdChanged(f32),
    LimiterLookaheadChanged(f32),
    PilotLevelChanged(f32),
    RdsLevelChanged(f32),
    StereoSeparationChanged(f32),
    PreemphasisChanged(Preemphasis),
    CompressorEnabled(bool),
    CompThresholdChanged(f32),
    CompRatioChanged(f32),
    CompAttackChanged(f32),
    CompReleaseChanged(f32),
    Group0aChanged(String),
    Group2aChanged(String),
    Group4aChanged(String),
    CtIntervalGroupsChanged(String),
    ApplyGroupMix,
    PsAltListChanged(String),
    PsAltIntervalChanged(String),
    ApplyPsAlternates,
    PresetSelected(String),
    PresetNameChanged(String),
    SavePreset,
    LoadPreset,
    TabSelected(Tab),
    Tick,
    CountryCodeChanged(String),
    AreaCodeChanged(String),
    ProgramRefChanged(String),
    EccChanged(String),
    ApplyPiFromParts,
    DurationChanged(String),
    AudioChanged(String),
    OutputChanged(String),
    Generate,
    Generated(Result<(), String>),
    GenerateRadioDnsPack,
    RadioDnsGenerated(Result<String, String>),
    RadioDnsDomainChanged(String),
    RadioDnsLogoPathChanged(String),
    RadioDnsSrvHostChanged(String),
    RadioDnsSrvPortChanged(String),
    RadioDnsBroadcasterChanged(String),
    RadioDnsBrowseLogo,
    RadioDnsLogoPicked(Option<String>),
    RadioDnsOpenFolder,
    RadioDnsValidatePack,
    RadioDnsValidationComplete(Result<String, String>),
    RadioDnsOpenSiXml,
    RadioDnsCopySrv,
    RadioDnsCopyFqdn,
    RadioDnsCopyBearer,
    RadioDnsCopyDnsBundle,
    RadioDnsCopyCname,
    RefreshDevices,
    InputSelected(String),
    OutputSelected(String),
    StartStream,
    StopStream,
}

pub struct App {
    ps: String,
    rt: String,
    pi_hex: String,
    ta: bool,
    tp: bool,
    ms: bool,
    di_stereo: bool,
    di_artificial: bool,
    di_compressed: bool,
    di_dynamic: bool,
    pty_items: Vec<PtyItem>,
    pty_selected: PtyItem,
    ab_flag: bool,
    ab_auto: bool,
    ct_enabled: bool,
    duration: String,
    audio_path: String,
    output_path: String,
    frequency_mhz: String,
    af_list_text: String,
    af_warning: Option<String>,
    af_base: String,
    af_spacing: String,
    af_count: String,
    ps_scroll_enabled: bool,
    ps_scroll_text: String,
    ps_scroll_cps: f32,
    rt_scroll_enabled: bool,
    rt_scroll_text: String,
    rt_scroll_cps: f32,
    output_gain: f32,
    limiter_enabled: bool,
    limiter_threshold: f32,
    limiter_lookahead_ms: f32,
    pilot_level: f32,
    rds_level: f32,
    stereo_separation: f32,
    preemphasis_items: Vec<Preemphasis>,
    preemphasis_selected: Preemphasis,
    compressor_enabled: bool,
    comp_threshold: f32,
    comp_ratio: f32,
    comp_attack: f32,
    comp_release: f32,
    group_0a: String,
    group_2a: String,
    group_4a: String,
    ct_interval_groups: String,
    ps_alt_list_text: String,
    ps_alt_interval: String,
    meter_rms: f32,
    meter_peak: f32,
    meter_pilot: f32,
    meter_rds: f32,
    meter_bands_db: [f32; 48],
    scope_samples: Vec<f32>,
    scope_prev: Vec<f32>,
    spectrum_peak_db: Vec<f32>,
    spectrum_avg_db: Vec<f32>,
    xrun_count: u32,
    buffer_fill: f32,
    latency_ms: f32,
    pi_country_hex: String,
    pi_area_hex: String,
    pi_program_hex: String,
    ecc_hex: String,
    presets: Vec<Preset>,
    preset_selected: Option<String>,
    preset_name: String,
    tab_selected: Tab,
    status: String,
    generating: bool,
    radiodns_generating: bool,
    radiodns_last_output: Option<String>,
    radiodns_domain: String,
    radiodns_logo_path: String,
    radiodns_srv_host: String,
    radiodns_srv_port: String,
    radiodns_broadcaster_fqdn: String,
    radiodns_validation: Option<String>,
    radiodns_autofill_srv_host: bool,
    input_devices: Vec<String>,
    output_devices: Vec<String>,
    selected_input: Option<String>,
    selected_output: Option<String>,
    engine: Option<AudioEngine>,
}

impl Default for App {
    fn default() -> Self {
        App {
            ps: "BOUZIDFM".to_string(),
            rt: "BOUZIDFM Sidi Bouzid 98.0 MHz".to_string(),
            pi_hex: "7200".to_string(),
            ta: false,
            tp: false,
            ms: true,
            di_stereo: true,
            di_artificial: false,
            di_compressed: false,
            di_dynamic: false,
            pty_items: pty_items(),
            pty_selected: PtyItem { code: 10, label: "Pop music" },
            ab_flag: false,
            ab_auto: true,
            ct_enabled: true,
            duration: "10".to_string(),
            audio_path: "".to_string(),
            output_path: "mpx.wav".to_string(),
            frequency_mhz: "98.0".to_string(),
            af_list_text: "98.0".to_string(),
            af_warning: None,
            af_base: "98.0".to_string(),
            af_spacing: "0.2".to_string(),
            af_count: "1".to_string(),
            ps_scroll_enabled: false,
            ps_scroll_text: "BOUZIDFM".to_string(),
            ps_scroll_cps: 2.0,
            rt_scroll_enabled: false,
            rt_scroll_text: "BOUZIDFM Sidi Bouzid 98.0 MHz".to_string(),
            rt_scroll_cps: 2.0,
            output_gain: 1.0,
            limiter_enabled: true,
            limiter_threshold: 0.95,
            limiter_lookahead_ms: 2.0,
            pilot_level: 0.9,
            rds_level: 1.0,
            stereo_separation: 1.0,
            preemphasis_items: preemph_items(),
            preemphasis_selected: Preemphasis::Us50,
            compressor_enabled: false,
            comp_threshold: -18.0,
            comp_ratio: 3.0,
            comp_attack: 0.01,
            comp_release: 0.2,
            group_0a: "4".to_string(),
            group_2a: "1".to_string(),
            group_4a: "0".to_string(),
            ct_interval_groups: "0".to_string(),
            ps_alt_list_text: "".to_string(),
            ps_alt_interval: "0".to_string(),
            meter_rms: 0.0,
            meter_peak: 0.0,
            meter_pilot: 0.0,
            meter_rds: 0.0,
            meter_bands_db: [-60.0; 48],
            scope_samples: Vec::new(),
            scope_prev: Vec::new(),
            spectrum_peak_db: Vec::new(),
            spectrum_avg_db: Vec::new(),
            xrun_count: 0,
            buffer_fill: 0.0,
            latency_ms: 0.0,
            pi_country_hex: "7".to_string(),
            pi_area_hex: "2".to_string(),
            pi_program_hex: "00".to_string(),
            ecc_hex: "E2".to_string(),
            presets: Vec::new(),
            preset_selected: None,
            preset_name: "BOUZIDFM".to_string(),
            tab_selected: Tab::Dashboard,
            status: "Idle".to_string(),
            generating: false,
            radiodns_generating: false,
            radiodns_last_output: None,
            radiodns_domain: "https://YOUR_DOMAIN".to_string(),
            radiodns_logo_path: "".to_string(),
            radiodns_srv_host: "radio.your-domain.com".to_string(),
            radiodns_srv_port: "80".to_string(),
            radiodns_broadcaster_fqdn: "".to_string(),
            radiodns_validation: None,
            radiodns_autofill_srv_host: true,
            input_devices: Vec::new(),
            output_devices: Vec::new(),
            selected_input: None,
            selected_output: None,
            engine: None,
        }
    }
}

impl iced::Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut app = Self::default();
        app.presets = load_presets().unwrap_or_default();
        app.refresh_devices();
        (app, Command::none())
    }

    fn title(&self) -> String {
        "Pulse FM RDS Encoder".to_string()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::time::every(Duration::from_millis(200)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::PsChanged(v) => {
                self.ps = v;
                if let Some(engine) = &self.engine {
                    engine.update_ps(&self.ps);
                }
                Command::none()
            }
            Message::RtChanged(v) => {
                self.rt = v;
                if let Some(engine) = &self.engine {
                    engine.update_rt(&self.rt);
                }
                Command::none()
            }
            Message::PiChanged(v) => {
                self.pi_hex = v;
                if let Some(engine) = &self.engine {
                    if let Ok(pi) = parse_pi(&self.pi_hex) {
                        engine.update_pi(pi);
                    }
                }
                Command::none()
            }
            Message::TaChanged(v) => {
                self.ta = v;
                if let Some(engine) = &self.engine {
                    engine.update_ta(self.ta);
                }
                Command::none()
            }
            Message::TpChanged(v) => {
                self.tp = v;
                if let Some(engine) = &self.engine {
                    engine.update_tp(self.tp);
                }
                Command::none()
            }
            Message::MsChanged(v) => {
                self.ms = v;
                if let Some(engine) = &self.engine {
                    engine.update_ms(self.ms);
                }
                Command::none()
            }
            Message::DiStereoChanged(v) => {
                self.di_stereo = v;
                if let Some(engine) = &self.engine {
                    engine.update_di(self.di_bits());
                }
                Command::none()
            }
            Message::DiArtificialChanged(v) => {
                self.di_artificial = v;
                if let Some(engine) = &self.engine {
                    engine.update_di(self.di_bits());
                }
                Command::none()
            }
            Message::DiCompressedChanged(v) => {
                self.di_compressed = v;
                if let Some(engine) = &self.engine {
                    engine.update_di(self.di_bits());
                }
                Command::none()
            }
            Message::DiDynamicChanged(v) => {
                self.di_dynamic = v;
                if let Some(engine) = &self.engine {
                    engine.update_di(self.di_bits());
                }
                Command::none()
            }
            Message::PtyChanged(v) => {
                self.pty_selected = v;
                if let Some(engine) = &self.engine {
                    engine.update_pty(self.pty_selected.code);
                }
                Command::none()
            }
            Message::AbChanged(v) => {
                self.ab_flag = v;
                if let Some(engine) = &self.engine {
                    engine.update_ab(self.ab_flag);
                }
                Command::none()
            }
            Message::AbAutoChanged(v) => {
                self.ab_auto = v;
                if let Some(engine) = &self.engine {
                    engine.update_ab_auto(self.ab_auto);
                }
                Command::none()
            }
            Message::CtChanged(v) => {
                self.ct_enabled = v;
                if let Some(engine) = &self.engine {
                    engine.update_ct_enabled(self.ct_enabled);
                }
                Command::none()
            }
            Message::FrequencyChanged(v) => {
                self.frequency_mhz = v;
                Command::none()
            }
            Message::AfListChanged(v) => {
                self.af_list_text = v;
                if let Some(engine) = &self.engine {
                    let (list, warning) = parse_af_list(&self.af_list_text);
                    self.af_warning = warning;
                    engine.update_af_list(&list);
                }
                Command::none()
            }
            Message::AfBaseChanged(v) => {
                self.af_base = v;
                Command::none()
            }
            Message::AfSpacingChanged(v) => {
                self.af_spacing = v;
                Command::none()
            }
            Message::AfCountChanged(v) => {
                self.af_count = v;
                Command::none()
            }
            Message::AfGenerate => {
                let base = self.af_base.trim().parse::<f32>().unwrap_or(98.0);
                let spacing = self.af_spacing.trim().parse::<f32>().unwrap_or(0.2);
                let count = self.af_count.trim().parse::<usize>().unwrap_or(1).min(25);
                let mut freqs = Vec::new();
                for i in 0..count {
                    freqs.push(base + spacing * i as f32);
                }
                self.af_list_text = freqs.iter().map(|f| format!("{:.1}", f)).collect::<Vec<_>>().join(", ");
                let (list, warning) = parse_af_list(&self.af_list_text);
                self.af_warning = warning;
                if let Some(engine) = &self.engine {
                    engine.update_af_list(&list);
                }
                Command::none()
            }
            Message::PsScrollEnabled(v) => {
                self.ps_scroll_enabled = v;
                if let Some(engine) = &self.engine {
                    engine.update_ps_scroll(self.ps_scroll_enabled, &self.ps_scroll_text, self.ps_scroll_cps);
                }
                Command::none()
            }
            Message::PsScrollTextChanged(v) => {
                self.ps_scroll_text = v;
                if let Some(engine) = &self.engine {
                    engine.update_ps_scroll(self.ps_scroll_enabled, &self.ps_scroll_text, self.ps_scroll_cps);
                }
                Command::none()
            }
            Message::PsScrollSpeedChanged(v) => {
                self.ps_scroll_cps = v;
                if let Some(engine) = &self.engine {
                    engine.update_ps_scroll(self.ps_scroll_enabled, &self.ps_scroll_text, self.ps_scroll_cps);
                }
                Command::none()
            }
            Message::RtScrollEnabled(v) => {
                self.rt_scroll_enabled = v;
                if let Some(engine) = &self.engine {
                    engine.update_rt_scroll(self.rt_scroll_enabled, &self.rt_scroll_text, self.rt_scroll_cps);
                }
                Command::none()
            }
            Message::RtScrollTextChanged(v) => {
                self.rt_scroll_text = v;
                if let Some(engine) = &self.engine {
                    engine.update_rt_scroll(self.rt_scroll_enabled, &self.rt_scroll_text, self.rt_scroll_cps);
                }
                Command::none()
            }
            Message::RtScrollSpeedChanged(v) => {
                self.rt_scroll_cps = v;
                if let Some(engine) = &self.engine {
                    engine.update_rt_scroll(self.rt_scroll_enabled, &self.rt_scroll_text, self.rt_scroll_cps);
                }
                Command::none()
            }
            Message::GainChanged(v) => {
                self.output_gain = v;
                if let Some(engine) = &self.engine {
                    engine.update_gain(self.output_gain);
                }
                Command::none()
            }
            Message::LimiterEnabled(v) => {
                self.limiter_enabled = v;
                if let Some(engine) = &self.engine {
                    engine.update_limiter(self.limiter_enabled, self.limiter_threshold);
                }
                Command::none()
            }
            Message::LimiterThresholdChanged(v) => {
                self.limiter_threshold = v;
                if let Some(engine) = &self.engine {
                    engine.update_limiter(self.limiter_enabled, self.limiter_threshold);
                }
                Command::none()
            }
            Message::LimiterLookaheadChanged(v) => {
                self.limiter_lookahead_ms = v;
                if let Some(engine) = &self.engine {
                    let samples = ((self.limiter_lookahead_ms / 1000.0) * 228000.0) as usize;
                    engine.update_limiter_lookahead(samples);
                }
                Command::none()
            }
            Message::PilotLevelChanged(v) => {
                self.pilot_level = v;
                if let Some(engine) = &self.engine {
                    engine.update_pilot_level(self.pilot_level);
                }
                Command::none()
            }
            Message::RdsLevelChanged(v) => {
                self.rds_level = v;
                if let Some(engine) = &self.engine {
                    engine.update_rds_level(self.rds_level);
                }
                Command::none()
            }
            Message::StereoSeparationChanged(v) => {
                self.stereo_separation = v;
                if let Some(engine) = &self.engine {
                    engine.update_stereo_separation(self.stereo_separation);
                }
                Command::none()
            }
            Message::PreemphasisChanged(v) => {
                self.preemphasis_selected = v;
                if let Some(engine) = &self.engine {
                    engine.update_preemphasis(preemph_to_tau(self.preemphasis_selected.clone()));
                }
                Command::none()
            }
            Message::CompressorEnabled(v) => {
                self.compressor_enabled = v;
                if let Some(engine) = &self.engine {
                    engine.update_compressor(
                        self.compressor_enabled,
                        self.comp_threshold,
                        self.comp_ratio,
                        self.comp_attack,
                        self.comp_release,
                    );
                }
                Command::none()
            }
            Message::CompThresholdChanged(v) => {
                self.comp_threshold = v;
                if let Some(engine) = &self.engine {
                    engine.update_compressor(
                        self.compressor_enabled,
                        self.comp_threshold,
                        self.comp_ratio,
                        self.comp_attack,
                        self.comp_release,
                    );
                }
                Command::none()
            }
            Message::CompRatioChanged(v) => {
                self.comp_ratio = v;
                if let Some(engine) = &self.engine {
                    engine.update_compressor(
                        self.compressor_enabled,
                        self.comp_threshold,
                        self.comp_ratio,
                        self.comp_attack,
                        self.comp_release,
                    );
                }
                Command::none()
            }
            Message::CompAttackChanged(v) => {
                self.comp_attack = v;
                if let Some(engine) = &self.engine {
                    engine.update_compressor(
                        self.compressor_enabled,
                        self.comp_threshold,
                        self.comp_ratio,
                        self.comp_attack,
                        self.comp_release,
                    );
                }
                Command::none()
            }
            Message::CompReleaseChanged(v) => {
                self.comp_release = v;
                if let Some(engine) = &self.engine {
                    engine.update_compressor(
                        self.compressor_enabled,
                        self.comp_threshold,
                        self.comp_ratio,
                        self.comp_attack,
                        self.comp_release,
                    );
                }
                Command::none()
            }
            Message::Group0aChanged(v) => {
                self.group_0a = v;
                Command::none()
            }
            Message::Group2aChanged(v) => {
                self.group_2a = v;
                Command::none()
            }
            Message::Group4aChanged(v) => {
                self.group_4a = v;
                Command::none()
            }
            Message::CtIntervalGroupsChanged(v) => {
                self.ct_interval_groups = v;
                Command::none()
            }
            Message::ApplyGroupMix => {
                if let Some(engine) = &self.engine {
                    let g0 = self.group_0a.trim().parse::<usize>().unwrap_or(4);
                    let g2 = self.group_2a.trim().parse::<usize>().unwrap_or(1);
                    let g4 = self.group_4a.trim().parse::<usize>().unwrap_or(0);
                    engine.update_group_mix(g0, g2, g4);
                    let ctg = self.ct_interval_groups.trim().parse::<usize>().unwrap_or(0);
                    engine.update_ct_interval(ctg);
                }
                Command::none()
            }
            Message::PsAltListChanged(v) => {
                self.ps_alt_list_text = v;
                Command::none()
            }
            Message::PsAltIntervalChanged(v) => {
                self.ps_alt_interval = v;
                Command::none()
            }
            Message::ApplyPsAlternates => {
                if let Some(engine) = &self.engine {
                    let list = self.ps_alt_list_text
                        .split('|')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>();
                    let interval = self.ps_alt_interval.trim().parse::<usize>().unwrap_or(0);
                    engine.update_ps_alternates(list, interval);
                }
                Command::none()
            }
            Message::PresetSelected(v) => {
                self.preset_selected = Some(v);
                Command::none()
            }
            Message::PresetNameChanged(v) => {
                self.preset_name = v;
                Command::none()
            }
            Message::SavePreset => {
                let preset = self.to_preset();
                let mut presets = self.presets.clone();
                if let Some(pos) = presets.iter().position(|p| p.name == preset.name) {
                    presets[pos] = preset;
                } else {
                    presets.push(preset);
                }
                if let Err(e) = save_presets(&presets) {
                    self.status = format!("Preset save error: {}", e);
                } else {
                    self.presets = presets;
                }
                Command::none()
            }
            Message::LoadPreset => {
                if let Some(name) = &self.preset_selected {
                    if let Some(p) = self.presets.iter().find(|p| &p.name == name).cloned() {
                        self.apply_preset(p);
                    }
                }
                Command::none()
            }
            Message::TabSelected(tab) => {
                self.tab_selected = tab;
                Command::none()
            }
            Message::Tick => {
                if let Some(engine) = &self.engine {
                    let snapshot = engine.meter_snapshot();
                    self.meter_rms = snapshot.rms;
                    self.meter_peak = snapshot.peak;
                    self.meter_pilot = snapshot.pilot;
                    self.meter_rds = snapshot.rds;
                    for i in 0..self.meter_bands_db.len() {
                        let incoming = snapshot.bands_db[i];
                        let prev = self.meter_bands_db[i];
                        let decayed = prev - 1.5;
                        self.meter_bands_db[i] = incoming.max(decayed);
                    }
                    self.scope_prev = self.scope_samples.clone();
                    self.scope_samples = snapshot.scope;
                    self.spectrum_peak_db = snapshot.spectrum_peak_db;
                    self.spectrum_avg_db = snapshot.spectrum_avg_db;
                    self.xrun_count = snapshot.xrun_count;
                    self.buffer_fill = snapshot.buffer_fill;
                    self.latency_ms = snapshot.latency_ms;
                }
                Command::none()
            }
            Message::CountryCodeChanged(v) => {
                self.pi_country_hex = v;
                Command::none()
            }
            Message::AreaCodeChanged(v) => {
                self.pi_area_hex = v;
                Command::none()
            }
            Message::ProgramRefChanged(v) => {
                self.pi_program_hex = v;
                Command::none()
            }
            Message::EccChanged(v) => {
                self.ecc_hex = v;
                Command::none()
            }
            Message::ApplyPiFromParts => {
                match build_pi_from_parts(&self.pi_country_hex, &self.pi_area_hex, &self.pi_program_hex, &self.ecc_hex) {
                    Ok(pi) => {
                        self.pi_hex = format!("{:04X}", pi);
                        if let Some(engine) = &self.engine {
                            engine.update_pi(pi);
                        }
                    }
                    Err(e) => {
                        self.status = e;
                    }
                }
                Command::none()
            }
            Message::DurationChanged(v) => {
                self.duration = v;
                Command::none()
            }
            Message::AudioChanged(v) => {
                self.audio_path = v;
                Command::none()
            }
            Message::OutputChanged(v) => {
                self.output_path = v;
                Command::none()
            }
            Message::Generate => {
                if self.generating {
                    return Command::none();
                }

                let pi = match parse_pi(&self.pi_hex) {
                    Ok(v) => v,
                    Err(e) => {
                        self.status = e;
                        return Command::none();
                    }
                };

                let duration = match self.duration.trim().parse::<f32>() {
                    Ok(v) if v > 0.0 => v,
                    _ => {
                        self.status = "Duration must be a positive number".to_string();
                        return Command::none();
                    }
                };

                let audio_path = self.audio_path.trim();
                let audio_path = if audio_path.is_empty() {
                    None
                } else {
                    Some(audio_path.to_string())
                };

                let config = GenerateConfig {
                    duration_secs: duration,
                    audio_path,
                    ps: self.ps.clone(),
                    rt: self.rt.clone(),
                    pi,
                    tp: self.tp,
                    ta: self.ta,
                    pty: self.pty_selected.code,
                    ms: self.ms,
                    di: self.di_bits(),
                    ab: self.ab_flag,
                    ab_auto: self.ab_auto,
                    ct_enabled: self.ct_enabled,
                    af_list_mhz: parse_af_list(&self.af_list_text).0,
                    ps_scroll_enabled: self.ps_scroll_enabled,
                    ps_scroll_text: self.ps_scroll_text.clone(),
                    ps_scroll_cps: self.ps_scroll_cps,
                    rt_scroll_enabled: self.rt_scroll_enabled,
                    rt_scroll_text: self.rt_scroll_text.clone(),
                    rt_scroll_cps: self.rt_scroll_cps,
                    output_gain: self.output_gain,
                    limiter_enabled: self.limiter_enabled,
                    limiter_threshold: self.limiter_threshold,
                    limiter_lookahead: ((self.limiter_lookahead_ms / 1000.0) * 228000.0) as usize,
                    pilot_level: self.pilot_level,
                    rds_level: self.rds_level,
                    stereo_separation: self.stereo_separation,
                    preemphasis_tau: preemph_to_tau(self.preemphasis_selected.clone()),
                    compressor_enabled: self.compressor_enabled,
                    comp_threshold_db: self.comp_threshold,
                    comp_ratio: self.comp_ratio,
                    comp_attack: self.comp_attack,
                    comp_release: self.comp_release,
                    group_0a: self.group_0a.trim().parse::<usize>().unwrap_or(4),
                    group_2a: self.group_2a.trim().parse::<usize>().unwrap_or(1),
                    group_4a: self.group_4a.trim().parse::<usize>().unwrap_or(0),
                    ct_interval_groups: self.ct_interval_groups.trim().parse::<usize>().unwrap_or(0),
                    ps_alt_list: self.ps_alt_list_text
                        .split('|')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                    ps_alt_interval: self.ps_alt_interval.trim().parse::<usize>().unwrap_or(0),
                };

                let output_path = self.output_path.trim().to_string();
                if output_path.is_empty() {
                    self.status = "Output path is required".to_string();
                    return Command::none();
                }

                self.status = "Generating...".to_string();
                self.generating = true;

                Command::perform(
                    async move {
                        generate_mpx_wav(&config, &output_path, |_| {})
                            .map_err(|e: anyhow::Error| e.to_string())
                    },
                    Message::Generated,
                )
            }
            Message::Generated(result) => {
                self.generating = false;
                match result {
                    Ok(()) => self.status = "Done".to_string(),
                    Err(e) => self.status = format!("Error: {}", e),
                }
                Command::none()
            }
            Message::GenerateRadioDnsPack => {
                if self.radiodns_generating {
                    return Command::none();
                }

                let ps = self.ps.clone();
                let rt = self.rt.clone();
                let freq = self.frequency_mhz.clone();
                let pi = self.pi_hex.clone();
                let ecc = self.ecc_hex.clone();
                let domain = self.radiodns_domain.clone();
                let logo_path = self.radiodns_logo_path.clone();
                let srv_host = self.radiodns_srv_host.clone();
                let srv_port = self.radiodns_srv_port.clone();
                let broadcaster = self.radiodns_broadcaster_fqdn.clone();

                self.status = "Generating RadioDNS pack...".to_string();
                self.radiodns_generating = true;

                Command::perform(
                    async move { generate_radiodns_pack(ps, rt, freq, pi, ecc, domain, logo_path, srv_host, srv_port, broadcaster) },
                    Message::RadioDnsGenerated,
                )
            }
            Message::RadioDnsGenerated(result) => {
                self.radiodns_generating = false;
                match result {
                    Ok(path) => {
                        self.radiodns_last_output = Some(path);
                        self.status = "RadioDNS pack generated".to_string();
                        let base_dir = std::env::current_dir()
                            .unwrap_or_else(|_| std::path::PathBuf::from("."))
                            .join("radiodns");
                        let _ = open_in_file_manager(&base_dir);
                    }
                    Err(e) => self.status = format!("RadioDNS error: {}", e),
                }
                Command::none()
            }
            Message::RadioDnsDomainChanged(v) => {
                self.radiodns_domain = v;
                if self.radiodns_autofill_srv_host {
                    if let Some(host) = derive_host_from_base_url(&self.radiodns_domain) {
                        if self.radiodns_srv_host.trim().is_empty()
                            || self.radiodns_srv_host == "radio.your-domain.com"
                        {
                            self.radiodns_srv_host = format!("radio.{host}");
                        }
                    }
                }
                Command::none()
            }
            Message::RadioDnsLogoPathChanged(v) => {
                self.radiodns_logo_path = v;
                Command::none()
            }
            Message::RadioDnsSrvHostChanged(v) => {
                self.radiodns_srv_host = v;
                Command::none()
            }
            Message::RadioDnsSrvPortChanged(v) => {
                self.radiodns_srv_port = v;
                Command::none()
            }
            Message::RadioDnsBroadcasterChanged(v) => {
                self.radiodns_broadcaster_fqdn = v;
                Command::none()
            }
            Message::RadioDnsBrowseLogo => Command::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg"])
                        .pick_file()
                        .await
                        .map(|f| f.path().display().to_string())
                },
                Message::RadioDnsLogoPicked,
            ),
            Message::RadioDnsLogoPicked(path) => {
                if let Some(path) = path {
                    self.radiodns_logo_path = path;
                }
                Command::none()
            }
            Message::RadioDnsOpenFolder => {
                let base_dir = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .join("radiodns");
                if let Err(e) = fs::create_dir_all(&base_dir) {
                    self.status = format!("RadioDNS folder error: {}", e);
                    return Command::none();
                }
                if let Err(e) = open_in_file_manager(&base_dir) {
                    self.status = format!("Open folder error: {}", e);
                }
                Command::none()
            }
            Message::RadioDnsOpenSiXml => {
                let si_path = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .join("radiodns")
                    .join("SI.xml");
                if let Err(e) = open_in_file_manager(&si_path) {
                    self.status = format!("Open SI.xml error: {}", e);
                }
                Command::none()
            }
            Message::RadioDnsCopySrv => {
                let srv_line = build_srv_record_line(&self.radiodns_broadcaster_fqdn, &self.radiodns_srv_host, &self.radiodns_srv_port);
                self.status = "SRV record copied".to_string();
                Command::batch(vec![iced::clipboard::write(srv_line)])
            }
            Message::RadioDnsCopyFqdn => {
                let fqdn = self.radiodns_fm_strings().0.unwrap_or_else(|| "—".to_string());
                self.status = "FQDN copied".to_string();
                Command::batch(vec![iced::clipboard::write(fqdn)])
            }
            Message::RadioDnsCopyBearer => {
                let bearer = self.radiodns_fm_strings().1.unwrap_or_else(|| "—".to_string());
                self.status = "Bearer copied".to_string();
                Command::batch(vec![iced::clipboard::write(bearer)])
            }
            Message::RadioDnsCopyDnsBundle => {
                let (fqdn, _bearer, _lookup, _warning) = self.radiodns_fm_strings();
                let bundle = build_dns_bundle(
                    fqdn.as_deref(),
                    &self.radiodns_broadcaster_fqdn,
                    &self.radiodns_srv_host,
                    &self.radiodns_srv_port,
                );
                self.status = "DNS bundle copied".to_string();
                Command::batch(vec![iced::clipboard::write(bundle)])
            }
            Message::RadioDnsCopyCname => {
                let (fqdn, _bearer, _lookup, _warning) = self.radiodns_fm_strings();
                let cname = build_cname_line(fqdn.as_deref(), &self.radiodns_broadcaster_fqdn);
                self.status = "CNAME copied".to_string();
                Command::batch(vec![iced::clipboard::write(cname)])
            }
            Message::RadioDnsValidatePack => {
                Command::perform(async move { validate_radiodns_pack() }, Message::RadioDnsValidationComplete)
            }
            Message::RadioDnsValidationComplete(result) => {
                match result {
                    Ok(msg) => self.radiodns_validation = Some(msg),
                    Err(e) => self.radiodns_validation = Some(format!("Validation failed: {}", e)),
                }
                Command::none()
            }
            Message::RefreshDevices => {
                self.refresh_devices();
                Command::none()
            }
            Message::InputSelected(v) => {
                self.selected_input = Some(v);
                Command::none()
            }
            Message::OutputSelected(v) => {
                self.selected_output = Some(v);
                Command::none()
            }
            Message::StartStream => {
                if self.engine.is_some() {
                    return Command::none();
                }
                let output = match self.selected_output.clone() {
                    Some(v) => v,
                    None => {
                        self.status = "Select an output device".to_string();
                        return Command::none();
                    }
                };
                let pi = match parse_pi(&self.pi_hex) {
                    Ok(v) => v,
                    Err(e) => {
                        self.status = e;
                        return Command::none();
                    }
                };
                let config = AudioEngineConfig {
                    input_device: self.selected_input.clone(),
                    output_device: output,
                    ps: self.ps.clone(),
                    rt: self.rt.clone(),
                    pi,
                    tp: self.tp,
                    ta: self.ta,
                    pty: self.pty_selected.code,
                    ms: self.ms,
                    di: self.di_bits(),
                    ab: self.ab_flag,
                    ab_auto: self.ab_auto,
                    ct_enabled: self.ct_enabled,
                    af_list_mhz: parse_af_list(&self.af_list_text).0,
                    ps_scroll_enabled: self.ps_scroll_enabled,
                    ps_scroll_text: self.ps_scroll_text.clone(),
                    ps_scroll_cps: self.ps_scroll_cps,
                    rt_scroll_enabled: self.rt_scroll_enabled,
                    rt_scroll_text: self.rt_scroll_text.clone(),
                    rt_scroll_cps: self.rt_scroll_cps,
                    output_gain: self.output_gain,
                    limiter_enabled: self.limiter_enabled,
                    limiter_threshold: self.limiter_threshold,
                    limiter_lookahead: ((self.limiter_lookahead_ms / 1000.0) * 228000.0) as usize,
                    pilot_level: self.pilot_level,
                    rds_level: self.rds_level,
                    stereo_separation: self.stereo_separation,
                    preemphasis_tau: preemph_to_tau(self.preemphasis_selected.clone()),
                    compressor_enabled: self.compressor_enabled,
                    comp_threshold_db: self.comp_threshold,
                    comp_ratio: self.comp_ratio,
                    comp_attack: self.comp_attack,
                    comp_release: self.comp_release,
                    group_0a: self.group_0a.trim().parse::<usize>().unwrap_or(4),
                    group_2a: self.group_2a.trim().parse::<usize>().unwrap_or(1),
                    group_4a: self.group_4a.trim().parse::<usize>().unwrap_or(0),
                    ct_interval_groups: self.ct_interval_groups.trim().parse::<usize>().unwrap_or(0),
                    ps_alt_list: self.ps_alt_list_text
                        .split('|')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                    ps_alt_interval: self.ps_alt_interval.trim().parse::<usize>().unwrap_or(0),
                };
                match start_engine(config) {
                    Ok(engine) => {
                        self.engine = Some(engine);
                        self.status = "Streaming (192 kHz)".to_string();
                    }
                    Err(e) => {
                        self.status = format!("Stream error: {}", e);
                    }
                }
                Command::none()
            }
            Message::StopStream => {
                self.engine = None;
                self.status = "Stopped".to_string();
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let preset_names = self.presets.iter().map(|p| p.name.clone()).collect::<Vec<_>>();

        let tab_button = |label: &str, tab: Tab| {
            let selected = self.tab_selected == tab;
            button(text(label).size(14))
                .padding([8, 14])
                .style(theme::Button::Custom(Box::new(TabButton { selected })))
                .on_press(Message::TabSelected(tab))
        };

        let tabs = row![
            tab_button("Dashboard", Tab::Dashboard),
            tab_button("Audio", Tab::Audio),
            tab_button("RDS", Tab::Rds),
            tab_button("Processing", Tab::Processing),
            tab_button("Meters", Tab::Meters),
            tab_button("Export", Tab::Export),
            tab_button("RadioDNS", Tab::RadioDns),
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        let presets_card = card(
            "Presets",
            column![
                row![
                    text("Preset:"),
                    pick_list(preset_names.clone(), self.preset_selected.clone(), Message::PresetSelected),
                    button("Load")
                        .style(theme::Button::Custom(Box::new(GhostButton)))
                        .on_press(Message::LoadPreset),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("Name:"),
                    text_input("Preset name", &self.preset_name).on_input(Message::PresetNameChanged),
                    button("Save")
                        .style(theme::Button::Custom(Box::new(PrimaryButton)))
                        .on_press(Message::SavePreset),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
        );

        let stream_card = || {
            card(
            "Stream",
            column![
                row![
                    if self.engine.is_some() {
                        button("Streaming...")
                            .padding(10)
                            .style(theme::Button::Custom(Box::new(GhostButton)))
                    } else {
                        button("Start Stream")
                            .on_press(Message::StartStream)
                            .padding(10)
                            .style(theme::Button::Custom(Box::new(PrimaryButton)))
                    },
                    button("Stop")
                        .on_press(Message::StopStream)
                        .padding(10)
                        .style(theme::Button::Custom(Box::new(DangerButton))),
                    text(&self.status).style(color_muted()),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
            )
        };

        let device_card = || {
            card(
                "Devices",
                column![
                    row![
                        text("Input:"),
                        pick_list(self.input_devices.clone(), self.selected_input.clone(), Message::InputSelected),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Output:"),
                        pick_list(self.output_devices.clone(), self.selected_output.clone(), Message::OutputSelected),
                        button("Refresh")
                            .on_press(Message::RefreshDevices)
                            .style(theme::Button::Custom(Box::new(GhostButton))),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                ],
            )
        };

        let health_card = card(
            "Device Health",
            column![
                row![
                    text(format!("XRuns {}", self.xrun_count)).style(color_muted()),
                    text(format!("Buffer {:.0}%", (self.buffer_fill * 100.0).clamp(0.0, 100.0))).style(color_muted()),
                    text(format!("Latency {:.1} ms", self.latency_ms)).style(color_muted()),
                ]
                .spacing(14)
                .align_items(Alignment::Center),
            ],
        );

        let station_card = || {
            card(
            "Station",
            column![
                row![
                    text("PS:"),
                    text_input("BOUZIDFM", &self.ps).on_input(Message::PsChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("RT:"),
                    text_input("BOUZIDFM Sidi Bouzid 98.0 MHz", &self.rt).on_input(Message::RtChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("PI (hex):"),
                    text_input("7200", &self.pi_hex).on_input(Message::PiChanged),
                    checkbox("TP", self.tp, Message::TpChanged),
                    checkbox("TA", self.ta, Message::TaChanged),
                    checkbox("Music (MS)", self.ms, Message::MsChanged),
                    checkbox("CT", self.ct_enabled, Message::CtChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("PTY:"),
                    pick_list(self.pty_items.clone(), Some(self.pty_selected.clone()), Message::PtyChanged),
                    checkbox("RT A/B", self.ab_flag, Message::AbChanged),
                    checkbox("Auto A/B", self.ab_auto, Message::AbAutoChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
            )
        };

        let rds_identity_card = card(
            "Identity + DI",
            column![
                text("PI default uses Tunisia country code 7 (0x7xxx). Replace with your assigned PI.").style(color_muted()),
                row![
                    text("PI builder:"),
                    text_input("7", &self.pi_country_hex).on_input(Message::CountryCodeChanged),
                    text_input("2", &self.pi_area_hex).on_input(Message::AreaCodeChanged),
                    text_input("00", &self.pi_program_hex).on_input(Message::ProgramRefChanged),
                    button("Apply PI")
                        .on_press(Message::ApplyPiFromParts)
                        .style(theme::Button::Custom(Box::new(PrimaryButton))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("ECC (hex):"),
                    text_input("E2", &self.ecc_hex).on_input(Message::EccChanged),
                    text("DI:"),
                    checkbox("Stereo", self.di_stereo, Message::DiStereoChanged),
                    checkbox("Artificial head", self.di_artificial, Message::DiArtificialChanged),
                    checkbox("Compressed", self.di_compressed, Message::DiCompressedChanged),
                    checkbox("Dynamic PTY", self.di_dynamic, Message::DiDynamicChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
        );

        let rds_schedule_card = card(
            "Group Scheduling",
            column![
                row![
                    text("Mix 0A/2A/4A:"),
                    text_input("4", &self.group_0a).on_input(Message::Group0aChanged),
                    text_input("1", &self.group_2a).on_input(Message::Group2aChanged),
                    text_input("0", &self.group_4a).on_input(Message::Group4aChanged),
                    text("CT interval (groups):"),
                    text_input("0", &self.ct_interval_groups).on_input(Message::CtIntervalGroupsChanged),
                    button("Apply")
                        .on_press(Message::ApplyGroupMix)
                        .style(theme::Button::Custom(Box::new(PrimaryButton))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("Alternate PS:"),
                    text_input("ALT1|ALT2", &self.ps_alt_list_text).on_input(Message::PsAltListChanged),
                    text("Interval (groups):"),
                    text_input("0", &self.ps_alt_interval).on_input(Message::PsAltIntervalChanged),
                    button("Apply PS")
                        .on_press(Message::ApplyPsAlternates)
                        .style(theme::Button::Custom(Box::new(PrimaryButton))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
        );

        let af_card = card(
            "AF Helper",
            column![
                row![
                    text("Ref freq (MHz):"),
                    text_input("98.0", &self.frequency_mhz).on_input(Message::FrequencyChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("AF list (MHz):"),
                    text_input("98.0", &self.af_list_text).on_input(Message::AfListChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("Generate from:"),
                    text_input("Base", &self.af_base).on_input(Message::AfBaseChanged),
                    text_input("Spacing", &self.af_spacing).on_input(Message::AfSpacingChanged),
                    text_input("Count", &self.af_count).on_input(Message::AfCountChanged),
                    button("Generate")
                        .on_press(Message::AfGenerate)
                        .style(theme::Button::Custom(Box::new(GhostButton))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                if let Some(ref warning) = self.af_warning {
                    text(warning).style(color_accent_warm())
                } else {
                    text("")
                },
            ],
        );

        let scrolling_card = card(
            "Scrolling",
            column![
                row![
                    checkbox("PS scroll", self.ps_scroll_enabled, Message::PsScrollEnabled),
                    text_input("BOUZIDFM", &self.ps_scroll_text).on_input(Message::PsScrollTextChanged),
                    text(format!("{:.1} cps", self.ps_scroll_cps)),
                    slider(0.5..=10.0, self.ps_scroll_cps, Message::PsScrollSpeedChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    checkbox("RT scroll", self.rt_scroll_enabled, Message::RtScrollEnabled),
                    text_input("BOUZIDFM Sidi Bouzid 98.0 MHz", &self.rt_scroll_text).on_input(Message::RtScrollTextChanged),
                    text(format!("{:.1} cps", self.rt_scroll_cps)),
                    slider(0.5..=10.0, self.rt_scroll_cps, Message::RtScrollSpeedChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
        );

        let output_card = card(
            "Output",
            column![
                row![
                    text(format!("Gain {:.2}x", self.output_gain)),
                    slider(0.5..=2.0, self.output_gain, Message::GainChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    checkbox("Limiter", self.limiter_enabled, Message::LimiterEnabled),
                    text(format!("Threshold {:.2}", self.limiter_threshold)),
                    slider(0.5..=1.0, self.limiter_threshold, Message::LimiterThresholdChanged),
                    text(format!("Lookahead {:.1} ms", self.limiter_lookahead_ms)),
                    slider(0.5..=10.0, self.limiter_lookahead_ms, Message::LimiterLookaheadChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
        );

        let levels_card = card(
            "Stereo + RDS",
            column![
                row![
                    text(format!("Pilot {:.2}", self.pilot_level)),
                    slider(0.2..=1.5, self.pilot_level, Message::PilotLevelChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text(format!("RDS {:.2}", self.rds_level)),
                    slider(0.2..=1.5, self.rds_level, Message::RdsLevelChanged),
                    text(format!("Stereo sep {:.2}", self.stereo_separation)),
                    slider(0.5..=1.5, self.stereo_separation, Message::StereoSeparationChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
        );

        let processing_card = card(
            "Processing",
            column![
                row![
                    text("Pre-emphasis:"),
                    pick_list(self.preemphasis_items.clone(), Some(self.preemphasis_selected.clone()), Message::PreemphasisChanged),
                    checkbox("Compressor", self.compressor_enabled, Message::CompressorEnabled),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text(format!("Thr {:.1} dB", self.comp_threshold)),
                    slider(-30.0..=0.0, self.comp_threshold, Message::CompThresholdChanged),
                    text(format!("Ratio {:.1}", self.comp_ratio)),
                    slider(1.0..=6.0, self.comp_ratio, Message::CompRatioChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text(format!("Attack {:.3}s", self.comp_attack)),
                    slider(0.001..=0.1, self.comp_attack, Message::CompAttackChanged),
                    text(format!("Release {:.2}s", self.comp_release)),
                    slider(0.05..=1.0, self.comp_release, Message::CompReleaseChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
        );

        let meter_summary_card = || {
            card(
                "Meters",
                column![
                    row![
                        text(format!("RMS {:.2}", self.meter_rms)),
                        progress_bar(0.0..=1.0, self.meter_rms),
                        text(format!("Peak {:.2}", self.meter_peak)),
                        progress_bar(0.0..=1.0, self.meter_peak),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text(format!("Pilot 19 kHz {:.2}", self.meter_pilot)),
                        progress_bar(0.0..=1.0, self.meter_pilot),
                        text(format!("RDS 57 kHz {:.2}", self.meter_rds)),
                        progress_bar(0.0..=1.0, self.meter_rds),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                ],
            )
        };

        let meters_full = card_accent(
            "MPX Meter",
            column![
                row![
                    text(format!("RMS {:.2}", self.meter_rms)),
                    progress_bar(0.0..=1.0, self.meter_rms),
                    text(format!("Peak {:.2}", self.meter_peak)),
                    progress_bar(0.0..=1.0, self.meter_peak),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text(format!("Pilot 19 kHz {:.2}", self.meter_pilot)),
                    progress_bar(0.0..=1.0, self.meter_pilot),
                    text(format!("RDS 57 kHz {:.2}", self.meter_rds)),
                    progress_bar(0.0..=1.0, self.meter_rds),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text(format!("XRuns {}", self.xrun_count)),
                    text(format!("Buffer {:.0}%", (self.buffer_fill * 100.0).clamp(0.0, 100.0))),
                    text(format!("Latency {:.1} ms", self.latency_ms)),
                ]
                .spacing(14)
                .align_items(Alignment::Center),
                row![
                    text("Spectrum (dB):"),
                    Canvas::new(SpectrumView {
                        spectrum_peak_db: self.spectrum_peak_db.clone(),
                        spectrum_avg_db: self.spectrum_avg_db.clone(),
                    })
                    .width(Length::Fill)
                    .height(200),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("Scope:"),
                    Canvas::new(ScopeView { samples: self.scope_samples.clone(), prev: self.scope_prev.clone() })
                        .width(Length::Fill)
                        .height(180),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
        );

        let export_card = card(
            "WAV Export",
            column![
                row![
                    text("Duration (sec):"),
                    text_input("10", &self.duration).on_input(Message::DurationChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("Audio WAV (optional):"),
                    text_input("", &self.audio_path).on_input(Message::AudioChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("Output WAV:"),
                    text_input("mpx.wav", &self.output_path).on_input(Message::OutputChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                if self.generating {
                    button("Generating...")
                        .padding(10)
                        .style(theme::Button::Custom(Box::new(GhostButton)))
                } else {
                    button("Generate")
                        .on_press(Message::Generate)
                        .padding(10)
                        .style(theme::Button::Custom(Box::new(PrimaryButton)))
                },
            ],
        );

        let (rd_fqdn, rd_bearer, rd_lookup, rd_warning) = self.radiodns_fm_strings();

        let radiodns_info = card(
            "Project Logo (Station Logos)",
            column![
                text("RadioDNS Project Logo uses an SI.xml file to publish your station names, descriptions, and logos.").style(color_muted()),
                text("Required logo set (PNG): 32x32, 32x112, 128x128, 320x240, 600x600.").style(color_muted()),
                text("1) Create SI.xml with short/medium names, descriptions, and 5 logo sizes.").style(color_muted()),
                text("2) Host SI.xml at: /radiodns/spi/3.1/SI.xml (case sensitive).").style(color_muted()),
                text("3) Add a _radioepg._tcp SRV record pointing to your web server.").style(color_muted()),
                text("4) Register your stations with RadioDNS.").style(color_muted()),
                text("5) In SI.xml, include a bearer like: fm:<gcc>.<pi>.<freq>.").style(color_muted()),
                text("Automation: this app can generate SI.xml and placeholder logos into ./radiodns/.").style(color_muted()),
            ]
            .spacing(6),
        );

        let srv_line = build_srv_record_line(
            &self.radiodns_broadcaster_fqdn,
            &self.radiodns_srv_host,
            &self.radiodns_srv_port,
        );

        let cname_line = build_cname_line(rd_fqdn.as_deref(), &self.radiodns_broadcaster_fqdn);

        let radiodns_helper = card(
            "RadioDNS Helper (FM)",
            column![
                text("Uses your Frequency, PI, and ECC fields to build the FM RadioDNS lookup.").style(color_muted()),
                if let Some(ref warning) = rd_warning {
                    text(warning).style(color_accent_warm())
                } else {
                    text("")
                },
                text(format!("FQDN: {}", rd_fqdn.clone().unwrap_or_else(|| "—".to_string()))),
                text(format!("Bearer: {}", rd_bearer.unwrap_or_else(|| "—".to_string()))),
                text(format!("Lookup: {}", rd_lookup.unwrap_or_else(|| "—".to_string()))),
                text("Then lookup SRV on the returned broadcaster FQDN:").style(color_muted()),
                text("nslookup -type=SRV _radioepg._tcp.<broadcaster-fqdn>").style(color_muted()),
                text(format!("SRV record: {}", srv_line)).style(color_muted()),
                text(format!("CNAME: {}", cname_line)).style(color_muted()),
                row![
                    button("Copy FQDN")
                        .style(theme::Button::Custom(Box::new(GhostButton)))
                        .on_press(Message::RadioDnsCopyFqdn),
                    button("Copy Bearer")
                        .style(theme::Button::Custom(Box::new(GhostButton)))
                        .on_press(Message::RadioDnsCopyBearer),
                    button("Copy DNS Bundle")
                        .style(theme::Button::Custom(Box::new(GhostButton)))
                        .on_press(Message::RadioDnsCopyDnsBundle),
                    button("Copy CNAME")
                        .style(theme::Button::Custom(Box::new(GhostButton)))
                        .on_press(Message::RadioDnsCopyCname),
                    button("Copy SRV")
                        .style(theme::Button::Custom(Box::new(GhostButton)))
                        .on_press(Message::RadioDnsCopySrv),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                if let Some(ref path) = self.radiodns_last_output {
                    text(format!("Last output: {}", path)).style(color_muted())
                } else {
                    text("Last output: —").style(color_muted())
                },
                if self.radiodns_generating {
                    button("Generating...").padding(10).style(theme::Button::Custom(Box::new(GhostButton)))
                } else {
                    button("Generate RadioDNS Pack")
                        .padding(10)
                        .style(theme::Button::Custom(Box::new(PrimaryButton)))
                        .on_press(Message::GenerateRadioDnsPack)
                },
            ]
            .spacing(6),
        );

        let radiodns_automation = card(
            "Automation Settings",
            column![
                row![
                    text("Base URL:"),
                    text_input("https://your-domain.com", &self.radiodns_domain).on_input(Message::RadioDnsDomainChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("Logo source (optional):"),
                    text_input("/path/to/logo.png", &self.radiodns_logo_path).on_input(Message::RadioDnsLogoPathChanged),
                    button("Browse")
                        .style(theme::Button::Custom(Box::new(GhostButton)))
                        .on_press(Message::RadioDnsBrowseLogo),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("Broadcaster FQDN:"),
                    text_input("broadcaster.example.com", &self.radiodns_broadcaster_fqdn).on_input(Message::RadioDnsBroadcasterChanged),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("SRV target host:"),
                    text_input("radio.your-domain.com", &self.radiodns_srv_host).on_input(Message::RadioDnsSrvHostChanged),
                    text("Port:"),
                    text_input("80", &self.radiodns_srv_port).on_input(Message::RadioDnsSrvPortChanged),
                    button("Open Folder")
                        .style(theme::Button::Custom(Box::new(GhostButton)))
                        .on_press(Message::RadioDnsOpenFolder),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    button("Open SI.xml")
                        .style(theme::Button::Custom(Box::new(GhostButton)))
                        .on_press(Message::RadioDnsOpenSiXml),
                    button("Validate Pack")
                        .style(theme::Button::Custom(Box::new(PrimaryButton)))
                        .on_press(Message::RadioDnsValidatePack),
                    button("Copy SRV")
                        .style(theme::Button::Custom(Box::new(GhostButton)))
                        .on_press(Message::RadioDnsCopySrv),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                if let Some(ref msg) = self.radiodns_validation {
                    text(msg).style(color_muted())
                } else {
                    text("Validation: —").style(color_muted())
                },
                text("If logo source is set, the app will resize and generate all required sizes.").style(color_muted()),
            ]
            .spacing(8),
        );

        let radiodns_tab = column![
            row![
                column![radiodns_info, radiodns_automation].spacing(16).width(Length::FillPortion(3)),
                column![radiodns_helper].spacing(16).width(Length::FillPortion(2)),
            ]
            .spacing(16)
            .align_items(Alignment::Start),
        ];

        let status_pill = if self.engine.is_some() {
            pill("LIVE", color_live(), Color::from_rgb8(6, 24, 19))
        } else {
            pill("IDLE", color_surface_alt(), color_muted())
        };

        let hero = container(
            row![
                column![
                    text("Pulse FM").size(30).style(color_text()),
                    text("RDS Encoder").size(22).style(color_accent()),
                    text("Live MPX pipeline for macOS • 192 kHz device output").size(14).style(color_muted()),
                ]
                .spacing(4)
                .width(Length::FillPortion(3)),
                column![
                    row![
                        status_pill,
                        text(&self.status).style(color_muted()),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text(format!("XRuns {}", self.xrun_count)).style(color_muted()),
                        text(format!("Buffer {:.0}%", (self.buffer_fill * 100.0).clamp(0.0, 100.0))).style(color_muted()),
                        text(format!("Latency {:.1} ms", self.latency_ms)).style(color_muted()),
                    ]
                    .spacing(14)
                    .align_items(Alignment::Center),
                ]
                .spacing(8)
                .width(Length::FillPortion(2)),
            ]
            .spacing(24)
            .align_items(Alignment::Center),
        )
        .padding(16)
        .width(Length::Fill)
        .style(theme::Container::from(hero_style));

        let dashboard = column![
            row![
                column![stream_card(), device_card(), presets_card].spacing(16).width(Length::FillPortion(2)),
                column![station_card(), meter_summary_card()].spacing(16).width(Length::FillPortion(3)),
            ]
            .spacing(16)
            .align_items(Alignment::Start),
        ];

        let audio_tab = column![
            row![
                column![device_card(), stream_card(), health_card].spacing(16).width(Length::FillPortion(3)),
                column![meter_summary_card()].spacing(16).width(Length::FillPortion(2)),
            ]
            .spacing(16)
            .align_items(Alignment::Start),
        ];

        let rds_tab = column![
            row![
                column![station_card(), rds_identity_card].spacing(16).width(Length::FillPortion(3)),
                column![rds_schedule_card, af_card, scrolling_card].spacing(16).width(Length::FillPortion(2)),
            ]
            .spacing(16)
            .align_items(Alignment::Start),
        ];

        let processing_tab = column![
            row![
                column![output_card, levels_card].spacing(16).width(Length::FillPortion(3)),
                column![processing_card].spacing(16).width(Length::FillPortion(2)),
            ]
            .spacing(16)
            .align_items(Alignment::Start),
        ];

        let body: Element<'_, Message> = match self.tab_selected {
            Tab::Dashboard => dashboard.into(),
            Tab::Audio => audio_tab.into(),
            Tab::Rds => rds_tab.into(),
            Tab::Processing => processing_tab.into(),
            Tab::Meters => meters_full.into(),
            Tab::Export => export_card.into(),
            Tab::RadioDns => radiodns_tab.into(),
        };

        let content = column![
            hero,
            tabs,
            body,
        ]
        .spacing(18)
        .padding(24)
        .width(Length::Fill)
        .align_items(Alignment::Start);

        let scroll = scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill);

        container(scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .style(theme::Container::from(body_style))
            .into()
    }
}

impl App {
    fn di_bits(&self) -> u8 {
        let mut bits = 0u8;
        if self.di_stereo {
            bits |= 0b1000;
        }
        if self.di_artificial {
            bits |= 0b0100;
        }
        if self.di_compressed {
            bits |= 0b0010;
        }
        if self.di_dynamic {
            bits |= 0b0001;
        }
        bits
    }

    fn radiodns_fm_strings(&self) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
        let freq_mhz = self.frequency_mhz.trim().parse::<f32>().ok();
        let pi = parse_pi(&self.pi_hex).ok();
        let ecc = parse_hex_byte(&self.ecc_hex).ok();

        let mut warning = None;
        if freq_mhz.is_none() {
            warning = Some("Frequency is invalid. Expected a value like 98.0".to_string());
        } else if let Some(freq) = freq_mhz {
            if !(87.6..=107.9).contains(&freq) {
                warning = Some("Frequency is out of FM range (87.6–107.9 MHz).".to_string());
            }
        }
        if pi.is_none() {
            warning = Some("PI is invalid. Expected 4 hex digits (e.g., 7200).".to_string());
        }
        if ecc.is_none() {
            warning = Some("ECC is invalid. Expected 2 hex digits (e.g., E2).".to_string());
        }

        if let (Some(freq), Some(pi), Some(ecc)) = (freq_mhz, pi, ecc) {
            let freq_int = (freq * 100.0).round() as u32;
            let freq_str = format!("{:05}", freq_int);
            let pi_hex = format!("{:04x}", pi);
            let ecc_hex = format!("{:02x}", ecc);
            let gcc = format!("{}{}", pi_hex.chars().next().unwrap_or('0'), ecc_hex);
            let fqdn = format!("{freq}.{pi}.{gcc}.fm.radiodns.org", freq = freq_str, pi = pi_hex, gcc = gcc);
            let bearer = format!("fm:{gcc}.{pi}.{freq}", gcc = gcc, pi = pi_hex, freq = freq_str);
            let lookup = format!("nslookup -type=CNAME {fqdn}");
            return (Some(fqdn), Some(bearer), Some(lookup), warning);
        }

        (None, None, None, warning)
    }

    fn refresh_devices(&mut self) {
        match list_input_devices() {
            Ok(devices) => {
                if self.selected_input.is_none() && !devices.is_empty() {
                    self.selected_input = Some(devices[0].clone());
                }
                self.input_devices = devices;
            }
            Err(e) => {
                self.status = format!("Input device error: {}", e);
            }
        }

        match list_output_devices() {
            Ok(devices) => {
                if self.selected_output.is_none() && !devices.is_empty() {
                    self.selected_output = Some(devices[0].clone());
                }
                self.output_devices = devices;
            }
            Err(e) => {
                self.status = format!("Output device error: {}", e);
            }
        }
    }

    fn to_preset(&self) -> Preset {
        Preset {
            name: self.preset_name.clone(),
            ps: self.ps.clone(),
            rt: self.rt.clone(),
            pi_hex: self.pi_hex.clone(),
            tp: self.tp,
            ta: self.ta,
            pty: self.pty_selected.code,
            ms: self.ms,
            di: self.di_bits(),
            ab: self.ab_flag,
            ab_auto: self.ab_auto,
            ct_enabled: self.ct_enabled,
            af_list_text: self.af_list_text.clone(),
            ps_scroll_enabled: self.ps_scroll_enabled,
            ps_scroll_text: self.ps_scroll_text.clone(),
            ps_scroll_cps: self.ps_scroll_cps,
            rt_scroll_enabled: self.rt_scroll_enabled,
            rt_scroll_text: self.rt_scroll_text.clone(),
            rt_scroll_cps: self.rt_scroll_cps,
            output_gain: self.output_gain,
            limiter_enabled: self.limiter_enabled,
            limiter_threshold: self.limiter_threshold,
            limiter_lookahead_ms: self.limiter_lookahead_ms,
            pilot_level: self.pilot_level,
            rds_level: self.rds_level,
            stereo_separation: self.stereo_separation,
            preemphasis: self.preemphasis_selected.to_string(),
            compressor_enabled: self.compressor_enabled,
            comp_threshold: self.comp_threshold,
            comp_ratio: self.comp_ratio,
            comp_attack: self.comp_attack,
            comp_release: self.comp_release,
            group_0a: self.group_0a.clone(),
            group_2a: self.group_2a.clone(),
            group_4a: self.group_4a.clone(),
            ct_interval_groups: self.ct_interval_groups.clone(),
            ps_alt_list_text: self.ps_alt_list_text.clone(),
            ps_alt_interval: self.ps_alt_interval.clone(),
        }
    }

    fn apply_preset(&mut self, p: Preset) {
        self.preset_name = p.name.clone();
        self.ps = p.ps;
        self.rt = p.rt;
        self.pi_hex = p.pi_hex;
        self.tp = p.tp;
        self.ta = p.ta;
        self.ms = p.ms;
        if let Some(item) = self.pty_items.iter().find(|i| i.code == p.pty).cloned() {
            self.pty_selected = item;
        }
        self.ab_flag = p.ab;
        self.ab_auto = p.ab_auto;
        self.ct_enabled = p.ct_enabled;
        self.af_list_text = p.af_list_text;
        self.ps_scroll_enabled = p.ps_scroll_enabled;
        self.ps_scroll_text = p.ps_scroll_text;
        self.ps_scroll_cps = p.ps_scroll_cps;
        self.rt_scroll_enabled = p.rt_scroll_enabled;
        self.rt_scroll_text = p.rt_scroll_text;
        self.rt_scroll_cps = p.rt_scroll_cps;
        self.output_gain = p.output_gain;
        self.limiter_enabled = p.limiter_enabled;
        self.limiter_threshold = p.limiter_threshold;
        self.limiter_lookahead_ms = p.limiter_lookahead_ms;
        self.pilot_level = p.pilot_level;
        self.rds_level = p.rds_level;
        self.stereo_separation = p.stereo_separation;
        self.preemphasis_selected = match p.preemphasis.as_str() {
            "50 µs" => Preemphasis::Us50,
            "75 µs" => Preemphasis::Us75,
            _ => Preemphasis::Off,
        };
        self.compressor_enabled = p.compressor_enabled;
        self.comp_threshold = p.comp_threshold;
        self.comp_ratio = p.comp_ratio;
        self.comp_attack = p.comp_attack;
        self.comp_release = p.comp_release;
        self.group_0a = p.group_0a;
        self.group_2a = p.group_2a;
        self.group_4a = p.group_4a;
        self.ct_interval_groups = p.ct_interval_groups;
        self.ps_alt_list_text = p.ps_alt_list_text;
        self.ps_alt_interval = p.ps_alt_interval;

        // Apply to engine if running
        if let Some(engine) = &self.engine {
            if let Ok(pi) = parse_pi(&self.pi_hex) {
                engine.update_pi(pi);
            }
            engine.update_ps(&self.ps);
            engine.update_rt(&self.rt);
            engine.update_tp(self.tp);
            engine.update_ta(self.ta);
            engine.update_pty(self.pty_selected.code);
            engine.update_ms(self.ms);
            engine.update_ab(self.ab_flag);
            engine.update_ab_auto(self.ab_auto);
            engine.update_ct_enabled(self.ct_enabled);
            engine.update_af_list(&parse_af_list(&self.af_list_text).0);
            engine.update_ps_scroll(self.ps_scroll_enabled, &self.ps_scroll_text, self.ps_scroll_cps);
            engine.update_rt_scroll(self.rt_scroll_enabled, &self.rt_scroll_text, self.rt_scroll_cps);
            engine.update_gain(self.output_gain);
            engine.update_limiter(self.limiter_enabled, self.limiter_threshold);
            engine.update_limiter_lookahead(((self.limiter_lookahead_ms / 1000.0) * 228000.0) as usize);
            engine.update_pilot_level(self.pilot_level);
            engine.update_rds_level(self.rds_level);
            engine.update_stereo_separation(self.stereo_separation);
            engine.update_preemphasis(preemph_to_tau(self.preemphasis_selected.clone()));
            engine.update_compressor(self.compressor_enabled, self.comp_threshold, self.comp_ratio, self.comp_attack, self.comp_release);
            engine.update_group_mix(
                self.group_0a.trim().parse::<usize>().unwrap_or(4),
                self.group_2a.trim().parse::<usize>().unwrap_or(1),
                self.group_4a.trim().parse::<usize>().unwrap_or(0),
            );
            engine.update_ct_interval(self.ct_interval_groups.trim().parse::<usize>().unwrap_or(0));
            let list = self.ps_alt_list_text
                .split('|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            engine.update_ps_alternates(list, self.ps_alt_interval.trim().parse::<usize>().unwrap_or(0));
        }
    }
}

fn parse_pi(input: &str) -> Result<u16, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("PI code is required".to_string());
    }
    let t = t.strip_prefix("0x").unwrap_or(t);
    u16::from_str_radix(t, 16).map_err(|_| "PI must be a 4-hex-digit value".to_string())
}

fn parse_hex_byte(input: &str) -> Result<u8, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("Hex byte is required".to_string());
    }
    let t = t.strip_prefix("0x").unwrap_or(t);
    u8::from_str_radix(t, 16).map_err(|_| "Hex must be 2 digits".to_string())
}

fn generate_radiodns_pack(
    ps: String,
    rt: String,
    frequency_mhz: String,
    pi_hex: String,
    ecc_hex: String,
    domain: String,
    logo_path: String,
    srv_host: String,
    srv_port: String,
    broadcaster_fqdn: String,
) -> Result<String, String> {
    let freq = frequency_mhz.trim().parse::<f32>().map_err(|_| "Frequency is invalid".to_string())?;
    if !(87.6..=107.9).contains(&freq) {
        return Err("Frequency is out of FM range (87.6–107.9 MHz).".to_string());
    }
    let pi = parse_pi(&pi_hex)?;
    let ecc = parse_hex_byte(&ecc_hex)?;
    let base_url = domain.trim();
    if base_url.is_empty() {
        return Err("Base URL is required (e.g., https://your-domain.com).".to_string());
    }

    let freq_int = (freq * 100.0).round() as u32;
    let freq_str = format!("{:05}", freq_int);
    let pi_hex_lower = format!("{:04x}", pi);
    let ecc_hex_lower = format!("{:02x}", ecc);
    let gcc = format!("{}{}", pi_hex_lower.chars().next().unwrap_or('0'), ecc_hex_lower);
    let bearer = format!("fm:{gcc}.{pi}.{freq}", gcc = gcc, pi = pi_hex_lower, freq = freq_str);
    let fqdn = format!("{freq}.{pi}.{gcc}.fm.radiodns.org", freq = freq_str, pi = pi_hex_lower, gcc = gcc);

    let base_dir = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("radiodns");
    let logos_dir = base_dir.join("logos");
    fs::create_dir_all(&logos_dir).map_err(|e| e.to_string())?;

    let station_name = ps.trim();
    let description = if rt.trim().is_empty() { station_name } else { rt.trim() };
    let base_url = base_url.trim_end_matches('/');
    let si_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<serviceInformation xmlns="http://www.worlddab.org/schemas/spi/31" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.worlddab.org/schemas/spi/31 spi_31.xsd">
  <services>
    <service>
      <name short="{name}" medium="{name}" long="{name}"/>
      <description short="{desc}" long="{desc}"/>
      <bearer id="{bearer}"/>
      <media>
        <image id="logo_32x32" type="logo_unrestricted" width="32" height="32" mime="image/png">{base}/radiodns/logos/logo_32x32.png</image>
        <image id="logo_32x112" type="logo_unrestricted" width="32" height="112" mime="image/png">{base}/radiodns/logos/logo_32x112.png</image>
        <image id="logo_128x128" type="logo_unrestricted" width="128" height="128" mime="image/png">{base}/radiodns/logos/logo_128x128.png</image>
        <image id="logo_320x240" type="logo_unrestricted" width="320" height="240" mime="image/png">{base}/radiodns/logos/logo_320x240.png</image>
        <image id="logo_600x600" type="logo_unrestricted" width="600" height="600" mime="image/png">{base}/radiodns/logos/logo_600x600.png</image>
      </media>
    </service>
  </services>
</serviceInformation>
"#,
        name = station_name,
        desc = description,
        bearer = bearer,
        base = base_url,
    );
    fs::write(base_dir.join("SI.xml"), si_xml).map_err(|e| e.to_string())?;

    let logo_path = logo_path.trim();
    let sizes: &[(u32, u32)] = &[(32, 32), (32, 112), (128, 128), (320, 240), (600, 600)];
    if logo_path.is_empty() {
        for (w, h) in sizes {
            let name = format!("logo_{}x{}.png", w, h);
            let path = logos_dir.join(name);
            let img = RgbaImage::from_pixel(*w, *h, Rgba([0, 0, 0, 0]));
            img.save(&path).map_err(|e| e.to_string())?;
        }
    } else {
        let source = image::open(logo_path).map_err(|e| format!("Logo load failed: {}", e))?;
        for (w, h) in sizes {
            let name = format!("logo_{}x{}.png", w, h);
            let path = logos_dir.join(name);
            let resized = source.resize_exact(*w, *h, image::imageops::FilterType::Lanczos3);
            resized.save(&path).map_err(|e| e.to_string())?;
        }
    }

    let srv_port = if srv_port.trim().is_empty() { "80" } else { srv_port.trim() };
    let srv_host = srv_host.trim();
    let srv_line = if broadcaster_fqdn.trim().is_empty() {
        "Broadcaster FQDN not set. Add it in the UI to generate SRV line.".to_string()
    } else {
        format!(
            "_radioepg._tcp.{broadcaster} 86400 IN SRV 0 0 {port} {host}.",
            broadcaster = broadcaster_fqdn.trim(),
            port = srv_port,
            host = srv_host
        )
    };

    let readme = format!(
        "RadioDNS Pack\n\
\n\
Output folder: {dir}\n\
FM FQDN: {fqdn}\n\
Bearer: {bearer}\n\
SRV record: {srv_line}\n\
\n\
Next steps:\n\
1) Verify the Base URL in SI.xml matches your web domain.\n\
2) Upload SI.xml to /radiodns/spi/3.1/SI.xml (case sensitive).\n\
3) Upload logos to /radiodns/logos/.\n\
4) Create _radioepg._tcp SRV record pointing to your web server.\n\
5) Validate with RadioDNS.\n",
        dir = base_dir.display(),
        fqdn = fqdn,
        bearer = bearer,
        srv_line = srv_line
    );
    fs::write(base_dir.join("README.txt"), readme).map_err(|e| e.to_string())?;

    Ok(base_dir.display().to_string())
}

fn parse_af_list(input: &str) -> (Vec<f32>, Option<String>) {
    let mut out = Vec::new();
    let mut invalid = false;
    for part in input.split(',') {
        if part.trim().is_empty() {
            continue;
        }
        if let Ok(freq) = part.trim().parse::<f32>() {
            if (87.6..=107.9).contains(&freq) {
                out.push(freq);
            } else {
                invalid = true;
            }
        } else {
            invalid = true;
        }
    }
    let warning = if invalid {
        Some("AF list: invalid entries were ignored (valid range 87.6–107.9 MHz).".to_string())
    } else {
        None
    };
    (out, warning)
}

fn derive_host_from_base_url(base_url: &str) -> Option<String> {
    let trimmed = base_url.trim();
    if trimmed.is_empty() {
        return None;
    }
    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    let host = without_scheme.split('/').next().unwrap_or("").trim();
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

fn build_srv_record_line(broadcaster_fqdn: &str, srv_host: &str, srv_port: &str) -> String {
    if broadcaster_fqdn.trim().is_empty() {
        return "—".to_string();
    }
    let port = srv_port.trim();
    let port = if port.is_empty() { "80" } else { port };
    let host = srv_host.trim();
    format!(
        "_radioepg._tcp.{broadcaster} 86400 IN SRV 0 0 {port} {host}.",
        broadcaster = broadcaster_fqdn.trim(),
        port = port,
        host = host
    )
}

fn build_cname_line(fm_fqdn: Option<&str>, broadcaster_fqdn: &str) -> String {
    let fm_fqdn = match fm_fqdn {
        Some(v) if !v.trim().is_empty() => v.trim(),
        _ => return "—".to_string(),
    };
    if broadcaster_fqdn.trim().is_empty() {
        return "—".to_string();
    }
    format!("{fm} 86400 IN CNAME {broadcaster}.", fm = fm_fqdn, broadcaster = broadcaster_fqdn.trim())
}

fn build_dns_bundle(
    fm_fqdn: Option<&str>,
    broadcaster_fqdn: &str,
    srv_host: &str,
    srv_port: &str,
) -> String {
    let cname = build_cname_line(fm_fqdn, broadcaster_fqdn);
    let srv = build_srv_record_line(broadcaster_fqdn, srv_host, srv_port);
    format!("{}\n{}", cname, srv)
}

fn open_in_file_manager(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut cmd = std::process::Command::new("open");
    #[cfg(target_os = "windows")]
    let mut cmd = std::process::Command::new("explorer");
    #[cfg(target_os = "linux")]
    let mut cmd = std::process::Command::new("xdg-open");

    let status = cmd.arg(path).status().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Command failed with status {:?}", status.code()))
    }
}

fn validate_radiodns_pack() -> Result<String, String> {
    let base_dir = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("radiodns");
    let si_path = base_dir.join("SI.xml");
    let logos_dir = base_dir.join("logos");
    if !si_path.exists() {
        return Err("SI.xml not found in ./radiodns".to_string());
    }
    if !logos_dir.exists() {
        return Err("logos/ folder not found in ./radiodns".to_string());
    }

    let mut missing = Vec::new();
    let sizes: &[(u32, u32)] = &[(32, 32), (32, 112), (128, 128), (320, 240), (600, 600)];
    for (w, h) in sizes {
        let name = format!("logo_{}x{}.png", w, h);
        let path = logos_dir.join(&name);
        if !path.exists() {
            missing.push(name);
            continue;
        }
        let img = image::open(&path).map_err(|e| format!("Logo read failed: {} ({})", name, e))?;
        let (iw, ih) = img.dimensions();
        if iw != *w || ih != *h {
            return Err(format!("Logo {} has wrong size ({}x{}).", name, iw, ih));
        }
    }

    if !missing.is_empty() {
        return Err(format!("Missing logos: {}", missing.join(", ")));
    }

    Ok("Validation OK: SI.xml and all logos present with correct sizes.".to_string())
}

fn build_pi_from_parts(country_hex: &str, area_hex: &str, program_hex: &str, ecc_hex: &str) -> Result<u16, String> {
    let country = if country_hex.trim().is_empty() {
        let ecc = ecc_hex.trim().trim_start_matches("0x").to_uppercase();
        if ecc == "E2" {
            0x7
        } else {
            return Err("Country code is empty and ECC is unknown. Set country code manually.".to_string());
        }
    } else {
        u16::from_str_radix(country_hex.trim().trim_start_matches("0x"), 16)
            .map_err(|_| "Invalid country code hex".to_string())?
    };
    let area = u16::from_str_radix(area_hex.trim().trim_start_matches("0x"), 16)
        .map_err(|_| "Invalid area code hex".to_string())?;
    let program = u16::from_str_radix(program_hex.trim().trim_start_matches("0x"), 16)
        .map_err(|_| "Invalid program ref hex".to_string())?;

    if country > 0xF {
        return Err("Country code must be 0..F".to_string());
    }
    if area > 0xF {
        return Err("Area code must be 0..F".to_string());
    }
    if program > 0xFF {
        return Err("Program ref must be 00..FF".to_string());
    }
    Ok((country << 12) | (area << 8) | program)
}

fn preemph_to_tau(mode: Preemphasis) -> Option<f32> {
    match mode {
        Preemphasis::Off => None,
        Preemphasis::Us50 => Some(50e-6),
        Preemphasis::Us75 => Some(75e-6),
    }
}

fn card<'a>(title: &str, content: Column<'a, Message>) -> Element<'a, Message> {
    container(
        column![
            container(text(title).size(15).style(color_text()))
                .padding([6, 10])
                .width(Length::Fill)
                .style(theme::Container::from(header_style)),
            content.spacing(12),
        ]
        .spacing(12),
    )
    .padding(14)
    .width(Length::Fill)
    .style(theme::Container::from(card_style))
    .into()
}

fn card_accent<'a>(title: &str, content: Column<'a, Message>) -> Element<'a, Message> {
    container(
        column![
            container(text(title).size(15).style(color_text()))
                .padding([6, 10])
                .width(Length::Fill)
                .style(theme::Container::from(header_style)),
            content.spacing(12),
        ]
        .spacing(12),
    )
    .padding(14)
    .width(Length::Fill)
    .style(theme::Container::from(card_accent_style))
    .into()
}

fn header_style(_theme: &Theme) -> container_widget::Appearance {
    container_widget::Appearance {
        background: Some(Background::Color(color_surface_alt())),
        text_color: Some(color_text()),
        border_radius: 10.0.into(),
        border_width: 1.0,
        border_color: color_border(),
    }
}

fn card_style(_theme: &Theme) -> container_widget::Appearance {
    container_widget::Appearance {
        background: Some(Background::Color(color_surface())),
        text_color: Some(color_text()),
        border_radius: 14.0.into(),
        border_width: 1.0,
        border_color: color_border(),
    }
}

fn card_accent_style(_theme: &Theme) -> container_widget::Appearance {
    container_widget::Appearance {
        background: Some(Background::Color(color_surface())),
        text_color: Some(color_text()),
        border_radius: 14.0.into(),
        border_width: 2.0,
        border_color: color_accent(),
    }
}

fn hero_style(_theme: &Theme) -> container_widget::Appearance {
    container_widget::Appearance {
        background: Some(Background::Color(color_surface())),
        text_color: Some(color_text()),
        border_radius: 16.0.into(),
        border_width: 1.0,
        border_color: color_accent(),
    }
}

fn body_style(_theme: &Theme) -> container_widget::Appearance {
    container_widget::Appearance {
        background: Some(Background::Color(color_bg())),
        text_color: Some(color_text()),
        ..Default::default()
    }
}

fn pill<'a>(label: &str, bg: Color, fg: Color) -> Element<'a, Message> {
    container(text(label).size(12))
        .padding([4, 10])
        .style(theme::Container::Custom(Box::new(PillStyle { bg, fg })))
        .into()
}

struct PrimaryButton;

impl button_widget::StyleSheet for PrimaryButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button_widget::Appearance {
        button_widget::Appearance {
            background: Some(Background::Color(color_accent())),
            text_color: Color::from_rgb8(6, 16, 20),
            border_radius: 10.0.into(),
            border_width: 1.0,
            border_color: color_accent(),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(Color::from_rgb8(74, 222, 239)));
        active
    }

    fn pressed(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(Color::from_rgb8(22, 189, 214)));
        active
    }
}

struct GhostButton;

impl button_widget::StyleSheet for GhostButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button_widget::Appearance {
        button_widget::Appearance {
            background: Some(Background::Color(color_surface_alt())),
            text_color: color_text(),
            border_radius: 10.0.into(),
            border_width: 1.0,
            border_color: color_border(),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(Color::from_rgb8(36, 46, 60)));
        active
    }

    fn pressed(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(Color::from_rgb8(28, 38, 50)));
        active
    }
}

struct DangerButton;

impl button_widget::StyleSheet for DangerButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button_widget::Appearance {
        button_widget::Appearance {
            background: Some(Background::Color(color_danger())),
            text_color: Color::WHITE,
            border_radius: 10.0.into(),
            border_width: 1.0,
            border_color: color_danger(),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(Color::from_rgb8(248, 113, 113)));
        active
    }

    fn pressed(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(Color::from_rgb8(220, 38, 38)));
        active
    }
}

struct TabButton {
    selected: bool,
}

impl button_widget::StyleSheet for TabButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button_widget::Appearance {
        let (bg, text_color, border_color) = if self.selected {
            (color_accent(), Color::from_rgb8(6, 16, 20), color_accent())
        } else {
            (color_surface_alt(), color_text(), color_border())
        };
        button_widget::Appearance {
            background: Some(Background::Color(bg)),
            text_color,
            border_radius: 10.0.into(),
            border_width: 1.0,
            border_color,
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        if !self.selected {
            active.background = Some(Background::Color(Color::from_rgb8(36, 46, 60)));
        }
        active
    }

    fn pressed(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        if !self.selected {
            active.background = Some(Background::Color(Color::from_rgb8(28, 38, 50)));
        }
        active
    }
}

struct PillStyle {
    bg: Color,
    fg: Color,
}

impl container_widget::StyleSheet for PillStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container_widget::Appearance {
        container_widget::Appearance {
            background: Some(Background::Color(self.bg)),
            text_color: Some(self.fg),
            border_radius: 999.0.into(),
            border_width: 1.0,
            border_color: self.bg,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Preset {
    name: String,
    ps: String,
    rt: String,
    pi_hex: String,
    tp: bool,
    ta: bool,
    pty: u8,
    ms: bool,
    di: u8,
    ab: bool,
    ab_auto: bool,
    ct_enabled: bool,
    af_list_text: String,
    ps_scroll_enabled: bool,
    ps_scroll_text: String,
    ps_scroll_cps: f32,
    rt_scroll_enabled: bool,
    rt_scroll_text: String,
    rt_scroll_cps: f32,
    output_gain: f32,
    limiter_enabled: bool,
    limiter_threshold: f32,
    limiter_lookahead_ms: f32,
    pilot_level: f32,
    rds_level: f32,
    stereo_separation: f32,
    preemphasis: String,
    compressor_enabled: bool,
    comp_threshold: f32,
    comp_ratio: f32,
    comp_attack: f32,
    comp_release: f32,
    group_0a: String,
    group_2a: String,
    group_4a: String,
    ct_interval_groups: String,
    ps_alt_list_text: String,
    ps_alt_interval: String,
}

fn presets_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("presets.json")
}

fn load_presets() -> Result<Vec<Preset>, String> {
    let path = presets_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

fn save_presets(presets: &[Preset]) -> Result<(), String> {
    let data = serde_json::to_string_pretty(presets).map_err(|e| e.to_string())?;
    fs::write(presets_path(), data).map_err(|e| e.to_string())
}

struct SpectrumView {
    spectrum_peak_db: Vec<f32>,
    spectrum_avg_db: Vec<f32>,
}

impl<Message> Program<Message, Renderer> for SpectrumView {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let bg = Path::rectangle(iced::Point::ORIGIN, frame.size());
        frame.fill(&bg, Color::from_rgb8(22, 22, 26));

        let width = frame.size().width;
        let height = frame.size().height;

        let grid_color = Color::from_rgb8(60, 30, 70);
        for i in 0..=6 {
            let y = height * (i as f32 / 6.0);
            let line = Path::line(iced::Point::new(0.0, y), iced::Point::new(width, y));
            frame.stroke(&line, Stroke::default().with_width(1.0).with_color(grid_color));
        }

        let labels = [-60.0, -40.0, -20.0, 0.0];
        for (i, db) in labels.iter().enumerate() {
            let y = height - (height * (i as f32 / 3.0));
            frame.fill_text(Text {
                content: format!("{:>3} dB", db),
                position: iced::Point::new(6.0, y - 4.0),
                color: Color::from_rgb8(160, 160, 170),
                size: 12.0,
                ..Text::default()
            });
        }

        let draw_line = |frame: &mut Frame, data: &[f32], color: Color, width: f32| {
            if data.len() < 2 {
                return;
            }
            let step = frame.size().width / (data.len() as f32 - 1.0);
            let path = Path::new(|builder| {
                for (i, db) in data.iter().enumerate() {
                    let unit = (db.clamp(-60.0, 0.0) + 60.0) / 60.0;
                    let x = i as f32 * step;
                    let y = height - unit * height;
                    if i == 0 {
                        builder.move_to(iced::Point::new(x, y));
                    } else {
                        builder.line_to(iced::Point::new(x, y));
                    }
                }
            });
            frame.stroke(&path, Stroke::default().with_width(width).with_color(color));
        };

        draw_line(&mut frame, &self.spectrum_avg_db, Color::from_rgb8(0, 190, 255), 2.0);
        draw_line(&mut frame, &self.spectrum_peak_db, Color::from_rgb8(255, 120, 0), 1.0);

        let rds_x = width * (57000.0 / 96000.0);
        let rds_line = Path::line(
            iced::Point::new(rds_x, 0.0),
            iced::Point::new(rds_x, height),
        );
        frame.stroke(&rds_line, Stroke::default().with_width(2.0).with_color(Color::from_rgb8(255, 140, 0)));
        frame.fill_text(Text {
            content: "RDS 57k".to_string(),
            position: iced::Point::new(rds_x + 6.0, 8.0),
            color: Color::from_rgb8(255, 170, 40),
            size: 12.0,
            ..Text::default()
        });

        let markers = [0.0, 19000.0, 38000.0, 57000.0, 76000.0, 95000.0];
        for freq in markers {
            let x = width * (freq / 96000.0);
            let line = Path::line(iced::Point::new(x, 0.0), iced::Point::new(x, height));
            frame.stroke(&line, Stroke::default().with_width(1.0).with_color(Color::from_rgb8(50, 40, 60)));
            frame.fill_text(Text {
                content: format!("{:.0}k", freq / 1000.0),
                position: iced::Point::new(x + 4.0, height - 14.0),
                color: Color::from_rgb8(160, 160, 170),
                size: 11.0,
                ..Text::default()
            });
        }

        vec![frame.into_geometry()]
    }
}

struct ScopeView {
    samples: Vec<f32>,
    prev: Vec<f32>,
}

impl<Message> Program<Message, Renderer> for ScopeView {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let bg = Path::rectangle(iced::Point::ORIGIN, frame.size());
        frame.fill(&bg, Color::from_rgb8(18, 18, 20));

        let width = frame.size().width;
        let height = frame.size().height;

        let mid_y = height / 2.0;
        let mid_line = Path::line(iced::Point::new(0.0, mid_y), iced::Point::new(width, mid_y));
        frame.stroke(&mid_line, Stroke::default().with_width(1.0).with_color(Color::from_rgb8(60, 60, 70)));

        let draw_trace = |frame: &mut Frame, data: &[f32], width: f32, mid_y: f32, color: Color, thickness: f32| {
            if data.len() < 2 {
                return;
            }
            let step = width / (data.len() as f32 - 1.0);
            let path = Path::new(|builder| {
                for (i, s) in data.iter().enumerate() {
                    let x = i as f32 * step;
                    let y = mid_y - (s.clamp(-1.0, 1.0) * mid_y);
                    if i == 0 {
                        builder.move_to(iced::Point::new(x, y));
                    } else {
                        builder.line_to(iced::Point::new(x, y));
                    }
                }
            });
            frame.stroke(&path, Stroke::default().with_width(thickness).with_color(color));
        };

        draw_trace(
            &mut frame,
            &self.prev,
            width,
            mid_y,
            Color::from_rgba(0.0, 1.0, 0.55, 0.2),
            6.0,
        );
        draw_trace(
            &mut frame,
            &self.samples,
            width,
            mid_y,
            Color::from_rgba(0.0, 1.0, 0.6, 0.35),
            3.5,
        );
        draw_trace(
            &mut frame,
            &self.samples,
            width,
            mid_y,
            Color::from_rgb8(0, 255, 140),
            1.5,
        );

        vec![frame.into_geometry()]
    }
}
