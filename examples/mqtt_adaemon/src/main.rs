mod config;
mod options;

use crate::config::{get_config, Config};
use crate::options::CliOptions;
use anyhow::{Context, Result};
use flexi_logger::{Logger, LoggerHandle};
use futures::try_join;
use futures::{executor::block_on, stream::StreamExt};
use log::*;
use paho_mqtt as mqtt;
use r4dcb08_lib::async_client::R4DCB08;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{ops::Deref, panic, time::Duration};
use structopt::StructOpt;

fn logging_init(verbose: u8) -> LoggerHandle {
    let log_level = match verbose {
        0 => LevelFilter::Info.as_str(),
        1 => LevelFilter::Debug.as_str(),
        _ => LevelFilter::Trace.as_str(),
    };
    let log_handle = Logger::try_with_env_or_str(log_level)
        .expect("Cannot init logging")
        .start()
        .expect("Cannot start logging");

    panic::set_hook(Box::new(|panic_info| {
        let (filename, line, column) = panic_info
            .location()
            .map(|loc| (loc.file(), loc.line(), loc.column()))
            .unwrap_or(("<unknown>", 0, 0));
        let cause = panic_info
            .payload()
            .downcast_ref::<String>()
            .map(String::deref);
        let cause = cause.unwrap_or_else(|| {
            panic_info
                .payload()
                .downcast_ref::<&str>()
                .copied()
                .unwrap_or("<cause unknown>")
        });

        error!(
            "Thread '{}' panicked at {}:{}:{}: {}",
            std::thread::current().name().unwrap_or("<unknown>"),
            filename,
            line,
            column,
            cause
        );
    }));
    log_handle
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = CliOptions::from_args();
    let _log_handle = logging_init(options.verbose);
    let config = get_config(&options)?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        trace!("Received Ctrl-C")
    })
    .expect("Error setting Ctrl-C handler");

    log::info!("Starting...");
    log::trace!("Config: {:?}", config);

    let mut cli = mqtt::AsyncClient::new(config.mqtt.url.clone())
        .with_context(|| "Error creating mqtt client")?;

    // Use 5sec timeouts for sync calls.
    //cli.set_timeout(Duration::from_secs(5));

    let mut conn_builder = mqtt::ConnectOptionsBuilder::new();
    let mut conn_builder = conn_builder
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true);

    if let Some(user_name) = &config.mqtt.username {
        conn_builder = conn_builder.user_name(user_name)
    }
    if let Some(password) = &config.mqtt.password {
        conn_builder = conn_builder.password(password)
    }
    let conn_ops = conn_builder.finalize();

    // Get message stream before connecting.
    let mut strm = cli.get_stream(25);

    // Connect and wait for it to complete or fail.
    // The default connection uses MQTT v3.x
    cli.connect(conn_ops)
        .wait()
        .with_context(|| "Mqtt client unable to connect")?;

    let cli_clone = cli.clone();
    let subscriber = async {
        cli_clone
            .subscribe("r4dcb08/test", 0)
            .await
            .with_context(|| "")?;

        // Just loop on incoming messages.
        println!("Waiting for messages...");

        // Note that we're not providing a way to cleanly shut down and
        // disconnect. Therefore, when you kill this app (with a ^C or
        // whatever) the server will get an unexpected drop and then
        // should emit the LWT message.

        while let Some(msg_opt) = strm.next().await {
            if let Some(msg) = msg_opt {
                println!("AAAAAAAAAAAAAAA {}", msg);
            } else {
                // A "None" means we were disconnected. Try to reconnect...
                println!("Lost connection. Attempting reconnect.");
                while let Err(err) = cli_clone.reconnect().await {
                    println!("Error reconnecting: {}", err);
                    // For tokio use: tokio::time::delay_for()
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    //async_std::task::sleep(Duration::from_millis(1000)).await;
                }
            }
        }

        // Explicit return type for the async block
        Ok::<(), anyhow::Error>(())
    };

    let builder = tokio_serial::new(config.modbus.device.clone(), config.modbus.baud_rate)
        .parity(tokio_serial::Parity::None)
        .stop_bits(tokio_serial::StopBits::One)
        .flow_control(tokio_serial::FlowControl::None)
        .data_bits(tokio_serial::DataBits::Eight);

    let mut port = tokio_serial::SerialStream::open(&builder).unwrap();
    //port.set_exclusive(true)?;

    let mut device = R4DCB08::new(tokio_modbus::client::rtu::attach_slave(
        port,
        tokio_modbus::Slave(config.modbus.address),
    ));
    //device.set_timeout(config.modbus.timeout);
    device.set_exec_delay(config.modbus.delay);

    go_online(&mut cli, &config)?;
    let run_tok = run(&running, &mut cli, &config, &mut device);

    try_join!(subscriber, run_tok)?;

    log::info!("Stopping...");
    go_offline(&cli, &config)?;

    cli.disconnect(None)
        .wait()
        .with_context(|| "Error disconnect mqtt client")?;

    Ok(())
}

fn get_topic(entity_id: Option<&str>, appendix: &str) -> String {
    if let Some(entity_id) = entity_id {
        format!("r4dcb08/{}/{}", entity_id, appendix)
    } else {
        format!("r4dcb08/{}", appendix)
    }
}

const MQTT_APPENDIX_AVAILABILITY: &str = "availability";
const MQTT_APPENDIX_TEMPERATURE: &str = "temperature";

fn go_online(client: &mut mqtt::AsyncClient, config: &Config) -> Result<()> {
    let msg = mqtt::Message::new(
        get_topic(config.mqtt.entity_id.as_deref(), MQTT_APPENDIX_AVAILABILITY),
        "online",
        config.mqtt.qos(),
    );
    client
        .publish(msg)
        .wait()
        .with_context(|| "Cannot publish mqtt message")
}

async fn run(
    running: &Arc<AtomicBool>,
    client: &mut mqtt::AsyncClient,
    config: &Config,
    device: &mut R4DCB08,
) -> Result<()> {
    while running.load(Ordering::SeqCst) {
        let reply =
            tokio::time::timeout(config.modbus.timeout, device.read_temperature()).await??;
        trace!("Temperature: {:?}", reply);
        let mut tokens = Vec::new();
        for (channel, temperature) in reply.iter().enumerate() {
            let topic = format!(
                "{}/{}",
                get_topic(config.mqtt.entity_id.as_deref(), MQTT_APPENDIX_TEMPERATURE),
                channel
            );
            let msg = mqtt::Message::new(topic, temperature.to_string(), config.mqtt.qos());
            tokens.push(client.publish(msg));
        }

        for token in tokens {
            token
                .wait()
                .with_context(|| "Cannot publish mqtt message")?;
        }

        if !running.load(Ordering::SeqCst) {
            break;
        }
        tokio::time::sleep(config.modbus.poll_rate).await;
    }
    Ok(())
}

fn go_offline(client: &mqtt::AsyncClient, config: &Config) -> Result<()> {
    let msg = mqtt::Message::new_retained(
        get_topic(config.mqtt.entity_id.as_deref(), MQTT_APPENDIX_AVAILABILITY),
        "offline",
        config.mqtt.qos(),
    );
    client
        .publish(msg)
        .wait()
        .with_context(|| "Cannot publish mqtt message")
}
