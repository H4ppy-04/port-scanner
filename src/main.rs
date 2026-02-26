use clap::{Parser, Subcommand, ValueEnum, value_parser};
use csv::Reader;
use serde::Deserialize;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const PORT_LIMIT: u16 = 1024;
const STATIC_TIMOUT_MS: u64 = 150;
const SERVICES_FILE: &str = "services.csv";

#[derive(Parser)]
#[command(version, about, arg_required_else_help = true, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Deserialize)]
struct Service {
    name: String,
    port: u16,
    protocol: String,
    comment: Option<String>,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, ValueEnum)]
enum Mode {
    /// Run swiftly. This uses multithreading and is much faster.
    Fast,

    /// Crawly slowly
    Slow,
}

#[derive(Subcommand)]
enum Commands {
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
        mode: Mode,

        /// How long (in milliseconds) a port gets scanned for before it's dropped.
        ///
        /// The default value is 150 if none is specified.
        #[arg(long)]
        timeout: Option<u64>,
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

pub fn main() {
    let cli = Cli::parse();

    let open_ports = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    match cli.command {
        Some(Commands::Scan {
            address,
            port,
            mode,
            timeout,
        }) => {
            let port_limit = port.unwrap_or(PORT_LIMIT);
            if mode == Mode::Fast {
                // perform multithreading
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
                }
                for handle in handles {
                    handle.join().unwrap();
                }
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
        }
        None => {}
    }

    let mut reader = Reader::from_path(SERVICES_FILE).unwrap();
    let mut services: Vec<Service> = Vec::new();
    for i in reader.deserialize() {
        let service: Service = i.unwrap();
        services.push(service);
    }

    for port in open_ports.lock().unwrap().iter() {
        for service in services.iter() {
            if service.port == *port {
                let protocol = &service.protocol;
                let name = &service.name;
                if let Some(description) = &service.comment {
                    println!("{port}/{protocol} - {name} ({description})");
                } else {
                    println!("{port}/{protocol} - {name}");
                }
            }
        }
    }
}
