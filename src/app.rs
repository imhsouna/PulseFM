use iced::widget::{button, checkbox, column, container, pick_list, progress_bar, row, scrollable, slider, text, text_input, Column};
use iced::widget::button as button_widget;
use iced::widget::container as container_widget;
use iced::widget::slider as slider_widget;
use iced::widget::text_input as text_input_widget;
use iced::widget::progress_bar as progress_bar_widget;
use iced::widget::scrollable as scrollable_widget;
use iced::{Alignment, Background, Command, Element, Length, Theme};
use iced::theme;
use iced::Event;
use iced::window;
use serde::{Deserialize, Serialize};
use rand::Rng;
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
pub(crate) struct CountryItem {
    label: &'static str,
    country_hex: Option<&'static str>,
    ecc_hex: Option<&'static str>,
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
    About,
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
            Tab::About => write!(f, "About"),
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

impl std::fmt::Display for CountryItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
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

fn country_items() -> Vec<CountryItem> {
    vec![
        CountryItem {
            label: "Tunisia (7 / E2)",
            country_hex: Some("7"),
            ecc_hex: Some("E2"),
        },
        CountryItem {
            label: "Custom (enter manually)",
            country_hex: None,
            ecc_hex: None,
        },
    ]
}

fn color_bg() -> Color {
    Color::from_rgb8(5, 7, 15)
}

fn color_surface() -> Color {
    Color::from_rgb8(13, 18, 30)
}

fn color_surface_alt() -> Color {
    Color::from_rgb8(19, 26, 42)
}

fn color_border() -> Color {
    Color::from_rgb8(35, 48, 75)
}

fn color_text() -> Color {
    Color::from_rgb8(240, 244, 252)
}

fn color_muted() -> Color {
    Color::from_rgb8(120, 145, 180)
}

fn color_accent() -> Color {
    Color::from_rgb8(56, 189, 248)
}

fn color_accent_warm() -> Color {
    Color::from_rgb8(251, 146, 60)
}

fn color_live() -> Color {
    Color::from_rgb8(52, 211, 153)
}

fn color_danger() -> Color {
    Color::from_rgb8(248, 113, 113)
}

fn color_gradient_end() -> Color {
    Color::from_rgb8(168, 85, 247)
}

fn rgba8f(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::from_rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
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
    CountrySelected(CountryItem),
    GenerateRandomPi,
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

    CopyPi,
    WindowResized(u32, u32),
    NoOp,
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
    country_items: Vec<CountryItem>,
    country_selected: CountryItem,
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
    window_width: f32,
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
            country_items: country_items(),
            country_selected: CountryItem {
                label: "Tunisia (7 / E2)",
                country_hex: Some("7"),
                ecc_hex: Some("E2"),
            },
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
            window_width: 1200.0,
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
        iced::Subscription::batch(vec![
            iced::time::every(Duration::from_millis(200)).map(|_| Message::Tick),
            iced::subscription::events().map(|event| match event {
                Event::Window(window::Event::Resized { width, height: _ }) => Message::WindowResized(width, 0),
                _ => Message::NoOp,
            }),
        ])
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
            Message::CountrySelected(item) => {
                self.country_selected = item.clone();
                if let Some(country) = item.country_hex {
                    self.pi_country_hex = country.to_string();
                }
                if let Some(ecc) = item.ecc_hex {
                    self.ecc_hex = ecc.to_string();
                }
                Command::none()
            }
            Message::GenerateRandomPi => {
                let country_hex = self.country_selected.country_hex.unwrap_or(self.pi_country_hex.trim());
                let country = u16::from_str_radix(country_hex.trim_start_matches("0x"), 16).unwrap_or(0x7);
                let area = rand::thread_rng().gen_range(0u16..=0xF);
                let program = rand::thread_rng().gen_range(0u16..=0xFF);
                let pi = (country << 12) | (area << 8) | program;
                self.pi_country_hex = format!("{:X}", country);
                self.pi_area_hex = format!("{:X}", area);
                self.pi_program_hex = format!("{:02X}", program);
                self.pi_hex = format!("{:04X}", pi);
                if let Some(engine) = &self.engine {
                    engine.update_pi(pi);
                }
                self.status = "Random PI generated (testing only)".to_string();
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

            Message::CopyPi => {
                let pi = self.pi_hex.trim();
                if pi.is_empty() {
                    self.status = "PI is empty".to_string();
                    return Command::none();
                }
                self.status = "PI copied".to_string();
                Command::batch(vec![iced::clipboard::write(pi.to_string())])
            }
            Message::WindowResized(width, _height) => {
                self.window_width = width as f32;
                Command::none()
            }
            Message::NoOp => Command::none(),

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
                if let Some(engine) = &self.engine {
                    engine.stop();
                }
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
            tab_button("About", Tab::About),
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        let presets_card = || {
            card(
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
                        text_input("Preset name", &self.preset_name).on_input(Message::PresetNameChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                        button("Save")
                            .style(theme::Button::Custom(Box::new(PrimaryButton)))
                            .on_press(Message::SavePreset),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                ],
            )
        };

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
                    text_input("BOUZIDFM", &self.ps).on_input(Message::PsChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("RT:"),
                    text_input("BOUZIDFM Sidi Bouzid 98.0 MHz", &self.rt).on_input(Message::RtChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("PI (hex):"),
                    text_input("7200", &self.pi_hex).on_input(Message::PiChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
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

        let pi_preview = build_pi_from_parts(&self.pi_country_hex, &self.pi_area_hex, &self.pi_program_hex, &self.ecc_hex)
            .map(|pi| format!("{:04X}", pi))
            .unwrap_or_else(|_| "—".to_string());

        let rds_identity_card = || {
            card(
                "Identity + DI",
                column![
                    text("PI (Program Identification) should come from your regulator. Use this helper to format a valid PI from parts.").style(color_muted()),
                    text("Tunisia example: country code 7, ECC E2. For other countries, enter your assigned values.").style(color_muted()),
                    row![
                        text("Country preset:"),
                        pick_list(self.country_items.clone(), Some(self.country_selected.clone()), Message::CountrySelected),
                        text("PI builder:"),
                        text_input("7", &self.pi_country_hex).on_input(Message::CountryCodeChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                        text_input("2", &self.pi_area_hex).on_input(Message::AreaCodeChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                        text_input("00", &self.pi_program_hex).on_input(Message::ProgramRefChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                        button("Apply PI")
                            .on_press(Message::ApplyPiFromParts)
                            .style(theme::Button::Custom(Box::new(PrimaryButton))),
                        button("Copy PI")
                            .on_press(Message::CopyPi)
                            .style(theme::Button::Custom(Box::new(GhostButton))),
                        button("Random PI")
                            .on_press(Message::GenerateRandomPi)
                            .style(theme::Button::Custom(Box::new(GhostButton))),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text(format!("PI preview: {}", pi_preview)).style(color_muted()),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    text("Random PI is for testing only. For production, use the code assigned by your regulator.").style(color_muted()),
                    row![
                        text("ECC (hex):"),
                        text_input("E2", &self.ecc_hex).on_input(Message::EccChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                        text("ECC identifies country in RDS. Leave default if unknown.").style(color_muted()),
                        text("DI:"),
                        checkbox("Stereo", self.di_stereo, Message::DiStereoChanged),
                        checkbox("Artificial head", self.di_artificial, Message::DiArtificialChanged),
                        checkbox("Compressed", self.di_compressed, Message::DiCompressedChanged),
                        checkbox("Dynamic PTY", self.di_dynamic, Message::DiDynamicChanged),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                ],
            )
        };

        let rds_schedule_card = || card(
            "Group Scheduling",
            column![
                row![
                    text("Mix 0A/2A/4A:"),
                    text_input("4", &self.group_0a).on_input(Message::Group0aChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                    text_input("1", &self.group_2a).on_input(Message::Group2aChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                    text_input("0", &self.group_4a).on_input(Message::Group4aChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                    text("CT interval (groups):"),
                    text_input("0", &self.ct_interval_groups).on_input(Message::CtIntervalGroupsChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                    button("Apply")
                        .on_press(Message::ApplyGroupMix)
                        .style(theme::Button::Custom(Box::new(PrimaryButton))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("Alternate PS:"),
                    text_input("ALT1|ALT2", &self.ps_alt_list_text).on_input(Message::PsAltListChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                    text("Interval (groups):"),
                    text_input("0", &self.ps_alt_interval).on_input(Message::PsAltIntervalChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                    button("Apply PS")
                        .on_press(Message::ApplyPsAlternates)
                        .style(theme::Button::Custom(Box::new(PrimaryButton))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
        );

        let af_card = || card(
            "AF Helper",
            column![
                row![
                    text("Ref freq (MHz):"),
                    text_input("98.0", &self.frequency_mhz).on_input(Message::FrequencyChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("AF list (MHz):"),
                    text_input("98.0", &self.af_list_text).on_input(Message::AfListChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("Generate from:"),
                    text_input("Base", &self.af_base).on_input(Message::AfBaseChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                    text_input("Spacing", &self.af_spacing).on_input(Message::AfSpacingChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                    text_input("Count", &self.af_count).on_input(Message::AfCountChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                    button("Generate")
                        .on_press(Message::AfGenerate)
                        .style(theme::Button::Custom(Box::new(GhostButton))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                if let Some(ref warning) = self.af_warning {
                    text(warning).style(color_accent_warm())
                } else {
                    text(" ").style(color_muted())
                },
            ],
        );

        let scrolling_card = || card(
            "Scrolling",
            column![
                row![
                    checkbox("PS scroll", self.ps_scroll_enabled, Message::PsScrollEnabled),
                    text_input("BOUZIDFM", &self.ps_scroll_text).on_input(Message::PsScrollTextChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                    text(format!("{:.1} cps", self.ps_scroll_cps)),
                    slider(0.5..=10.0, self.ps_scroll_cps, Message::PsScrollSpeedChanged).style(theme::Slider::Custom(Box::new(CustomSlider))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    checkbox("RT scroll", self.rt_scroll_enabled, Message::RtScrollEnabled),
                    text_input("BOUZIDFM Sidi Bouzid 98.0 MHz", &self.rt_scroll_text).on_input(Message::RtScrollTextChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                    text(format!("{:.1} cps", self.rt_scroll_cps)),
                    slider(0.5..=10.0, self.rt_scroll_cps, Message::RtScrollSpeedChanged).style(theme::Slider::Custom(Box::new(CustomSlider))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
        );

        let output_card = || card(
            "Output",
            column![
                row![
                    text(format!("Gain {:.2}x", self.output_gain)),
                    slider(0.5..=2.0, self.output_gain, Message::GainChanged).style(theme::Slider::Custom(Box::new(CustomSlider))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    checkbox("Limiter", self.limiter_enabled, Message::LimiterEnabled),
                    text(format!("Threshold {:.2}", self.limiter_threshold)),
                    slider(0.5..=1.0, self.limiter_threshold, Message::LimiterThresholdChanged).style(theme::Slider::Custom(Box::new(CustomSlider))),
                    text(format!("Lookahead {:.1} ms", self.limiter_lookahead_ms)),
                    slider(0.5..=10.0, self.limiter_lookahead_ms, Message::LimiterLookaheadChanged).style(theme::Slider::Custom(Box::new(CustomSlider))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
        );

        let levels_card = || card(
            "Stereo + RDS",
            column![
                row![
                    text(format!("Pilot {:.2}", self.pilot_level)),
                    slider(0.2..=1.5, self.pilot_level, Message::PilotLevelChanged).style(theme::Slider::Custom(Box::new(CustomSlider))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text(format!("RDS {:.2}", self.rds_level)),
                    slider(0.2..=1.5, self.rds_level, Message::RdsLevelChanged).style(theme::Slider::Custom(Box::new(CustomSlider))),
                    text(format!("Stereo sep {:.2}", self.stereo_separation)),
                    slider(0.5..=1.5, self.stereo_separation, Message::StereoSeparationChanged).style(theme::Slider::Custom(Box::new(CustomSlider))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            ],
        );

        let processing_card = || card(
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
                    slider(-30.0..=0.0, self.comp_threshold, Message::CompThresholdChanged).style(theme::Slider::Custom(Box::new(CustomSlider))),
                    text(format!("Ratio {:.1}", self.comp_ratio)),
                    slider(1.0..=6.0, self.comp_ratio, Message::CompRatioChanged).style(theme::Slider::Custom(Box::new(CustomSlider))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text(format!("Attack {:.3}s", self.comp_attack)),
                    slider(0.001..=0.1, self.comp_attack, Message::CompAttackChanged).style(theme::Slider::Custom(Box::new(CustomSlider))),
                    text(format!("Release {:.2}s", self.comp_release)),
                    slider(0.05..=1.0, self.comp_release, Message::CompReleaseChanged).style(theme::Slider::Custom(Box::new(CustomSlider))),
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
                        progress_bar(0.0..=1.0, self.meter_rms).style(theme::ProgressBar::Custom(Box::new(CustomProgressBar))),
                        text(format!("Peak {:.2}", self.meter_peak)),
                        progress_bar(0.0..=1.0, self.meter_peak).style(theme::ProgressBar::Custom(Box::new(PeakyProgressBar))),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text(format!("Pilot 19 kHz {:.2}", self.meter_pilot)),
                        progress_bar(0.0..=1.0, self.meter_pilot).style(theme::ProgressBar::Custom(Box::new(CustomProgressBar))),
                        text(format!("RDS 57 kHz {:.2}", self.meter_rds)),
                        progress_bar(0.0..=1.0, self.meter_rds).style(theme::ProgressBar::Custom(Box::new(WarmProgressBar))),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                ],
            )
        };

        let meters_full = || card_accent(
            "MPX Meter",
            column![
                row![
                    text(format!("RMS {:.2}", self.meter_rms)).style(color_accent()),
                    progress_bar(0.0..=1.0, self.meter_rms).style(theme::ProgressBar::Custom(Box::new(CustomProgressBar))),
                    text(format!("Peak {:.2}", self.meter_peak)).style(color_danger()),
                    progress_bar(0.0..=1.0, self.meter_peak).style(theme::ProgressBar::Custom(Box::new(PeakyProgressBar))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("Pilot 19 kHz").style(color_accent_warm()),
                    progress_bar(0.0..=1.0, self.meter_pilot).style(theme::ProgressBar::Custom(Box::new(WarmProgressBar))),
                    text("RDS 57 kHz").style(color_accent()),
                    progress_bar(0.0..=1.0, self.meter_rds).style(theme::ProgressBar::Custom(Box::new(CustomProgressBar))),
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

        let export_card = || card(
            "WAV Export",
            column![
                row![
                    text("Duration (sec):"),
                    text_input("10", &self.duration).on_input(Message::DurationChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("Audio WAV (optional):"),
                    text_input("", &self.audio_path).on_input(Message::AudioChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                row![
                    text("Output WAV:"),
                    text_input("mpx.wav", &self.output_path).on_input(Message::OutputChanged).style(theme::TextInput::Custom(Box::new(CustomTextInput))),
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

        let about_tab = column![
            card(
                "About Pulse FM",
                column![
                    text("Pulse FM RDS Encoder is a cross‑platform MPX generator with RDS tools and a live audio pipeline.").style(color_muted()),
                    text("Use it to generate MPX for analysis, streaming to 192 kHz devices, or offline WAV export.").style(color_muted()),
                    text("It does not transmit RF and is intended for baseband audio only.").style(color_muted()),
                ]
                .spacing(6),
            ),
            card(
                "Developer",
                column![
                    text("Hsouna Zinoubi").style(color_text()),
                    text("BOUZIDFM").style(color_muted()),
                    text("imhsouna@gmail.com").style(color_muted()),
                ]
                .spacing(4),
            ),
            card(
                "Quick Tips",
                column![
                    text("• Select an output device that supports 192 kHz float32.").style(color_muted()),
                    text("• Use the Audio tab to refresh device lists.").style(color_muted()),

                ]
                .spacing(4),
            ),
        ]
        .spacing(16)
        .width(Length::Fill);

        let compact = self.window_width < 980.0;

        let status_pill = if self.engine.is_some() {
            pill("● LIVE", color_live(), Color::from_rgb8(6, 24, 19))
        } else {
            pill("○ IDLE", color_surface_alt(), color_muted())
        };

        let status_text = text(&self.status).style(color_muted());

        let hero = container(
            row![
                column![
                    row![
                        text("Pulse FM").size(32).style(color_text()),
                        text("FM").size(32).style(rgba8f(56, 189, 248, 0.6)),
                    ]
                    .spacing(2)
                    .align_items(Alignment::Center),
                    text("RDS Encoder Studio").size(16).style(rgba8f(168, 85, 247, 0.7)),
                    text("Live MPX pipeline • 192 kHz • FM/RDS broadcast tools").size(12).style(color_muted()),
                ]
                .spacing(4)
                .width(Length::FillPortion(3)),
                column![
                    row![
                        status_pill,
                        status_text,
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        container(
                            row![
                                text(format!("{}", self.xrun_count)).size(13).style(color_accent()),
                                text("XRuns").size(11).style(color_muted()),
                            ]
                            .spacing(4)
                            .align_items(Alignment::Center),
                        )
                        .padding([4, 10])
                        .style(theme::Container::Custom(Box::new(MetricPill))),
                        text("|").size(11).style(rgba8f(255, 255, 255, 0.06)),
                        container(
                            row![
                                text(format!("{:.0}%", (self.buffer_fill * 100.0).clamp(0.0, 100.0))).size(13).style(color_accent_warm()),
                                text("Buf").size(11).style(color_muted()),
                            ]
                            .spacing(4)
                            .align_items(Alignment::Center),
                        )
                        .padding([4, 10])
                        .style(theme::Container::Custom(Box::new(MetricPill))),
                        text("|").size(11).style(rgba8f(255, 255, 255, 0.06)),
                        container(
                            row![
                                text(format!("{:.1}", self.latency_ms)).size(13).style(color_live()),
                                text("ms").size(11).style(color_muted()),
                            ]
                            .spacing(4)
                            .align_items(Alignment::Center),
                        )
                        .padding([4, 10])
                        .style(theme::Container::Custom(Box::new(MetricPill))),
                    ]
                    .spacing(0)
                    .align_items(Alignment::Center),
                ]
                .spacing(8)
                .width(Length::FillPortion(2)),
            ]
            .spacing(24)
            .align_items(Alignment::Center),
        )
        .padding(20)
        .width(Length::Fill)
        .style(theme::Container::from(hero_style));

        let body: Element<'_, Message> = match self.tab_selected {
            Tab::Dashboard => {
                if compact {
                    column![
                        stream_card(),
                        device_card(),
                        presets_card(),
                        station_card(),
                        meter_summary_card(),
                    ]
                    .spacing(16)
                    .into()
                } else {
                    column![
                        row![
                            column![stream_card(), device_card(), presets_card()].spacing(16).width(Length::FillPortion(2)),
                            column![station_card(), meter_summary_card()].spacing(16).width(Length::FillPortion(3)),
                        ]
                        .spacing(16)
                        .align_items(Alignment::Start),
                    ]
                    .into()
                }
            }
            Tab::Audio => {
                if compact {
                    column![
                        device_card(),
                        stream_card(),
                        health_card,
                        meter_summary_card(),
                    ]
                    .spacing(16)
                    .into()
                } else {
                    column![
                        row![
                            column![device_card(), stream_card(), health_card].spacing(16).width(Length::FillPortion(3)),
                            column![meter_summary_card()].spacing(16).width(Length::FillPortion(2)),
                        ]
                        .spacing(16)
                        .align_items(Alignment::Start),
                    ]
                    .into()
                }
            }
            Tab::Rds => {
                if compact {
                    column![
                        station_card(),
                        rds_identity_card(),
                        rds_schedule_card(),
                        af_card(),
                        scrolling_card(),
                    ]
                    .spacing(16)
                    .into()
                } else {
                    column![
                        row![
                            column![station_card(), rds_identity_card()].spacing(16).width(Length::FillPortion(3)),
                            column![rds_schedule_card(), af_card(), scrolling_card()].spacing(16).width(Length::FillPortion(2)),
                        ]
                        .spacing(16)
                        .align_items(Alignment::Start),
                    ]
                    .into()
                }
            }
            Tab::Processing => {
                if compact {
                    column![output_card(), levels_card(), processing_card()]
                        .spacing(16)
                        .into()
                } else {
                    column![
                        row![
                            column![output_card(), levels_card()].spacing(16).width(Length::FillPortion(3)),
                            column![processing_card()].spacing(16).width(Length::FillPortion(2)),
                        ]
                        .spacing(16)
                        .align_items(Alignment::Start),
                    ]
                    .into()
                }
            }
            Tab::Meters => meters_full().into(),
            Tab::Export => export_card().into(),
            Tab::About => about_tab.into(),
        };

        let content = column![
            hero,
            tabs,
            body,
        ]
        .spacing(18)
        .padding(24)
        .width(Length::Fill);

        let scroll = scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::Scrollable::Custom(Box::new(HiddenScrollbar)));

        container(scroll)
            .width(Length::Fill)
            .height(Length::Fill)
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
        border_radius: 12.0.into(),
        border_width: 1.0,
        border_color: color_border(),
    }
}

fn card_style(_theme: &Theme) -> container_widget::Appearance {
    container_widget::Appearance {
        background: Some(Background::Color(color_surface())),
        text_color: Some(color_text()),
        border_radius: 16.0.into(),
        border_width: 1.0,
        border_color: color_border(),
    }
}

fn card_accent_style(_theme: &Theme) -> container_widget::Appearance {
    container_widget::Appearance {
        background: Some(Background::Color(color_surface())),
        text_color: Some(color_text()),
        border_radius: 16.0.into(),
        border_width: 1.5,
        border_color: color_accent(),
    }
}

fn hero_style(_theme: &Theme) -> container_widget::Appearance {
    container_widget::Appearance {
        background: Some(Background::Color(color_surface())),
        text_color: Some(color_text()),
        border_radius: 20.0.into(),
        ..Default::default()
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
    container(text(label).size(12).style(fg))
        .padding([5, 12])
        .style(theme::Container::Custom(Box::new(PillStyle { bg, fg })))
        .into()
}

struct PrimaryButton;

impl button_widget::StyleSheet for PrimaryButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button_widget::Appearance {
        button_widget::Appearance {
            background: Some(Background::Color(color_gradient_end())),
            text_color: Color::from_rgb8(255, 255, 255),
            border_radius: 12.0.into(),
            border_width: 1.0,
            border_color: rgba8f(168, 85, 247, 0.5),
            shadow_offset: iced::Vector::new(0.0, 4.0),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(Color::from_rgb8(139, 92, 246)));
        active.shadow_offset = iced::Vector::new(0.0, 6.0);
        active.border_color = rgba8f(168, 85, 247, 0.8);
        active
    }

    fn pressed(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(Color::from_rgb8(126, 34, 206)));
        active.shadow_offset = iced::Vector::new(0.0, 2.0);
        active
    }
}

struct GhostButton;

impl button_widget::StyleSheet for GhostButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button_widget::Appearance {
        button_widget::Appearance {
            background: Some(Background::Color(color_surface_alt())),
            text_color: color_muted(),
            border_radius: 12.0.into(),
            border_width: 1.0,
            border_color: color_border(),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(color_surface()));
        active.border_color = rgba8f(56, 189, 248, 0.3);
        active.text_color = color_text();
        active
    }

    fn pressed(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(Color::from_rgb8(15, 23, 42)));
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
            border_radius: 12.0.into(),
            border_width: 1.0,
            border_color: rgba8f(248, 113, 113, 0.5),
            shadow_offset: iced::Vector::new(0.0, 4.0),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(Color::from_rgb8(252, 129, 129)));
        active.shadow_offset = iced::Vector::new(0.0, 6.0);
        active
    }

    fn pressed(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        active.background = Some(Background::Color(Color::from_rgb8(220, 38, 38)));
        active.shadow_offset = iced::Vector::new(0.0, 2.0);
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
            (rgba8f(56, 189, 248, 0.15), color_accent(), rgba8f(56, 189, 248, 0.3))
        } else {
            (rgba8f(255, 255, 255, 0.03), color_muted(), rgba8f(255, 255, 255, 0.06))
        };
        button_widget::Appearance {
            background: Some(Background::Color(bg)),
            text_color,
            border_radius: 12.0.into(),
            border_width: 1.0,
            border_color,
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button_widget::Appearance {
        if self.selected {
            return self.active(style);
        }
        button_widget::Appearance {
            background: Some(Background::Color(rgba8f(255, 255, 255, 0.06))),
            text_color: color_text(),
            border_radius: 12.0.into(),
            border_width: 1.0,
            border_color: rgba8f(255, 255, 255, 0.1),
            ..Default::default()
        }
    }

    fn pressed(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut active = self.active(style);
        active.background = if self.selected {
            Some(Background::Color(rgba8f(56, 189, 248, 0.2)))
        } else {
            Some(Background::Color(rgba8f(255, 255, 255, 0.04)))
        };
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

struct MetricPill;

impl container_widget::StyleSheet for MetricPill {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container_widget::Appearance {
        container_widget::Appearance {
            background: Some(Background::Color(rgba8f(255, 255, 255, 0.03))),
            text_color: Some(color_muted()),
            border_radius: 8.0.into(),
            border_width: 1.0,
            border_color: rgba8f(255, 255, 255, 0.05),
        }
    }
}

struct CustomTextInput;

impl text_input_widget::StyleSheet for CustomTextInput {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> text_input_widget::Appearance {
        text_input_widget::Appearance {
            background: Background::Color(rgba8f(255, 255, 255, 0.04)),
            border_radius: 10.0.into(),
            border_width: 1.0,
            border_color: color_border(),
            icon_color: color_muted(),
        }
    }

    fn focused(&self, _style: &Self::Style) -> text_input_widget::Appearance {
        text_input_widget::Appearance {
            background: Background::Color(rgba8f(56, 189, 248, 0.05)),
            border_radius: 10.0.into(),
            border_width: 1.5,
            border_color: rgba8f(56, 189, 248, 0.4),
            icon_color: color_accent(),
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        rgba8f(120, 145, 180, 0.4)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        color_text()
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        color_muted()
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        rgba8f(56, 189, 248, 0.3)
    }

    fn disabled(&self, _style: &Self::Style) -> text_input_widget::Appearance {
        text_input_widget::Appearance {
            background: Background::Color(rgba8f(255, 255, 255, 0.02)),
            border_radius: 10.0.into(),
            border_width: 1.0,
            border_color: rgba8f(255, 255, 255, 0.04),
            icon_color: color_muted(),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> text_input_widget::Appearance {
        text_input_widget::Appearance {
            background: Background::Color(rgba8f(255, 255, 255, 0.06)),
            border_radius: 10.0.into(),
            border_width: 1.0,
            border_color: rgba8f(56, 189, 248, 0.3),
            icon_color: color_accent(),
        }
    }
}

struct CustomSlider;

impl slider_widget::StyleSheet for CustomSlider {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> slider_widget::Appearance {
        slider_widget::Appearance {
            rail: slider_widget::Rail {
                colors: (rgba8f(56, 189, 248, 0.2), color_bg()),
                width: 4.0,
                border_radius: 2.0.into(),
            },
            handle: slider_widget::Handle {
                shape: slider_widget::HandleShape::Circle { radius: 7.0 },
                color: color_accent(),
                border_width: 1.0,
                border_color: rgba8f(56, 189, 248, 0.4),
            },
        }
    }

    fn hovered(&self, _style: &Self::Style) -> slider_widget::Appearance {
        slider_widget::Appearance {
            rail: slider_widget::Rail {
                colors: (rgba8f(56, 189, 248, 0.3), color_bg()),
                width: 4.0,
                border_radius: 2.0.into(),
            },
            handle: slider_widget::Handle {
                shape: slider_widget::HandleShape::Circle { radius: 8.0 },
                color: Color::from_rgb8(125, 211, 252),
                border_width: 1.5,
                border_color: rgba8f(56, 189, 248, 0.6),
            },
        }
    }

    fn dragging(&self, _style: &Self::Style) -> slider_widget::Appearance {
        slider_widget::Appearance {
            rail: slider_widget::Rail {
                colors: (rgba8f(56, 189, 248, 0.4), color_bg()),
                width: 4.0,
                border_radius: 2.0.into(),
            },
            handle: slider_widget::Handle {
                shape: slider_widget::HandleShape::Circle { radius: 8.0 },
                color: Color::from_rgb8(125, 211, 252),
                border_width: 1.5,
                border_color: rgba8f(56, 189, 248, 0.8),
            },
        }
    }
}

struct CustomProgressBar;

impl progress_bar_widget::StyleSheet for CustomProgressBar {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> progress_bar_widget::Appearance {
        progress_bar_widget::Appearance {
            background: Background::Color(rgba8f(255, 255, 255, 0.06)),
            bar: Background::Color(Color::from_rgb8(139, 92, 246)),
            border_radius: 4.0.into(),
        }
    }
}

struct WarmProgressBar;

impl progress_bar_widget::StyleSheet for WarmProgressBar {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> progress_bar_widget::Appearance {
        progress_bar_widget::Appearance {
            background: Background::Color(rgba8f(255, 255, 255, 0.06)),
            bar: Background::Color(Color::from_rgb8(251, 146, 60)),
            border_radius: 4.0.into(),
        }
    }
}

struct PeakyProgressBar;

impl progress_bar_widget::StyleSheet for PeakyProgressBar {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> progress_bar_widget::Appearance {
        progress_bar_widget::Appearance {
            background: Background::Color(rgba8f(255, 255, 255, 0.06)),
            bar: Background::Color(Color::from_rgb8(248, 113, 113)),
            border_radius: 4.0.into(),
        }
    }
}

struct HiddenScrollbar;

impl scrollable_widget::StyleSheet for HiddenScrollbar {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> scrollable_widget::Scrollbar {
        scrollable_widget::Scrollbar {
            background: None,
            border_radius: 0.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: scrollable_widget::Scroller {
                color: Color::TRANSPARENT,
                border_radius: 0.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self, _style: &Self::Style, _is_mouse_over_scrollbar: bool) -> scrollable_widget::Scrollbar {
        self.active(_style)
    }

    fn dragging(&self, _style: &Self::Style) -> scrollable_widget::Scrollbar {
        self.active(_style)
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
        frame.fill(&bg, Color::from_rgb8(6, 8, 20));

        let width = frame.size().width;
        let height = frame.size().height;

        let grid_color = rgba8f(99, 102, 241, 0.08);
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
                color: Color::from_rgb8(110, 120, 160),
                size: 11.0,
                ..Text::default()
            });
        }

        let draw_fill = |frame: &mut Frame, data: &[f32], color: Color| {
            if data.len() < 2 {
                return;
            }
            let step = frame.size().width / (data.len() as f32 - 1.0);
            let path = Path::new(|builder| {
                builder.move_to(iced::Point::new(0.0, height));
                for (i, db) in data.iter().enumerate() {
                    let unit = (db.clamp(-60.0, 0.0) + 60.0) / 60.0;
                    let x = i as f32 * step;
                    let y = height - unit * height;
                    builder.line_to(iced::Point::new(x, y));
                }
                builder.line_to(iced::Point::new(width, height));
                builder.close();
            });
            frame.fill(&path, color);
        };

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

        draw_fill(&mut frame, &self.spectrum_avg_db, rgba8f(56, 189, 248, 0.08));
        draw_fill(&mut frame, &self.spectrum_peak_db, rgba8f(251, 146, 60, 0.05));

        draw_line(&mut frame, &self.spectrum_avg_db, rgba8f(56, 189, 248, 0.6), 2.5);
        draw_line(&mut frame, &self.spectrum_peak_db, rgba8f(251, 146, 60, 0.8), 1.5);

        draw_line(
            &mut frame,
            &self.spectrum_avg_db,
            rgba8f(56, 189, 248, 0.8),
            1.0,
        );

        let rds_x = width * (57000.0 / 96000.0);
        let rds_line = Path::line(
            iced::Point::new(rds_x, 0.0),
            iced::Point::new(rds_x, height),
        );
        frame.stroke(&rds_line, Stroke::default().with_width(2.0).with_color(rgba8f(251, 146, 60, 0.5)));
        let glow_line = Path::line(
            iced::Point::new(rds_x, 0.0),
            iced::Point::new(rds_x, height),
        );
        frame.stroke(&glow_line, Stroke::default().with_width(6.0).with_color(rgba8f(251, 146, 60, 0.1)));
        frame.fill_text(Text {
            content: "RDS 57k".to_string(),
            position: iced::Point::new(rds_x + 8.0, 8.0),
            color: Color::from_rgb8(251, 170, 60),
            size: 11.0,
            ..Text::default()
        });

        let markers = [0.0, 19000.0, 38000.0, 57000.0, 76000.0, 95000.0];
        for freq in markers {
            let x = width * (freq / 96000.0);
            let line = Path::line(iced::Point::new(x, 0.0), iced::Point::new(x, height));
            frame.stroke(&line, Stroke::default().with_width(1.0).with_color(rgba8f(99, 102, 241, 0.1)));
            frame.fill_text(Text {
                content: format!("{:.0}k", freq / 1000.0),
                position: iced::Point::new(x + 4.0, height - 14.0),
                color: Color::from_rgb8(110, 120, 160),
                size: 10.0,
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
        frame.fill(&bg, Color::from_rgb8(5, 8, 18));

        let width = frame.size().width;
        let height = frame.size().height;

        let mid_y = height / 2.0;
        let mid_line = Path::line(iced::Point::new(0.0, mid_y), iced::Point::new(width, mid_y));
        frame.stroke(&mid_line, Stroke::default().with_width(1.0).with_color(rgba8f(99, 102, 241, 0.1)));

        let grid_h_lines = 4;
        for i in 1..grid_h_lines {
            let y = height * (i as f32 / grid_h_lines as f32);
            let line = Path::line(iced::Point::new(0.0, y), iced::Point::new(width, y));
            frame.stroke(&line, Stroke::default().with_width(1.0).with_color(rgba8f(99, 102, 241, 0.04)));
        }

        let draw_trace = |frame: &mut Frame, data: &[f32], width: f32, mid_y: f32, color: Color, thickness: f32| {
            if data.len() < 2 {
                return;
            }
            let step = width / (data.len() as f32 - 1.0);
            let path = Path::new(|builder| {
                for (i, s) in data.iter().enumerate() {
                    let x = i as f32 * step;
                    let y = mid_y - (s.clamp(-1.0, 1.0) * mid_y * 0.9);
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
            rgba8f(52, 211, 153, 0.08),
            8.0,
        );
        draw_trace(
            &mut frame,
            &self.samples,
            width,
            mid_y,
            rgba8f(52, 211, 153, 0.15),
            5.0,
        );
        draw_trace(
            &mut frame,
            &self.samples,
            width,
            mid_y,
            rgba8f(52, 211, 153, 0.9),
            1.8,
        );

        vec![frame.into_geometry()]
    }
}
