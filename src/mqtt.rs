use anyhow::{Context, Result};
use paho_mqtt::{Client, ConnectOptionsBuilder, CreateOptionsBuilder};
use serde::Deserialize;
use std::fmt;
use std::time::Duration;

#[derive(Deserialize, Clone)]
pub struct MqttConfig {
    uri: String,
    username: Option<String>,
    password: Option<String>,
    #[serde(default = "MqttConfig::default_topic")]
    topic: String,
    #[serde(default = "MqttConfig::default_qos")]
    qos: i32,
    #[serde(default = "MqttConfig::default_client_id")]
    client_id: String,
    #[serde(
        default = "MqttConfig::default_operation_timeout",
        with = "humantime_serde"
    )]
    operation_timeout: Duration,
    #[serde(
        default = "MqttConfig::default_keep_alive_interval",
        with = "humantime_serde"
    )]
    keep_alive_interval: Duration,
    #[serde(
        default = "MqttConfig::default_auto_reconnect_interval_min",
        with = "humantime_serde"
    )]
    auto_reconnect_interval_min: Duration,
    #[serde(
        default = "MqttConfig::default_auto_reconnect_interval_max",
        with = "humantime_serde"
    )]
    auto_reconnect_interval_max: Duration,
}

impl fmt::Debug for MqttConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MqttConfig")
            .field("uri", &self.uri)
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "********"))
            .field("topic", &self.topic)
            .field("qos", &self.qos)
            .field("client_id", &self.client_id)
            .field("operation_timeout", &self.operation_timeout)
            .field("keep_alive_interval", &self.keep_alive_interval)
            .field(
                "auto_reconnect_interval_min",
                &self.auto_reconnect_interval_min,
            )
            .field(
                "auto_reconnect_interval_max",
                &self.auto_reconnect_interval_max,
            )
            .finish()
    }
}

impl MqttConfig {
    fn default_topic() -> String {
        "r4dcb08".into()
    }

    fn default_qos() -> i32 {
        0
    }

    fn generate_random_string(len: usize) -> String {
        use rand::distr::{Alphanumeric, SampleString};

        Alphanumeric.sample_string(&mut rand::rng(), len)
    }

    fn default_client_id() -> String {
        format!("r4dcb08-{}", Self::generate_random_string(8))
    }

    fn default_operation_timeout() -> Duration {
        Duration::from_secs(10)
    }

    fn default_keep_alive_interval() -> Duration {
        Duration::from_secs(30)
    }

    fn default_auto_reconnect_interval_min() -> Duration {
        Duration::from_secs(1)
    }

    fn default_auto_reconnect_interval_max() -> Duration {
        Duration::from_secs(30)
    }

    pub const DEFAULT_CONFIG_FILE: &str = "mqtt.yaml";

    pub fn load(config_file_path: &str) -> Result<Self> {
        log::debug!("Loading config file from {config_file_path:?}");
        let config_file = std::fs::File::open(config_file_path)
            .with_context(|| format!("Cannot open MQTT config file {config_file_path:?}"))?;
        let mut config: Self = yaml_serde::from_reader(&config_file)
            .with_context(|| format!("Cannot read MQTT config from file: {config_file_path:?}"))?;

        // Override settings with environment variables if set
        if let Ok(env_uri) = std::env::var("MQTT_URI") {
            config.uri = env_uri;
        }
        if let Ok(env_user) = std::env::var("MQTT_USERNAME") {
            config.username = Some(env_user);
        }
        if let Ok(env_pass) = std::env::var("MQTT_PASSWORD") {
            config.password = Some(env_pass);
        }
        if let Ok(env_topic) = std::env::var("MQTT_TOPIC") {
            config.topic = env_topic;
        }
        if let Ok(env_client_id) = std::env::var("MQTT_CLIENT_ID") {
            config.client_id = env_client_id;
        }

        Ok(config)
    }

    pub fn create_client(&self) -> Result<Client> {
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(&self.uri)
            .client_id(&self.client_id)
            .persistence(None) // In-memory persistence
            .finalize();

        let mut client = Client::new(create_opts)
            .with_context(|| format!("Error creating MQTT client for server: {}", self.uri))?;

        client.set_timeout(self.operation_timeout);

        let mut conn_builder = ConnectOptionsBuilder::new();
        conn_builder
            .keep_alive_interval(self.keep_alive_interval)
            .clean_session(true) // Typically true for telemetry publishers
            .automatic_reconnect(
                self.auto_reconnect_interval_min,
                self.auto_reconnect_interval_max,
            ); // Enable auto-reconnect

        if let Some(user_name_str) = &self.username {
            conn_builder.user_name(user_name_str.as_str());
        }
        if let Some(password_str) = &self.password {
            conn_builder.password(password_str.as_str());
        }
        let conn_opts = conn_builder.finalize();

        log::info!(
            "Attempting to connect to MQTT broker: {} with client_id: {}",
            self.uri,
            self.client_id
        );

        client
            .connect(conn_opts)
            .with_context(|| "Failed to connect to MQTT broker")?;
        log::info!("Connected to MQTT broker.");
        Ok(client)
    }
}

pub struct MqttPublisher {
    client: Client,
    topic: String,
    qos: i32,
    channel_topics: Vec<String>,
}

impl MqttPublisher {
    pub fn new(config: MqttConfig) -> Result<Self> {
        let topic = config.topic.clone();
        let qos = config.qos;
        let channel_topics = (0..8).map(|ch_idx| format!("{topic}/CH{ch_idx}")).collect();
        let client = config.create_client()?;
        Ok(Self {
            client,
            topic,
            qos,
            channel_topics,
        })
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn qos(&self) -> i32 {
        self.qos
    }

    pub fn channel_topic(&self, channel_idx: usize) -> Option<&str> {
        self.channel_topics.get(channel_idx).map(|s| s.as_str())
    }

    pub fn publish(&self, topic: &str, payload: &str) -> Result<()> {
        let msg = paho_mqtt::Message::new_retained(topic, payload, self.qos);
        self.client
            .publish(msg)
            .with_context(|| format!("Failed to publish MQTT message to topic {topic}"))
            .map(|_| ())
    }

    pub fn disconnect(&self) {
        if self.client.is_connected() {
            log::info!("Disconnecting from MQTT broker...");
            if let Err(e) = self.client.disconnect(None) {
                log::warn!("Failed to disconnect cleanly from MQTT broker: {e}");
            } else {
                log::info!("Disconnected cleanly from MQTT broker.");
            }
        }
    }
}
