use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use dialoguer::Confirm;
use flexi_logger::{Logger, LoggerHandle};
use log::*;
use paho_mqtt as mqtt;
use r4dcb08_lib::{protocol as proto, tokio_sync_client::R4DCB08};
use std::io::{stdout, Write};
use std::{fmt, ops::Deref, panic, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq)]
struct BaudRate(proto::BaudRate);

impl fmt::Display for BaudRate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_u16())
    }
}

impl BaudRate {
    pub fn iter() -> core::array::IntoIter<Self, 5> {
        [
            Self(proto::BaudRate::B1200),
            Self(proto::BaudRate::B2400),
            Self(proto::BaudRate::B4800),
            Self(proto::BaudRate::B9600),
            Self(proto::BaudRate::B19200),
        ]
        .into_iter()
    }

    pub fn from(baud_rate: proto::BaudRate) -> Self {
        Self(baud_rate)
    }

    pub fn from_u16(baud_rate: u16) -> Result<Self> {
        match baud_rate {
            1200 => Ok(Self(proto::BaudRate::B1200)),
            2400 => Ok(Self(proto::BaudRate::B2400)),
            4800 => Ok(Self(proto::BaudRate::B4800)),
            9600 => Ok(Self(proto::BaudRate::B9600)),
            19200 => Ok(Self(proto::BaudRate::B19200)),
            _ => bail!("Baud rate must by any value of 1200, 2400, 4800, 9600, 19200."),
        }
    }

    pub fn as_u16(&self) -> u16 {
        match self.0 {
            proto::BaudRate::B1200 => 1200,
            proto::BaudRate::B2400 => 2400,
            proto::BaudRate::B4800 => 4800,
            proto::BaudRate::B9600 => 9600,
            proto::BaudRate::B19200 => 19200,
        }
    }

    pub fn minimum_rtu_delay(&self) -> Duration {
        // https://minimalmodbus.readthedocs.io/en/stable/serialcommunication.html#timing-of-the-serial-communications

        fn calc(rate: u64) -> Duration {
            let bit_time = Duration::from_secs_f64(1.0 / rate as f64);
            let char_time = bit_time * 11;
            let result = Duration::from_millis((char_time.as_secs_f64() * 3.5 * 1_000.0) as u64);
            let min_duration = Duration::from_micros(1_750);
            if result < min_duration {
                min_duration
            } else {
                result
            }
        }

        calc(self.as_u16() as u64)
    }
}

impl Deref for BaudRate {
    type Target = proto::BaudRate;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn default_device_name() -> String {
    if cfg!(target_os = "windows") {
        String::from("COM1")
    } else {
        String::from("/dev/ttyUSB0")
    }
}

fn parse_channel(s: &str) -> Result<u8, String> {
    clap_num::number_range(s, proto::CHANNELS_MIN, proto::CHANNELS_MAX)
}

fn parse_address(s: &str) -> Result<u8, String> {
    clap_num::maybe_hex_range(s, proto::ADDRESS_MIN, proto::ADDRESS_MAX)
}

fn parse_baud_rate(s: &str) -> Result<BaudRate, String> {
    let val = s.parse::<u16>().map_err(|e| format!("{e}"))?;
    let val = BaudRate::from_u16(val).map_err(|e| format!("{e}"))?;
    Ok(val)
}

fn parse_degree_celsius(s: &str) -> Result<f32, String> {
    // clap_num::number_range(s, proto::DEGREE_CELSIUS_MIN, proto::DEGREE_CELSIUS_MAX)
    let val = s.parse::<f32>().map_err(|e| format!("{e}"))?;
    if val > proto::DEGREE_CELSIUS_MAX {
        Err(format!("exceeds maximum of {}", proto::DEGREE_CELSIUS_MAX))
    } else if val < proto::DEGREE_CELSIUS_MIN {
        Err(format!(
            "less than minimum of {}",
            proto::DEGREE_CELSIUS_MIN
        ))
    } else {
        Ok(val)
    }
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
enum CliConnection {
    /// Use Modbus/TCP connection
    Tcp {
        // TCP address (e.g. 192.168.0.222:502)
        address: String,

        #[command(subcommand)]
        command: CliCommands,
    },
    /// Use Modbus/RTU connection
    Rtu {
        /// Device
        #[arg(short, long, default_value_t = default_device_name())]
        device: String,

        /// Baud rate any of 1200, 2400, 4800, 9600, 19200
        #[arg(long, default_value_t = BaudRate::from(*proto::FACTORY_DEFAULT_BAUD_RATE), value_parser = parse_baud_rate)]
        baud_rate: BaudRate,

        /// RS485 address from 1 to 247
        #[arg(short, long, default_value_t = proto::FACTORY_DEFAULT_ADDRESS, value_parser = parse_address)]
        address: u8,

        #[command(subcommand)]
        command: CliCommands,
    },
    /// Scan for a R4DCB08 temperature collector on any supported baud rate and query the Modbus RS485 address.
    RtuScan {
        /// Device
        #[arg(short, long, default_value_t = default_device_name())]
        device: String,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq, Default)]
enum DaemonMode {
    #[default]
    /// Print values to stdout [default]
    Stdout,
    /// Send values to a MQTT Broker
    Mqtt {
        /// URL to the MQTT broker like: mqtt://localhost:1883
        url: String,

        /// The user name for authentication with the broker
        #[arg(short, long)]
        username: Option<String>,

        /// The password for authentication with the broker
        #[arg(short, long)]
        password: Option<String>,

        /// MQTT topic
        #[arg(long, default_value = "r4dcb08")]
        topic: String,

        /// Quality of service to use
        #[arg(long, default_value = "0")]
        qos: u8,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
enum CliCommands {
    /// Daemon mode to read the current temperature from all channels
    Daemon {
        /// Interval for repeated polling of the values
        #[arg(value_parser = humantime::parse_duration, short, long, default_value = "2sec")]
        poll_iterval: Duration,

        #[command(subcommand)]
        mode: DaemonMode,
    },

    /// Read the current temperature from all channels
    Read,

    /// Read the current temperature correction values form all channels
    ReadCorrection,

    /// Read the current baud rate
    ReadBaudRate,

    /// Read temperature automatic reporting
    ReadAutomaticReport,

    /// Read all values
    ReadAll,

    /// Queries the current RS485 address, this message is broadcasted.
    /// Only one temperature module can be connected to the RS485 bus, more than one will be wrong!
    QueryAddress,

    /// Set the temperature correction per channel
    SetCorrection {
        /// Temperature sensore channel 0 to 7
        #[arg(value_parser = parse_channel)]
        channel: u8,
        /// Correction value in °Celsius
        #[arg(value_parser = parse_degree_celsius)]
        value: f32,
    },

    /// Set the baud rate. After the command you need to power up the module again!
    SetBaudRate {
        /// The new baud rate any value of 1200, 2400, 4800, 9600, 19200
        #[arg(value_parser = parse_baud_rate)]
        new_baud_rate: BaudRate,
    },

    /// Set the RS485 address
    SetAddress {
        /// The RS485 address can be from 1 to 247
        #[arg(value_parser = parse_address)]
        address: u8,
    },

    /// Set temperature automatic reporting
    SetAutomaticReport {
        /// Report time in seconds. 0 = disabled (default) or from 1 to 255 seconds.
        report_time: u8,
    },

    /// Reset the device to the factory default settings
    FactoryReset,
}

const fn about_text() -> &'static str {
    "R4DCB08 temperature collector/monitor for the command line"
}

#[derive(Parser, Debug)]
#[command(version, about=about_text(), long_about = None)]
struct CliArgs {
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,

    // Connection type
    #[command(subcommand)]
    pub connection: CliConnection,

    /// Serial Input/Output operations timeout
    #[arg(value_parser = humantime::parse_duration, long, default_value = "200ms")]
    timeout: Duration,

    // According to MODBUS specification:
    // Wait at least 3.5 char between frames
    // However, some USB - RS485 dongles requires at least 10ms to switch between TX and RX, so use a save delay between frames
    /// Delay between multiple modbus commands
    #[arg(value_parser = humantime::parse_duration, long, default_value = "50ms")]
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

macro_rules! print_temperature {
    ($device:expr) => {
        let rsp = $device
            .read_temperature()
            .with_context(|| "Cannot read temperature")?;
        println!("Temperatures in °C: {rsp:?}");
    };
}

macro_rules! print_temperature_correction {
    ($device:expr) => {
        let rsp = $device
            .read_temperature_correction()
            .with_context(|| "Cannot read temperature correction")?;
        println!("Temperature corrections in °C: {rsp:?}");
    };
}
macro_rules! print_baud_rate {
    ($device:expr) => {
        let rsp = $device
            .read_baud_rate()
            .with_context(|| "Cannot read baud rate")?;
        println!("Baud rate: {}", BaudRate::from(rsp));
    };
}
macro_rules! print_automatic_report {
    ($device:expr) => {
        let rsp = $device
            .read_automatic_report()
            .with_context(|| "Cannot read automatic report")?;
        println!(
            "Automatic report in seconds (0 means disabled): {}",
            rsp.as_secs()
        );
    };
}

fn check_rtu_delay(delay: Duration, baud_rate: &BaudRate) -> Duration {
    let min_rtu_delay = baud_rate.minimum_rtu_delay();
    if delay < min_rtu_delay {
        warn!(
            "Your RTU delay of {:?} is below the minimum delay of {:?}, fallback to minimum",
            delay, min_rtu_delay
        );
        return min_rtu_delay;
    }
    delay
}

fn rtu_scan(device: &String, baud_rate: &BaudRate, args: &CliArgs) -> Result<u8> {
    let mut d = R4DCB08::new(
        tokio_modbus::client::sync::rtu::connect_slave(
            &r4dcb08_lib::tokio_serial::serial_port_builder(device, baud_rate.as_u16() as u32),
            tokio_modbus::Slave(proto::READ_ADDRESS_BROADCAST_ADDRESS),
        )
        .with_context(|| format!("Cannot open device {} baud rate {}", device, baud_rate))?,
    );
    d.set_timeout(args.timeout);
    let rsp = d
        .read_address()
        .with_context(|| "Cannot read RS485 address")?;
    Ok(rsp)
}

fn confirm_only_one_module_connected() -> Result<bool> {
    println!("Use this command only if ONLY ONE temperature module is connected to the RS485 bus!");
    Ok(Confirm::new()
        .with_prompt("Do you want to continue?")
        .default(false)
        .show_default(true)
        .interact()?)
}

fn main() -> Result<()> {
    let args = CliArgs::parse();

    let mut delay = args.delay;

    let _log_handle = logging_init(args.verbose.log_level_filter());

    if let CliConnection::RtuScan { device } = &args.connection {
        if !confirm_only_one_module_connected()? {
            return Ok(());
        }
        for baud_rate in BaudRate::iter() {
            print!("Scan RTU {} baud rate {} ... ", device, baud_rate);
            stdout().flush().unwrap();
            let delay = check_rtu_delay(delay, &baud_rate);
            match rtu_scan(device, &baud_rate, &args) {
                Ok(address) => {
                    println!("succeeded");
                    println!("RS485 Address: {:#04x}", address);
                    println!("Baud rate: {}", baud_rate);
                    return Ok(());
                }
                Err(error) => {
                    println!("failed");
                    trace!("{:?}", error);
                    std::thread::sleep(delay);
                }
            }
        }
        bail!(
            "Cannot find connected temperature collector for device {}",
            device
        )
    }

    let (mut d, command) = match &args.connection {
        CliConnection::Tcp { address, command } => {
            let socket_addr = address
                .parse()
                .with_context(|| format!("Cannot parse address {}", address))?;
            trace!("Open TCP address {}", socket_addr);
            (
                R4DCB08::new(
                    tokio_modbus::client::sync::tcp::connect(socket_addr)
                        .with_context(|| format!("Cannot open {:?}", socket_addr))?,
                ),
                command,
            )
        }
        CliConnection::Rtu {
            device,
            baud_rate,
            address,
            command,
        } => {
            let address = if command == &CliCommands::QueryAddress {
                if !confirm_only_one_module_connected()? {
                    return Ok(());
                }
                if *address != proto::READ_ADDRESS_BROADCAST_ADDRESS {
                    info!(
                        "Ignore address {:#04x} use broadcast address {:#04x}",
                        address,
                        proto::READ_ADDRESS_BROADCAST_ADDRESS
                    );
                }
                proto::READ_ADDRESS_BROADCAST_ADDRESS
            } else {
                *address
            };
            trace!(
                "Open RTU {} address {:#04x} baud rate {}",
                device,
                address,
                baud_rate
            );
            delay = check_rtu_delay(delay, baud_rate);
            (
                R4DCB08::new(
                    tokio_modbus::client::sync::rtu::connect_slave(
                        &r4dcb08_lib::tokio_serial::serial_port_builder(
                            device,
                            baud_rate.as_u16() as u32,
                        ),
                        tokio_modbus::Slave(address),
                    )
                    .with_context(|| {
                        format!("Cannot open device {} baud rate {}", device, baud_rate)
                    })?,
                ),
                command,
            )
        }
        CliConnection::RtuScan { .. } => unreachable!(),
    };
    d.set_timeout(args.timeout);

    match command {
        CliCommands::Daemon { poll_iterval, mode } => match mode {
            DaemonMode::Stdout => loop {
                print_temperature!(&mut d);
                std::thread::sleep(delay.max(*poll_iterval));
            },
            DaemonMode::Mqtt {
                url,
                username,
                password,
                topic,
                qos,
            } => {
                let mut cli =
                    mqtt::Client::new(url.clone()).with_context(|| "Error creating MQTT client")?;

                // Use 5sec timeouts for sync calls.
                cli.set_timeout(Duration::from_secs(5));

                let mut conn_builder = mqtt::ConnectOptionsBuilder::new();
                let mut conn_builder = conn_builder
                    .keep_alive_interval(Duration::from_secs(20))
                    .clean_session(true);

                if let Some(user_name) = username {
                    conn_builder = conn_builder.user_name(user_name)
                }
                if let Some(password) = password {
                    conn_builder = conn_builder.password(password)
                }
                let conn_ops = conn_builder.finalize();

                // Connect and wait for it to complete or fail.
                // The default connection uses MQTT v3.x
                cli.connect(conn_ops)
                    .with_context(|| "MQTT client unable to connect")?;

                loop {
                    let reply = d.read_temperature()?;
                    trace!("Temperature: {:?}", reply);
                    for (channel, temperature) in reply.iter().enumerate() {
                        let topic = format!("{topic}/{channel}");
                        let msg = mqtt::Message::new(topic, temperature.to_string(), *qos as i32);
                        cli.publish(msg)
                            .with_context(|| "Cannot publish MQTT message")?;
                    }
                    std::thread::sleep(delay.max(*poll_iterval));
                }
            }
        },
        CliCommands::Read => {
            print_temperature!(&mut d);
        }
        CliCommands::ReadCorrection => {
            print_temperature_correction!(&mut d);
        }
        CliCommands::ReadBaudRate => {
            print_baud_rate!(&mut d);
        }
        CliCommands::ReadAutomaticReport => {
            print_automatic_report!(&mut d);
        }
        CliCommands::ReadAll => {
            print_temperature!(&mut d);
            std::thread::sleep(delay);
            print_temperature_correction!(&mut d);
            std::thread::sleep(delay);
            print_baud_rate!(&mut d);
            std::thread::sleep(delay);
            print_automatic_report!(&mut d);
        }
        CliCommands::QueryAddress => {
            let rsp = d
                .read_address()
                .with_context(|| "Cannot read RS485 address")?;
            println!("RS485 address: {:#04x}", rsp);
        }
        CliCommands::SetCorrection { channel, value } => {
            d.set_temperature_correction(*channel, *value)
                .with_context(|| "Cannot set temperature correction")?;
        }
        CliCommands::SetBaudRate { new_baud_rate } => {
            d.set_baud_rate(**new_baud_rate)
                .with_context(|| "Cannot set baud rate")?;
            println!("The baud rate will be updated when the module is powered up again!");
        }
        CliCommands::SetAddress { address } => {
            d.set_address(*address)
                .with_context(|| "Cannot set RS485 address")?;
        }
        CliCommands::SetAutomaticReport {
            report_time: report_in_seconds,
        } => {
            d.set_automatic_report(Duration::from_secs(*report_in_seconds as u64))
                .with_context(|| "Cannot set automatic report")?;
        }
        CliCommands::FactoryReset => {
            println!("\
                Reset to factory settings:\n\
                address={:#04x} | baud rate={} | temperature correction all channels=0.0 | automatic report=0 (disabled)\n\
                \n\
                After this operation, the device will no longer be responsive!\n\
                You must power off and on again to complete the reset.", proto::FACTORY_DEFAULT_ADDRESS, BaudRate::from(*proto::FACTORY_DEFAULT_BAUD_RATE)
            );
            let confirmation = Confirm::new()
                .with_prompt("Do you want to continue?")
                .default(false)
                .show_default(true)
                .interact()?;
            if !confirmation {
                return Ok(());
            }
            print!("Check connection to temperature collector ... ");
            stdout().flush().unwrap();
            match d.read_temperature() {
                Ok(_) => println!("succeeded"),
                Err(error) => {
                    println!("failed");
                    return Err(error.into());
                }
            }
            std::thread::sleep(delay);
            if let Err(error) = d.factory_reset() {
                let ignore_error = if let r4dcb08_lib::tokio_error::Error::ModbusError(
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
            println!(
                "Please disconnect and reconnect the power supply to the temperature collector!"
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtu_delay() {
        for baud_rate in BaudRate::iter() {
            match baud_rate.as_u16() {
                1200 => assert_eq!(baud_rate.minimum_rtu_delay().as_millis(), 32),
                2400 => assert_eq!(baud_rate.minimum_rtu_delay().as_millis(), 16),
                4800 => assert_eq!(baud_rate.minimum_rtu_delay().as_millis(), 8),
                9600 => assert_eq!(baud_rate.minimum_rtu_delay().as_millis(), 4),
                19200 => assert_eq!(baud_rate.minimum_rtu_delay().as_millis(), 2),
                _ => unreachable!(),
            }
        }
    }
}
