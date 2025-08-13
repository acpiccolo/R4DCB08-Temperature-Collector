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
use clap::Parser;
use dialoguer::Confirm;
use flexi_logger::{Logger, LoggerHandle};
use log::*;
use r4dcb08_lib::{protocol as proto, tokio_sync_client::R4DCB08};
use std::io::{Write, stdout};
use std::{panic, time::Duration};

mod commandline;
mod mqtt;

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
            .with_context(|| "Cannot read temperatures")?;
        println!("Temperatures (°C): {}", temperatures)
    };
}

macro_rules! print_temperature_correction {
    ($device:expr) => {
        let corrections = $device
            .read_temperature_correction()
            .with_context(|| "Cannot read temperature corrections")?;
        println!("Temperature corrections (°C): {}", corrections);
    };
}
macro_rules! print_baud_rate {
    ($device:expr) => {
        let baud_rate = $device
            .read_baud_rate()
            .with_context(|| "Cannot read baud rate")?;
        println!("Baud rate: {}", baud_rate);
    };
}
macro_rules! print_automatic_report {
    ($device:expr) => {
        let rsp = $device
            .read_automatic_report()
            .with_context(|| "Cannot read automatic report")?;
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
            &r4dcb08_lib::tokio_common::serial_port_builder(device_name, baud_rate),
            tokio_modbus::Slave(*proto::Address::BROADCAST),
        )
        .with_context(|| {
            format!("Cannot open serial port {device_name} for scanning at baud {baud_rate}")
        })?,
    );
    client.set_timeout(timeout);

    Ok(client
        .read_address()
        .with_context(|| "Cannot read address")?)
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

/// Handles the RTU scan command.
///
/// This function iterates through all supported baud rates and attempts to find a device.
fn handle_rtu_scan(
    device: &str,
    delay: Duration,
    timeout: Duration,
    log_level: LevelFilter,
) -> Result<()> {
    if !confirm_only_one_module_connected()? {
        info!("RTU scan aborted by user.");
        return Ok(());
    }
    info!("Starting RTU scan on device: {device}");
    let baud_rates_to_scan = [
        proto::BaudRate::B19200, // Start with common/faster rates
        proto::BaudRate::B9600,
        proto::BaudRate::B4800,
        proto::BaudRate::B2400,
        proto::BaudRate::B1200,
    ];

    for baud_rate in baud_rates_to_scan {
        print!("Scanning at {baud_rate} baud on {device} ... ");
        stdout().flush().context("Failed to flush stdout")?;

        let scan_attempt_delay = check_rtu_delay(delay, &baud_rate);

        match rtu_scan(device, &baud_rate, timeout) {
            Ok(address) => {
                println!("SUCCESS!");
                println!("  Device found with RS485 Address: {address}");
                println!("  Communication established at Baud rate: {baud_rate}");
                return Ok(()); // Exit after first successful find
            }
            Err(error) => {
                println!("failed.");
                // Log the detailed error only at trace or debug level for scan failures
                if log_level >= LevelFilter::Debug {
                    debug!("Scan error at {baud_rate} baud: {error:?}");
                } else {
                    trace!("Scan error at {baud_rate} baud: {error:?}"); // Keep for trace if needed
                }
                std::thread::sleep(scan_attempt_delay); // Wait before trying next baud rate
            }
        }
    }
    // If loop completes, no device was found
    bail!("No R4DCB08 device found on {device} across all supported baud rates.");
}

/// Creates a new R4DCB08 client based on the provided command-line arguments.
fn create_client<'a>(
    connection: &'a commandline::CliConnection,
    delay: &mut Duration,
) -> Result<(R4DCB08, &'a commandline::CliCommands)> {
    let (client, command_to_execute) = match connection {
        commandline::CliConnection::Tcp {
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
        commandline::CliConnection::Rtu {
            device,
            baud_rate,
            address,
            command,
        } => {
            let final_device_address = if command == &commandline::CliCommands::QueryAddress {
                if !confirm_only_one_module_connected()? {
                    info!("QueryAddress command aborted by user.");
                    std::process::exit(0);
                }
                // For QueryAddress, always use the broadcast address.
                if address != &proto::Address::BROADCAST {
                    info!(
                        "Overriding specified address {address} with broadcast address {} for QueryAddress command.",
                        proto::Address::BROADCAST
                    );
                }
                proto::Address::BROADCAST
            } else {
                *address
            };

            info!(
                "Attempting to connect via RTU to device {device} (Address: {final_device_address}, Baud: {baud_rate})..."
            );
            *delay = check_rtu_delay(*delay, baud_rate);
            (
                R4DCB08::new(tokio_modbus::client::sync::rtu::connect_slave(
                    &r4dcb08_lib::tokio_common::serial_port_builder(device, baud_rate),
                    tokio_modbus::Slave(*final_device_address),
                )?),
                command,
            )
        }
        commandline::CliConnection::RtuScan { .. } => {
            unreachable!("RtuScan should be handled earlier.")
        }
    };
    Ok((client, command_to_execute))
}

/// Handles the factory reset command.
///
/// This function prompts the user for confirmation, verifies the connection to the device,
/// and then sends the factory reset command.
fn handle_factory_reset(client: &mut R4DCB08, delay: Duration) -> Result<()> {
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
        Ok(_) => {
            println!("connection OK.");
        }
        Err(err) => {
            println!("failed");
            return Err(err.into());
        }
    }

    std::thread::sleep(delay);

    info!("Sending factory reset command...");
    if let Err(error) = client.factory_reset() {
        let ignore_error = if let r4dcb08_lib::tokio_common::Error::TokioError(
            tokio_modbus::Error::Transport(error),
        ) = &error
        {
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
    Ok(())
}

fn main() -> Result<()> {
    let args = commandline::CliArgs::parse();

    // 1. Initialize logging as early as possible
    let _log_handle = logging_init(args.verbose.log_level_filter());
    info!(
        "R4DCB08 CLI started. Log level: {}",
        args.verbose.log_level_filter()
    );

    // 2. Handle RTU Scan command separately as it has a different workflow
    if let commandline::CliConnection::RtuScan { device } = &args.connection {
        return handle_rtu_scan(
            device,
            args.delay,
            args.timeout,
            args.verbose.log_level_filter(),
        );
    }

    // 3. Setup for regular TCP/RTU commands
    let mut delay = args.delay;
    let (mut client, command_to_execute) = create_client(&args.connection, &mut delay)?;
    client.set_timeout(args.timeout);

    // 4. Execute the command
    match command_to_execute {
        commandline::CliCommands::Daemon {
            poll_interval,
            output,
        } => {
            info!("Starting daemon mode: output={output:?}, interval={poll_interval:?}");
            match output {
                commandline::DaemonOutput::Console => loop {
                    debug!("Daemon: Reading temperatures for stdout...");
                    print_temperature!(&mut client);
                    std::thread::sleep(delay.max(*poll_interval));
                },
                commandline::DaemonOutput::Mqtt { config_file } => {
                    mqtt::run_daemon(&mut client, &delay, poll_interval, config_file)?;
                }
            }
        }
        commandline::CliCommands::Read => {
            info!("Executing: Read Temperatures");
            print_temperature!(&mut client);
        }
        commandline::CliCommands::ReadCorrection => {
            info!("Executing: Read Temperature Corrections");
            print_temperature_correction!(&mut client);
        }
        commandline::CliCommands::ReadBaudRate => {
            info!("Executing: Read Baud Rate");
            print_baud_rate!(&mut client);
        }
        commandline::CliCommands::ReadAutomaticReport => {
            info!("Executing: Read Automatic Report Interval");
            print_automatic_report!(&mut client);
        }
        commandline::CliCommands::ReadAll => {
            info!("Executing: Read All Device Values");
            print_temperature!(&mut client);
            std::thread::sleep(delay);
            print_temperature_correction!(&mut client);
            std::thread::sleep(delay);
            print_baud_rate!(&mut client);
            std::thread::sleep(delay);
            print_automatic_report!(&mut client);
        }
        commandline::CliCommands::QueryAddress => {
            info!("Executing: Query Device Address (Broadcast)");
            let address = client
                .read_address()
                .with_context(|| "Cannot read RS485 address")?;
            println!("RS485 address: {address}");
        }
        commandline::CliCommands::SetCorrection { channel, value } => {
            info!("Executing: Set Temperature Correction for Channel {channel} to {value} °C");
            client
                .set_temperature_correction(*channel, *value)
                .with_context(|| {
                    format!("Failed to set temperature correction for channel {channel} to {value}")
                })?;
            println!(
                "Temperature correction for channel {channel} set to {value} °C successfully."
            );
        }
        commandline::CliCommands::SetBaudRate { new_baud_rate } => {
            info!("Executing: Set Baud Rate to {new_baud_rate}");
            client
                .set_baud_rate(*new_baud_rate)
                .with_context(|| format!("Failed to set baud rate to {new_baud_rate}"))?;
            println!(
                "Baud rate set to {new_baud_rate}. Important: Power cycle the device for this change to take effect!"
            );
        }
        commandline::CliCommands::SetAddress {
            address: new_address,
        } => {
            info!("Executing: Set Device Address to {new_address}");
            client
                .set_address(*new_address)
                .with_context(|| format!("Failed to set new RS485 address to {new_address}"))?;
            println!(
                "Device address successfully set to {new_address}. Subsequent communication must use this new address."
            );
        }
        commandline::CliCommands::SetAutomaticReport { report_time } => {
            info!(
                "Executing: Set Automatic Report Interval to {} seconds",
                report_time.as_secs()
            );
            client.set_automatic_report(*report_time).with_context(|| {
                format!(
                    "Failed to set automatic report interval to {}s",
                    report_time.as_secs()
                )
            })?;
            if report_time.is_disabled() {
                println!("Automatic temperature reporting disabled successfully.");
            } else {
                println!(
                    "Automatic temperature reporting interval set to {} seconds successfully.",
                    report_time.as_secs()
                );
            }
        }
        commandline::CliCommands::FactoryReset => {
            handle_factory_reset(&mut client, delay)?;
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
