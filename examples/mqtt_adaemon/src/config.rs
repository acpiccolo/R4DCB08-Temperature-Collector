use crate::options::CliOptions;
use directories_next::ProjectDirs;
use serde::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct MqttConfig {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub entity_id: Option<String>,
    /// Quality of service code to use
    #[serde(default = "default_qos")]
    qos: u8,
    #[serde(default = "default_repeats")]
    pub repeats: u32,
}

fn default_qos() -> u8 {
    0
}

fn default_repeats() -> u32 {
    1
}

impl MqttConfig {
    pub fn qos(&self) -> i32 {
        assert!((0..2).contains(&self.qos));
        self.qos as i32
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModbusConfig {
    #[serde(default = "default_device")]
    pub device: String,
    #[serde(default = "default_baud_rate")]
    pub baud_rate: u32,
    #[serde(default = "default_timeout", with = "humantime_serde")]
    pub timeout: Duration,
    #[serde(default = "default_address")]
    pub address: u8,
    #[serde(default = "default_delay", with = "humantime_serde")]
    pub delay: Duration,
    #[serde(default = "default_poll_rate", with = "humantime_serde")]
    pub poll_rate: Duration,
}

fn default_device() -> String {
    String::from("/dev/ttyUSB0")
}

fn default_baud_rate() -> u32 {
    9600
}

fn default_timeout() -> Duration {
    Duration::from_secs(5)
}

fn default_address() -> u8 {
    0x01
}

fn default_delay() -> Duration {
    Duration::from_millis(10)
}

fn default_poll_rate() -> Duration {
    Duration::from_secs(3)
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub modbus: ModbusConfig,
}

pub(crate) fn get_config(options: &CliOptions) -> anyhow::Result<Config> {
    let path = get_config_file_path(options);
    log::debug!("Loading config file from {:?}", &path);
    let config_file = File::open(path)?;
    let config: Config = serde_yaml::from_reader(&config_file)?;

    Ok(config)
}

fn get_config_file_path(options: &CliOptions) -> PathBuf {
    let default_file = Path::new("config.yml");
    let user_dir_file = get_user_dir_path();

    match (&options.config, default_file, user_dir_file) {
        (Some(config), _, _) => config.clone(),
        (None, config, _) if config.exists() => config.to_path_buf(),
        (None, _, Some(config)) if config.exists() => config,
        _ => panic!("No config file found"),
    }
}

fn get_user_dir_path() -> Option<PathBuf> {
    if let Some(project_dirs) = ProjectDirs::from("me", "ac", "r4dcb08_mqtt") {
        let config_dir = project_dirs.config_dir();
        Some(config_dir.join("config.yml"))
    } else {
        None
    }
}
