//! R4DCB08 Temperature Collector CLI
//!
//! A command-line interface (CLI) application for interacting with R4DCB08
//! 8-channel temperature collector modules using Modbus RTU (serial) or Modbus TCP.
//!
//! This tool allows users to:
//! - Read current temperatures from all sensor channels.
//! - Read and set temperature correction offsets for sensor calibration.
//! - Read and set the Modbus communication baud rate.
//! - Read and set the Modbus device address.
//! - Configure automatic temperature reporting.
//! - Perform a factory reset of the device.
//! - Run in a continuous daemon mode to poll temperatures and either print them
//!   to the console or publish them to an MQTT broker.
//! - Scan an RTU bus for connected R4DCB08 devices.
//!
//! The CLI leverages the `r4dcb08_lib` crate for protocol definitions and client operations.

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use dialoguer::Confirm;
use flexi_logger::{Logger, LoggerHandle};
use log::*;
use paho_mqtt as mqtt;
use serde::Deserialize;
use r4dcb08_lib::{protocol as proto, tokio_sync_client::R4DCB08};
use std::fs::File;
use std::io::{Read, Write, stdout};
use std::{panic, time::Duration};
use serde_yaml;

fn default_device_name() -> String {
    if cfg!(target_os = "windows") {
        String::from("COM1") // Common default for Windows, though may vary.
    } else {
        String::from("/dev/ttyUSB0") // Common default for USB-to-serial adapters on Linux.
    }
}

fn parse_channel(s: &str) -> Result<proto::Channel, String> {
    let channel_num =
        clap_num::maybe_hex::<u8>(s).map_err(|e| format!("Invalid channel number format: {e}"))?;
    proto::Channel::try_from(channel_num).map_err(|e| e.to_string())
}

fn parse_address(s: &str) -> Result<proto::Address, String> {
    let address_val =
        clap_num::maybe_hex::<u8>(s).map_err(|e| format!("Invalid address format: {e}"))?;
    proto::Address::try_from(address_val).map_err(|e| e.to_string())
}

fn parse_automatic_report(s: &str) -> Result<proto::AutomaticReport, String> {
    let report_time_val = clap_num::maybe_hex::<u8>(s)
        .map_err(|e| format!("Invalid automatic report time format: {e}"))?;
    // proto::AutomaticReport::from() does not validate range (0-255), but u8 ensures this.
    Ok(proto::AutomaticReport::from(report_time_val))
}

fn parse_baud_rate(s: &str) -> Result<proto::BaudRate, String> {
    let rate_val = s
        .parse::<u16>()
        .map_err(|e| format!("Invalid baud rate number format: {e}"))?;
    proto::BaudRate::try_from(rate_val).map_err(|e| e.to_string())
}

fn parse_degree_celsius(s: &str) -> Result<proto::Temperature, String> {
    let temp_val = s
        .parse::<f32>()
        .map_err(|e| format!("Invalid temperature value format: {e}"))?;
    proto::Temperature::try_from(temp_val).map_err(|e| e.to_string())
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
enum CliConnection {
    /// Connect to a temperature collector via Modbus TCP.
    Tcp {
        /// The IP address or hostname and port of the Modbus TCP device.
        /// Example: "192.168.1.100:502" or "modbus-gateway.local:502".
        address: String,

        /// TCP-specific commands for the connected device.
        #[command(subcommand)]
        command: CliCommands,
    },
    /// Connect to a temperature collector via Modbus RTU (Serial).
    Rtu {
        /// Serial port device name.
        /// Examples: "/dev/ttyUSB0" (Linux), "COM3" (Windows).
        #[arg(short, long, default_value_t = default_device_name())]
        device: String,

        /// Baud rate for serial communication.
        /// Must match the device's configured baud rate.
        /// Supported values: 1200, 2400, 4800, 9600, 19200.
        #[arg(long, default_value_t = proto::BaudRate::default(), value_parser = parse_baud_rate)]
        baud_rate: proto::BaudRate,

        /// The Modbus RTU device address.
        /// Must be unique on the RS485 bus, ranging from 1 to 247.
        #[arg(short, long, default_value_t = proto::Address::default(), value_parser = parse_address)]
        address: proto::Address,

        /// RTU-specific commands for the connected device.
        #[command(subcommand)]
        command: CliCommands,
    },
    /// Scan a Modbus RTU (Serial) bus for R4DCB08 devices.
    /// This command attempts to find devices by trying all supported baud rates
    /// and querying for a response using the broadcast address.
    /// **Warning:** Ensure only R4DCB08 devices or devices that do not conflict
    /// with this scan are on the bus segment being scanned.
    #[clap(verbatim_doc_comment)]
    RtuScan {
        /// Serial port device name to scan.
        /// Examples: "/dev/ttyUSB0" (Linux), "COM3" (Windows).
        #[arg(short, long, default_value_t = default_device_name())]
        device: String,
    },
}

#[derive(Debug, Deserialize)]
struct MqttConfigFile {
    url: String,
    username: Option<String>,
    password: Option<String>,
    topic: Option<String>,
    qos: Option<i32>,
    client_id: Option<String>,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
enum DaemonMode {
    /// Continuously read temperatures and print them to the standard output (console).
    Stdout,
    /// Continuously read temperatures and publish them to an MQTT broker.
    Mqtt {
        /// Path to an optional YAML file for MQTT configuration.
        /// If not provided, "mqtt.yaml" is loaded from the current directory.
        #[arg(long, value_name = "PATH")]
        mqtt_config_path: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
enum CliCommands {
    /// Run in daemon mode: continuously poll temperatures at a specified interval.
    /// Output can be directed to stdout or an MQTT broker.
    #[clap(verbatim_doc_comment)]
    Daemon {
        /// The interval at which to poll for temperature readings.
        /// Examples: "2s", "500ms", "1min".
        #[arg(value_parser = humantime::parse_duration, short, long, default_value = "2s", verbatim_doc_comment)]
        poll_interval: Duration,

        /// Specifies the output mode for daemon operation.
        #[command(subcommand)]
        mode: DaemonMode,
    },

    /// Read and display the current temperatures from all sensor channels.
    Read,

    /// Read and display the current temperature correction values for all sensor channels.
    ReadCorrection,

    /// Read and display the currently configured Modbus communication baud rate.
    ReadBaudRate,

    /// Read and display the current automatic temperature reporting interval.
    ReadAutomaticReport,

    /// Read and display all primary device values: temperatures, corrections, baud rate, and auto-report interval.
    ReadAll,

    /// Query the current Modbus RTU device address.
    /// This command sends a request to the broadcast address.
    /// **Warning:** Ensure only ONE R4DCB08 device is connected to the RS485 bus
    /// when using this command to avoid response collisions.
    #[clap(verbatim_doc_comment)]
    QueryAddress,

    /// Set a temperature correction offset for a specific sensor channel.
    /// This is useful for calibrating individual sensors to compensate for inaccuracies.
    /// The correction value is added to the raw sensor reading by the module.
    #[clap(verbatim_doc_comment)]
    SetCorrection {
        /// Target temperature sensor channel number (0 to 7).
        /// Can be specified in decimal or hexadecimal (e.g., "0x0" to "0x7").
        #[arg(value_parser = parse_channel, verbatim_doc_comment)]
        channel: proto::Channel,
        /// Correction value in degrees Celsius (°C).
        /// Can be positive or negative. The module typically supports 0.1°C increments.
        /// Example: "1.5" for +1.5°C, "-0.3" for -0.3°C.
        #[arg(value_parser = parse_degree_celsius, verbatim_doc_comment)]
        value: proto::Temperature,
    },

    /// Set the Modbus communication baud rate for the device.
    /// **Important:** The new baud rate setting will only take effect after the
    /// R4DCB08 module is **power cycled** (turned off and then on again).
    /// All devices on a single Modbus RTU bus segment must use the same baud rate.
    #[clap(verbatim_doc_comment)]
    SetBaudRate {
        /// The new baud rate to set.
        /// Supported values: 1200, 2400, 4800, 9600, 19200.
        #[arg(value_parser = parse_baud_rate, verbatim_doc_comment)]
        new_baud_rate: proto::BaudRate,
    },

    /// Set a new Modbus RTU device address for the device.
    /// The new address must be unique on the RS485 bus (1-247) to avoid conflicts.
    /// **Important:** After changing the address, you must use the new address
    /// for subsequent communication with this device.
    #[clap(verbatim_doc_comment)]
    SetAddress {
        /// The new Modbus RTU device address (1 to 247).
        /// Can be specified in decimal or hexadecimal (e.g., "0x01" to "0xF7").
        #[arg(value_parser = parse_address, verbatim_doc_comment)]
        address: proto::Address,
    },

    /// Configure the automatic temperature reporting interval.
    /// When enabled (interval > 0), the module may periodically send temperature data
    /// unsolicitedly (behavior depends on module firmware and Modbus mode).
    #[clap(verbatim_doc_comment)]
    SetAutomaticReport {
        /// Reporting interval in seconds.
        /// `0` disables automatic reporting (query mode).
        /// `1` to `255` sets the interval in seconds.
        #[arg(value_parser = parse_automatic_report, verbatim_doc_comment)]
        report_time: proto::AutomaticReport,
    },

    /// Restore the R4DCB08 module to its factory default settings.
    /// This resets Modbus address, baud rate, temperature corrections, and auto-report settings.
    /// **Warning:** This is an irreversible operation.
    /// **Important:** A **power cycle** is required to complete the reset.
    #[clap(verbatim_doc_comment)]
    FactoryReset,
}

const fn about_text() -> &'static str {
    "R4DCB08 Temperature Collector CLI - Interact with R4DCB08 temperature modules via Modbus RTU/TCP."
}

#[derive(Parser, Debug)]
#[command(name="tempcol", author, version, about=about_text(), long_about = None, propagate_version = true)]
struct CliArgs {
    /// Configure verbosity of logging output.
    /// -v for info, -vv for debug, -vvv for trace. Default is off.
    #[command(flatten)]
    verbose: Verbosity<WarnLevel>,

    /// Specifies the connection method and device-specific commands.
    #[command(subcommand)]
    pub connection: CliConnection,

    /// Modbus I/O timeout for read/write operations.
    /// Examples: "1s", "500ms".
    #[arg(global = true, long, default_value = "200ms", value_parser = humantime::parse_duration, verbatim_doc_comment)]
    timeout: Duration,

    /// Minimum delay between multiple Modbus commands sent to the same device.
    /// Important for Modbus RTU, especially with USB-to-RS485 converters that need time
    /// to switch between transmitting (TX) and receiving (RX) modes.
    /// Examples: "50ms", "100ms".
    #[arg(global = true, long, default_value = "50ms", value_parser = humantime::parse_duration, verbatim_doc_comment)]
    delay: Duration,
}

fn logging_init(loglevel: LevelFilter) -> LoggerHandle {
    let log_handle = Logger::try_with_env_or_str(loglevel.as_str())
        .expect("Cannot init logging")
        .start()
        .expect("Cannot start logging");

    panic::set_hook(Box::new(|panic_info| {
        let (filename, line, column) = panic_info
            .location()
            .map(|loc| (loc.file(), loc.line(), loc.column()))
            .unwrap_or(("<unknown_file>", 0, 0)); // Provide defaults

        let cause_str = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "<unknown_panic_cause>"
        };

        error!(
            target: "panic", // Optional target for filtering
            "Thread '{}' panicked at '{}': {}:{} - Cause: {}",
            std::thread::current().name().unwrap_or("<unnamed>"),
            filename,
            line,
            column,
            cause_str
        );
    }));
    log_handle
}

macro_rules! print_temperature {
    ($device:expr) => {
        let temperatures = $device
            .read_temperatures()
            .with_context(|| "Cannot read temperatures")??;
        println!("Temperatures (°C): {}", temperatures)
    };
}

macro_rules! print_temperature_correction {
    ($device:expr) => {
        let corrections = $device
            .read_temperature_correction()
            .with_context(|| "Cannot read temperature corrections")??;
        println!("Temperature corrections (°C): {}", corrections);
    };
}
macro_rules! print_baud_rate {
    ($device:expr) => {
        let baud_rate = $device
            .read_baud_rate()
            .with_context(|| "Cannot read baud rate")??;
        println!("Baud rate: {}", baud_rate);
    };
}
macro_rules! print_automatic_report {
    ($device:expr) => {
        let rsp = $device
            .read_automatic_report()
            .with_context(|| "Cannot read automatic report")??;
        println!("Automatic report in seconds (0 means disabled): {}", *rsp);
    };
}

/// Calculates the minimum recommended delay for Modbus RTU based on baud rate.
/// This is typically 3.5 character times.
fn minimum_rtu_delay(baud_rate: &proto::BaudRate) -> Duration {
    // 1 start bit + 8 data bits + 1 parity bit (optional, assumed none or 1 for char time) + 1 stop bit = 10 bits
    // Or 1 start bit + 8 data bits + 2 stop bits = 11 bits if no parity
    // The protocol document implies "N, 8, 1" (No parity, 8 data, 1 stop) -> 10 bits per char for data part.
    // Modbus standard often assumes 11 bits for character time calculation in silence intervals.
    let bits_per_char = 11.0; // Common assumption for Modbus character time (start + 8 data + parity/stop + stop)
    let rate = *baud_rate as u16 as f64;
    if rate == 0.0 {
        // Avoid division by zero, default to a safe delay
        return Duration::from_millis(16);
    }

    let char_time_secs = bits_per_char / rate;
    let inter_frame_delay_secs = 3.5 * char_time_secs;
    let delay_micros = (inter_frame_delay_secs * 1_000_000.0) as u64;

    // Modbus spec minimum silence is 1.75ms for baud rates > 19200, but this device max is 19200.
    // For robustness, especially with converters, a slightly higher practical minimum might be good.
    const PRACTICAL_MIN_INTER_FRAME_DELAY_MICROS: u64 = 1_750; // 1.75 ms
    Duration::from_micros(delay_micros.max(PRACTICAL_MIN_INTER_FRAME_DELAY_MICROS))
}

/// Checks if the user-provided RTU delay is sufficient; if not, uses the calculated minimum.
fn check_rtu_delay(user_delay: Duration, baud_rate: &proto::BaudRate) -> Duration {
    let min_rtu_delay = minimum_rtu_delay(baud_rate);
    if user_delay < min_rtu_delay {
        warn!(
            "User-defined RTU delay of {user_delay:?} is below the recommended minimum of {min_rtu_delay:?} for {baud_rate} baud. Using minimum."
        );
        min_rtu_delay
    } else {
        user_delay
    }
}

fn rtu_scan(
    device_name: &str,
    baud_rate: &proto::BaudRate,
    timeout: Duration,
) -> Result<proto::Address> {
    let mut client = R4DCB08::new(
        tokio_modbus::client::sync::rtu::connect_slave(
            &r4dcb08_lib::tokio_serial::serial_port_builder(device_name, baud_rate),
            tokio_modbus::Slave(*proto::Address::BROADCAST),
        )
        .with_context(|| {
            format!("Cannot open serial port {device_name} for scanning at baud {baud_rate}")
        })?,
    );
    client.set_timeout(timeout);

    Ok(client
        .read_address()
        .with_context(|| "Cannot read address")??)
}

/// Prompts the user for confirmation when an operation requires only one module on the bus.
fn confirm_only_one_module_connected() -> Result<bool> {
    println!(
        "WARNING: This operation should only be performed if a SINGLE R4DCB08 module \
         is connected to the Modbus RTU bus segment."
    );
    println!("Multiple devices may lead to unpredictable behavior or communication errors.");
    Confirm::new()
        .with_prompt("Do you want to continue with this understanding?")
        .default(false)
        .show_default(true)
        .interact()
        .context("Failed to get user confirmation.")
}

fn main() -> Result<()> {
    let args = CliArgs::parse();

    // Initialize logging as early as possible
    let _log_handle = logging_init(args.verbose.log_level_filter());
    info!(
        "R4DCB08 CLI started. Log level: {}",
        args.verbose.log_level_filter()
    );

    // Handle RTU Scan separately as it has a different workflow
    if let CliConnection::RtuScan {
        device: device_name,
    } = &args.connection
    {
        if !confirm_only_one_module_connected()? {
            info!("RTU scan aborted by user.");
            return Ok(());
        }
        info!("Starting RTU scan on device: {device_name}");
        let baud_rates_to_scan = [
            proto::BaudRate::B19200, // Start with common/faster rates
            proto::BaudRate::B9600,
            proto::BaudRate::B4800,
            proto::BaudRate::B2400,
            proto::BaudRate::B1200,
        ];

        for baud_rate in baud_rates_to_scan {
            print!("Scanning at {baud_rate} baud on {device_name} ... ");
            stdout().flush().context("Failed to flush stdout")?;

            let scan_attempt_delay = check_rtu_delay(args.delay, &baud_rate);

            match rtu_scan(device_name, &baud_rate, args.timeout) {
                Ok(address) => {
                    println!("SUCCESS!");
                    println!("  Device found with RS485 Address: {address}");
                    println!("  Communication established at Baud rate: {baud_rate}");
                    return Ok(()); // Exit after first successful find
                }
                Err(error) => {
                    println!("failed.");
                    // Log the detailed error only at trace or debug level for scan failures
                    if args.verbose.log_level_filter() >= LevelFilter::Debug {
                        debug!("Scan error at {baud_rate} baud: {error:?}");
                    } else {
                        trace!("Scan error at {baud_rate} baud: {error:?}"); // Keep for trace if needed
                    }
                    std::thread::sleep(scan_attempt_delay); // Wait before trying next baud rate
                }
            }
        }
        // If loop completes, no device was found
        bail!(
            "No R4DCB08 device found on {} across all supported baud rates.",
            device_name
        );
    }

    // --- Setup for regular TCP/RTU commands ---
    let mut effective_delay = args.delay;

    let (mut client, command_to_execute) = match &args.connection {
        CliConnection::Tcp {
            address: tcp_address_str,
            command,
        } => {
            let socket_addr = tcp_address_str
                .parse()
                .with_context(|| format!("Invalid TCP address format: '{tcp_address_str}'"))?;
            info!("Attempting to connect via TCP to {socket_addr}...");
            let modbus_ctx =
                tokio_modbus::client::sync::tcp::connect(socket_addr).with_context(|| {
                    format!("Failed to connect to Modbus TCP device at {socket_addr}")
                })?;
            (R4DCB08::new(modbus_ctx), command)
        }
        CliConnection::Rtu {
            device: device_name,
            baud_rate,
            address: device_address,
            command,
        } => {
            let final_device_address = if command == &CliCommands::QueryAddress {
                if !confirm_only_one_module_connected()? {
                    info!("QueryAddress command aborted by user.");
                    return Ok(());
                }
                // For QueryAddress, always use the broadcast address.
                if device_address != &proto::Address::BROADCAST {
                    info!(
                        "Overriding specified address {} with broadcast address {} for QueryAddress command.",
                        device_address,
                        proto::Address::BROADCAST
                    );
                }
                proto::Address::BROADCAST
            } else {
                *device_address
            };

            info!(
                "Attempting to connect via RTU to device {device_name} (Address: {final_device_address}, Baud: {baud_rate})..."
            );

            // Adjust delay based on baud rate for RTU
            effective_delay = check_rtu_delay(args.delay, baud_rate);
            debug!("Using effective RTU delay of {effective_delay:?}");

            (
                R4DCB08::new(tokio_modbus::client::sync::rtu::connect_slave(
                    &r4dcb08_lib::tokio_serial::serial_port_builder(device_name, baud_rate),
                    tokio_modbus::Slave(*final_device_address),
                )?),
                command,
            )
        }
        CliConnection::RtuScan { .. } => unreachable!("RtuScan should be handled earlier."),
    };

    let effective_delay = effective_delay;
    client.set_timeout(args.timeout);

    match command_to_execute {
        CliCommands::Daemon {
            poll_interval,
            mode,
        } => {
            info!("Starting daemon mode with polling interval: {poll_interval:?}");
            match mode {
                DaemonMode::Stdout => loop {
                    debug!("Daemon: Reading temperatures for stdout...");
                    print_temperature!(&mut client);
                    std::thread::sleep(effective_delay.max(*poll_interval));
                },
                DaemonMode::Mqtt { mqtt_config_path } => {
                    let config_path_str =
                        mqtt_config_path.unwrap_or_else(|| "mqtt.yaml".to_string());

                    info!("Loading MQTT configuration from {}...", config_path_str);

                    let contents = match File::open(&config_path_str) {
                        Ok(mut file) => {
                            let mut s = String::new();
                            file.read_to_string(&mut s).with_context(|| {
                                format!("Failed to read MQTT config file at '{}'", config_path_str)
                            })?;
                            s
                        }
                        Err(e) => {
                            // Provide a specific error message if the default file is not found
                            if config_path_str == "mqtt.yaml" && e.kind() == std::io::ErrorKind::NotFound {
                                bail!("Default MQTT config file 'mqtt.yaml' not found in the current directory. Please provide a path using --mqtt-config-path or create 'mqtt.yaml'.");
                            }
                            return Err(e).with_context(|| {
                                format!("Failed to open MQTT config file at '{}'", config_path_str)
                            });
                        }
                    };

                    let loaded_config: MqttConfigFile =
                        serde_yaml::from_str(&contents).with_context(|| {
                            format!(
                                "Failed to parse MQTT config file at '{}'",
                                config_path_str
                            )
                        })?;

                    // The URL will be made non-optional in MqttConfigFile in a subsequent step.
                    // For now, if it's None, this will panic, which is acceptable temporarily.
                    // This unwrap will be removed once MqttConfigFile.url is no longer Option<String>.
                    let final_url = loaded_config.url.clone().with_context(|| {
                        format!(
                            "MQTT URL not found in config file '{}'. It is a required field.",
                            config_path_str
                        )
                    })?;
                    let final_username = loaded_config.username.clone();
                    let final_password = loaded_config.password.clone();
                    let final_topic = loaded_config
                        .topic
                        .clone()
                        .unwrap_or_else(|| "r4dcb08".to_string());
                    let final_qos = loaded_config.qos.unwrap_or(0);
                    let final_client_id = loaded_config.client_id.clone();

                    info!("Daemon: Configuring MQTT client for broker at {}", final_url);
                    let mut create_opts_builder = mqtt::CreateOptionsBuilder::new()
                        .server_uri(&final_url);

                    if let Some(client_id_str) = &final_client_id {
                        create_opts_builder = create_opts_builder.client_id(client_id_str.as_str());
                    }
                    // If final_client_id is None, paho-mqtt will generate a random one.

                    let create_opts = create_opts_builder.finalize();

                    let mut mqtt_client = mqtt::Client::new(create_opts).with_context(|| {
                        format!("Error creating MQTT client for URL: {}", final_url)
                    })?;

                    mqtt_client.set_timeout(Duration::from_secs(10)); // MQTT operation timeout

                    let mut conn_builder = mqtt::ConnectOptionsBuilder::new();
                    conn_builder
                        .keep_alive_interval(Duration::from_secs(30))
                        .clean_session(true) // Typically true for telemetry publishers
                        .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(300)); // Enable auto-reconnect

                    if let Some(user_name_str) = &final_username { // Borrow final_username
                        conn_builder.user_name(user_name_str.as_str());
                    }
                    if let Some(password_str) = &final_password { // Borrow final_password
                        conn_builder.password(password_str.as_str());
                    }
                    let conn_opts = conn_builder.finalize();

                    info!("Daemon: Connecting to MQTT broker...");
                    mqtt_client.connect(conn_opts).with_context(|| {
                        "MQTT client failed to connect. Check URL, credentials, and broker status."
                    })?;
                    info!("Daemon: Successfully connected to MQTT broker.");

                    loop {
                        debug!("Daemon: Reading temperatures for MQTT publishing...");
                        match client.read_temperatures() {
                            Ok(temperatures) => {
                                trace!("Temperatures read: {temperatures:?}");
                                for (ch_idx, temp_val) in temperatures.iter().enumerate() {
                                    let channel_topic = format!("{}/CH{ch_idx}", final_topic); // Use final_topic
                                    let payload_str = temp_val.to_string(); // Uses Display impl (e.g., "21.9" or "NAN")

                                    debug!(
                                        "Publishing to MQTT: Topic='{}', Payload='{}', QoS={}",
                                        channel_topic, payload_str, final_qos // Use final_qos
                                    );
                                    let msg = mqtt::Message::new_retained(
                                        channel_topic.clone(),
                                        payload_str.clone(),
                                        final_qos, // Use final_qos
                                    );

                                    if let Err(e) = mqtt_client.publish(msg) {
                                        error!(
                                            "Failed to publish MQTT message to {channel_topic}: {e}. Attempting to reconnect..."
                                        );
                                        // paho-mqtt handles reconnect if configured, otherwise this error is critical
                                        if !mqtt_client.is_connected() {
                                            warn!(
                                                "MQTT client disconnected. Attempting reconnect..."
                                            );
                                            if let Err(rc_err) = mqtt_client.reconnect() {
                                                error!(
                                                    "MQTT reconnect failed: {rc_err}. Exiting daemon."
                                                );
                                                bail!(
                                                    "MQTT reconnect failed, cannot continue daemon mode."
                                                );
                                            } else {
                                                info!("MQTT reconnected successfully.");
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Daemon: Error reading temperatures: {e}");
                                // Decide on retry strategy or if to bail
                            }
                        }
                        std::thread::sleep(effective_delay.max(*poll_interval));
                    }
                }
            }
        }
        CliCommands::Read => {
            info!("Executing: Read Temperatures");
            print_temperature!(&mut client);
        }
        CliCommands::ReadCorrection => {
            info!("Executing: Read Temperature Corrections");
            print_temperature_correction!(&mut client);
        }
        CliCommands::ReadBaudRate => {
            info!("Executing: Read Baud Rate");
            print_baud_rate!(&mut client);
        }
        CliCommands::ReadAutomaticReport => {
            info!("Executing: Read Automatic Report Interval");
            print_automatic_report!(&mut client);
        }
        CliCommands::ReadAll => {
            info!("Executing: Read All Device Values");
            print_temperature!(&mut client);
            std::thread::sleep(effective_delay);
            print_temperature_correction!(&mut client);
            std::thread::sleep(effective_delay);
            print_baud_rate!(&mut client);
            std::thread::sleep(effective_delay);
            print_automatic_report!(&mut client);
        }
        CliCommands::QueryAddress => {
            info!("Executing: Query Device Address (Broadcast)");
            let address = client
                .read_address()
                .with_context(|| "Cannot read RS485 address")??;
            println!("RS485 address: {address}");
        }
        CliCommands::SetCorrection { channel, value } => {
            info!("Executing: Set Temperature Correction for Channel {channel} to {value} °C");
            client
                .set_temperature_correction(*channel, *value)
                .with_context(|| {
                    format!("Failed to set temperature correction for channel {channel} to {value}")
                })??;
            println!(
                "Temperature correction for channel {channel} set to {value} °C successfully."
            );
        }
        CliCommands::SetBaudRate { new_baud_rate } => {
            info!("Executing: Set Baud Rate to {new_baud_rate}");
            client
                .set_baud_rate(*new_baud_rate)
                .with_context(|| format!("Failed to set baud rate to {new_baud_rate}"))??;
            println!(
                "Baud rate set to {new_baud_rate}. Important: Power cycle the device for this change to take effect!"
            );
        }
        CliCommands::SetAddress {
            address: new_address,
        } => {
            info!("Executing: Set Device Address to {new_address}");
            client
                .set_address(*new_address)
                .with_context(|| format!("Failed to set new RS485 address to {new_address}"))??;
            println!(
                "Device address successfully set to {new_address}. Subsequent communication must use this new address."
            );
        }
        CliCommands::SetAutomaticReport { report_time } => {
            info!(
                "Executing: Set Automatic Report Interval to {} seconds",
                report_time.as_secs()
            );
            client
                .set_automatic_report(*report_time)
                .with_context(|| {
                    format!(
                        "Failed to set automatic report interval to {}s",
                        report_time.as_secs()
                    )
                })??;
            if report_time.is_disabled() {
                println!("Automatic temperature reporting disabled successfully.");
            } else {
                println!(
                    "Automatic temperature reporting interval set to {} seconds successfully.",
                    report_time.as_secs()
                );
            }
        }
        CliCommands::FactoryReset => {
            info!("Executing: Factory Reset");
            println!(
                "WARNING: This will reset the R4DCB08 module to its factory default settings:\n\
                 - Modbus Address: {}\n\
                 - Baud Rate: {}\n\
                 - Temperature Corrections: All channels set to 0.0 °C\n\
                 - Automatic Report Interval: Disabled (0 seconds)\n",
                proto::Address::default(),
                proto::BaudRate::default()
            );
            println!(
                "After this operation, the device may become unresponsive on the current settings.\n\
                 A POWER CYCLE (disconnect and reconnect power) IS REQUIRED to complete the reset."
            );

            if !Confirm::new()
                .with_prompt("Are you sure you want to proceed with the factory reset?")
                .default(false)
                .show_default(true)
                .interact()?
            {
                info!("Factory reset aborted by user.");
                return Ok(());
            }

            // Try a quick read to confirm current connectivity before reset.
            print!("Verifying current connection to device before reset... ");
            stdout().flush().context("Failed to flush stdout")?;

            match client.read_temperatures() {
                Ok(Ok(_)) => {
                    println!("connection OK.");
                }
                Ok(Err(err)) => {
                    println!("failed");
                    return Err(err.into());
                }
                Err(err) => {
                    println!("failed");
                    return Err(err.into());
                }
            }

            std::thread::sleep(effective_delay);

            info!("Sending factory reset command...");
            if let Err(error) = client.factory_reset() {
                let ignore_error = if let tokio_modbus::Error::Transport(error) = &error {
                    if error.kind() == std::io::ErrorKind::TimedOut {
                        // After the a successful factory reset we get no response :-(
                        debug!("Reset to factory settings returned TimeOut error, can be ignored");
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if !ignore_error {
                    return Err(error.into());
                }
            }
            println!("Factory reset command sent successfully.");
            println!(
                "IMPORTANT: Please disconnect and reconnect power to the R4DCB08 module to complete the reset process."
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimum_rtu_delay_calculation() {
        // Based on 11 bits per character (1 start, 8 data, 1 parity/none, 1 stop as common Modbus assumption)
        // Char time = 11 / baud
        // 3.5 char times = 3.5 * 11 / baud = 38.5 / baud
        assert_eq!(
            minimum_rtu_delay(&proto::BaudRate::B1200).as_micros(),
            32083
        ); // 38.5 / 1200 * 1e6
        assert_eq!(
            minimum_rtu_delay(&proto::BaudRate::B2400).as_micros(),
            16041
        ); // 38.5 / 2400 * 1e6
        assert_eq!(minimum_rtu_delay(&proto::BaudRate::B4800).as_micros(), 8020); // 38.5 / 4800 * 1e6
        assert_eq!(minimum_rtu_delay(&proto::BaudRate::B9600).as_micros(), 4010); // 38.5 / 9600 * 1e6
        assert_eq!(
            minimum_rtu_delay(&proto::BaudRate::B19200).as_micros(),
            2005
        ); // 38.5 / 19200 * 1e6
        // Test the practical minimum clamp
        // Assuming a hypothetical baud rate that would result in < 1750us
        // e.g., 38400 baud: 38.5 / 38400 * 1e6 = 1002.6 us, should be clamped to 1750us
        // Since our max is 19200, this clamp is mostly for future-proofing or extreme cases.
        // For 19200, 2005us > 1750us, so no clamp.
        // Let's simulate a very high baud rate to test the clamp:
        // If we had a BaudRate::B38400, it would be clamped.
        // For now, the existing rates are above the clamp threshold.
    }

    #[test]
    fn test_check_rtu_delay() {
        let br_9600 = proto::BaudRate::B9600;
        let min_delay_9600 = minimum_rtu_delay(&br_9600); // Approx 4010 us

        assert_eq!(
            check_rtu_delay(Duration::from_millis(3), &br_9600),
            min_delay_9600
        ); // 3ms < min_delay_9600
        assert_eq!(
            check_rtu_delay(Duration::from_millis(5), &br_9600),
            Duration::from_millis(5)
        ); // 5ms > min_delay_9600
        assert_eq!(check_rtu_delay(min_delay_9600, &br_9600), min_delay_9600);
    }
}
