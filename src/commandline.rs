use crate::mqtt::MqttConfig;
use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use r4dcb08_lib::protocol as proto;
use std::time::Duration;

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
pub enum CliConnection {
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

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum DaemonOutput {
    /// Continuously read temperatures and print them to the standard output (console).
    Console,
    /// Continuously read temperatures and publish them to an MQTT broker.
    Mqtt {
        /// The configuration file for the MQTT broker
        #[arg(long, default_value_t = MqttConfig::DEFAULT_CONFIG_FILE.to_string())]
        config_file: String,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum CliCommands {
    /// Run in daemon mode: continuously poll temperatures at a specified interval.
    /// Output can be directed to stdout or an MQTT broker.
    #[clap(verbatim_doc_comment)]
    Daemon {
        /// Interval for fetching temperatures (e.g., "10s", "1m")
        #[arg(value_parser = humantime::parse_duration, short, long, default_value = "2sec", verbatim_doc_comment)]
        poll_interval: Duration,

        /// Specifies the output.
        #[command(subcommand)]
        output: DaemonOutput,
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
pub struct CliArgs {
    /// Configure verbosity of logging output.
    /// -v for info, -vv for debug, -vvv for trace. Default is off.
    #[command(flatten)]
    pub verbose: Verbosity<WarnLevel>,

    /// Specifies the connection method and device-specific commands.
    #[command(subcommand)]
    pub connection: CliConnection,

    /// Modbus I/O timeout for read/write operations.
    /// Examples: "1s", "500ms".
    #[arg(global = true, long, default_value = "200ms", value_parser = humantime::parse_duration, verbatim_doc_comment)]
    pub timeout: Duration,

    /// Minimum delay between multiple Modbus commands sent to the same device.
    /// Important for Modbus RTU, especially with USB-to-RS485 converters that need time
    /// to switch between transmitting (TX) and receiving (RX) modes.
    /// Examples: "50ms", "100ms".
    #[arg(global = true, long, default_value = "50ms", value_parser = humantime::parse_duration, verbatim_doc_comment)]
    pub delay: Duration,
}
