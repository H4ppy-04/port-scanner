use clap::{Parser, Subcommand, ValueEnum, value_parser};
use csv::Reader;
use directories::ProjectDirs;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const PORT_LIMIT: u16 = 1024;
const STATIC_TIMOUT_MS: u64 = 150;

#[derive(Parser)]
#[command(version, about = "Multithreaded port scanner", arg_required_else_help = true, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Service {
    name: String,
    port: u16,
    protocol: String,
    comment: Option<String>,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, ValueEnum, Default)]
enum Mode {
    /// Run swiftly. This uses multithreading and is much faster.
    #[default]
    Fast,

    /// Crawly slowly
    Slow,
}

#[derive(Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, ValueEnum)]
enum OutputFormat {
    /// Comma separated value format
    Csv,
    /// JavaScript object notation format
    Json,
    /// Plain text format
    #[default]
    Text,
}

#[derive(Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, ValueEnum)]
enum PortOutput {
    /// Show only TCP
    #[default]
    Tcp,
    /// Show only UDP
    Udp,
    /// Show both TCP, UDP, and any other protocols
    All,
}

#[derive(Subcommand)]
enum Commands {
    /// Get service csv file
    ///
    /// If downloading from source, if your /etc/services path is more expansive, run
    /// src/clean_services.py and replace the default services.csv file.
    GetServicePath,

    /// Scan port range
    Scan {
        #[arg()]
        address: String,

        /// Port limit within address to scan
        ///
        /// The default is capped at 1024, however this can be changed up to 25565
        #[arg(short, long, value_parser = value_parser!(u16).range(0..=25565))]
        port: Option<u16>,

        #[arg(value_enum, long)]
        mode: Option<Mode>,

        /// How long (in milliseconds) a port gets scanned for before it's dropped.
        ///
        /// The default value is 150 if none is specified.
        #[arg(long)]
        timeout: Option<u64>,

        /// How to parse the output format.
        ///
        /// The default value is simply plain text.
        #[arg(long, value_enum)]
        format: Option<OutputFormat>,

        /// Port options to show.
        ///
        /// The default value is TCP.
        #[arg(value_enum, long)]
        port_output: Option<PortOutput>,

        /// Output into a file.
        ///
        /// Automatically detects what format the file should write to based on the file extension.
        /// If there is no file extension, it assumes raw text.
        ///
        /// WARNING: If the file exists, it will overwrite!
        #[arg(long)]
        output_file: Option<PathBuf>,
    },
}

/// Establish if a port is open or closed.
fn scan_port(port: u16, address: &str, timeout: Option<u64>) -> bool {
    let socket = format!("{address}:{port}");
    if let Ok(mut addrs) = socket.to_socket_addrs() {
        if let Some(addr) = addrs.next() {
            return TcpStream::connect_timeout(
                &addr,
                Duration::from_millis(timeout.unwrap_or(STATIC_TIMOUT_MS)),
            )
            .is_ok();
        }
    }
    false
}

fn ensure_services_csv() -> std::path::PathBuf {
    let proj_dirs =
        ProjectDirs::from("com", "Org", "PortScanner").expect("Failed to get project directories.");
    let data_dir = proj_dirs.data_dir();
    if !data_dir.exists() {
        fs::create_dir_all(data_dir).expect("Failed to create data directory.");
    }
    let service_path = data_dir.join("services.csv");

    // copy if not exists
    if !service_path.exists() {
        let src_csv = Path::new("src/services.csv");
        if src_csv.exists() {
            fs::copy(src_csv, &service_path)
                .expect("Failed to copy services.csv to data directory.");
        } else {
            panic!("src/services.csv not found in project")
        }
    }
    service_path
}

pub fn main() {
    let cli = Cli::parse();

    let open_ports = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    let service_path = ensure_services_csv();

    match cli.command {
        Some(Commands::GetServicePath) => {
            dbg!(service_path);
        }
        Some(Commands::Scan {
            address,
            port,
            mode,
            timeout,
            format,
            port_output,
            output_file,
        }) => {
            let port_limit = port.unwrap_or(PORT_LIMIT);
            if mode.is_some_and(|m| m == Mode::Fast) || mode.is_none() {
                // perform multithreading
                let spinner = ProgressBar::new_spinner();

                for port in 1..=port_limit {
                    let open_ports = Arc::clone(&open_ports);
                    let address = address.clone();

                    let handle = thread::spawn(move || {
                        let addr = format!("{address}:{port}");
                        if TcpStream::connect_timeout(
                            &addr.parse().unwrap(),
                            Duration::from_millis(timeout.unwrap_or(STATIC_TIMOUT_MS)),
                        )
                        .is_ok()
                        {
                            open_ports.lock().unwrap().push(port);
                        }
                    });
                    handles.push(handle);
                    spinner.set_message(format!("Scanning port {port}/{port_limit}"));
                    spinner.tick();
                }
                for handle in handles {
                    handle.join().unwrap();
                }
                spinner.finish_with_message(format!(
                    "Finished scanning {} ports ({} open)",
                    port_limit,
                    open_ports.lock().unwrap().iter().count()
                ));
            } else {
                for port in 1..=port_limit {
                    let is_open = scan_port(port, &address, timeout);
                    if is_open {
                        println!("{port}: OPEN");
                        open_ports.lock().unwrap().push(port);
                    } else {
                        println!("{port}: Closed");
                    }
                }
            }

            let mut reader = Reader::from_path(service_path).unwrap();
            let mut services: Vec<Service> = Vec::new();
            for i in reader.deserialize() {
                let service: Service = i.unwrap();
                services.push(service);
            }

            let outfile_extension: Option<String> = output_file
                .as_ref()
                .and_then(|fs| fs.extension())
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_string());

            if let Some(fs) = output_file.clone() {
                File::create(fs).unwrap();
            }

            if format == Some(OutputFormat::Csv) {
                let csv_header = "port,protocol,name,description";
                println!("{csv_header}");
                {
                    if let Some(fs) = output_file.clone()
                        && outfile_extension.clone().is_some_and(|ext| &ext == "csv")
                    {
                        let mut file = File::create(fs).unwrap();
                        write!(file, "{csv_header}").unwrap();
                    }
                }
            }

            let mut shown_ports: Vec<u16> = vec![];
            let mut shown_services: Vec<&Service> = Vec::new();
            for port in open_ports.lock().unwrap().iter() {
                for service in services.iter() {
                    if service.port != *port {
                        continue;
                    }
                    let protocol = &service.protocol;
                    let name = &service.name;
                    match port_output {
                        Some(PortOutput::Tcp) if service.protocol != "tcp" => continue,
                        Some(PortOutput::Udp) if service.protocol != "udp" => continue,
                        Some(PortOutput::All) | None => {}
                        _ => {}
                    }

                    if shown_ports.contains(&service.port) {
                        continue;
                    }
                    shown_ports.push(service.port);
                    shown_services.push(service);

                    match format {
                        Some(OutputFormat::Json) => {
                            let json = serde_json::to_string_pretty(&shown_services).unwrap();
                            if let Some(fs) = output_file.clone()
                                && outfile_extension.clone().is_some_and(|ext| &ext == "json")
                            {
                                let mut file = OpenOptions::new()
                                    .append(true)
                                    .create(true)
                                    .open(fs)
                                    .unwrap();
                                write!(file, "{json}").unwrap();
                            }
                            println!("{json}");
                        }
                        Some(OutputFormat::Csv) => {
                            if let Some(description) = &service.comment {
                                println!("{port},{protocol},{name},{description}");
                            } else {
                                println!("{port},{protocol},{name}");
                            }
                        }
                        Some(OutputFormat::Text) | None => {
                            if let Some(description) = &service.comment {
                                println!("{port}/{protocol} - {name} ({description})");
                            } else {
                                println!("{port}/{protocol} - {name}");
                            }
                        }
                    }
                }
            }
        }
        None => {}
    }
}
