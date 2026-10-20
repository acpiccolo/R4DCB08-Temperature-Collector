use anyhow::{Context, Result};
use log::{debug, error, info, trace, warn};
use r4dcb08_lib::tokio_sync_safe_client::SafeClient;
use std::time::Duration;

use crate::{commandline, mqtt};

pub fn run(
    client: &mut SafeClient,
    output: commandline::DaemonOutput,
    interval: Duration,
) -> Result<()> {
    info!("Starting daemon mode: output={output:?}, interval={interval:?}");

    let mut mqtt_publisher: Option<mqtt::MqttPublisher> = None;

    if let commandline::DaemonOutput::Mqtt { config_file } = &output {
        let config = mqtt::MqttConfig::load(config_file)
            .with_context(|| format!("Failed to open MQTT config file at '{config_file}'"))?;
        info!("Successfully loaded MQTT config: {config:?}");
        let publisher =
            mqtt::MqttPublisher::new(config).with_context(|| "Failed to create MQTT publisher")?;
        info!("MQTT Publisher created successfully.");
        mqtt_publisher = Some(publisher);
    }

    loop {
        debug!("Daemon: Reading temperatures...");
        match client.read_temperatures() {
            Ok(temperatures) => {
                trace!("Temperatures read: {temperatures:?}");
                match &output {
                    commandline::DaemonOutput::Console => {
                        println!("Temperatures (°C): {}", temperatures);
                    }
                    commandline::DaemonOutput::Mqtt { .. } => {
                        if let Some(publisher) = &mqtt_publisher {
                            for (ch_idx, temp_val) in temperatures.iter().enumerate() {
                                let channel_topic = format!("{}/CH{ch_idx}", publisher.topic());
                                let payload_str = temp_val.to_string();

                                debug!(
                                    "Publishing to MQTT: Topic='{channel_topic}', Payload='{payload_str}', QoS={}",
                                    publisher.qos()
                                );

                                if let Err(e) = publisher.publish(&channel_topic, &payload_str) {
                                    error!("Failed to publish MQTT message to {channel_topic}: {e}.");
                                }
                            }
                        } else {
                            warn!("MQTT output selected, but publisher is not initialized. Skipping publish.");
                        }
                    }
                }
            }
            Err(e) => {
                error!("Daemon: Error reading temperatures: {e}");
            }
        }
        std::thread::sleep(interval);
    }
}
