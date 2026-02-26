use clap::{Parser, Subcommand, ValueEnum};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const PORT_LIMIT: u16 = 1024;
const PORT_FAST_MS: u64 = 50;
const PORT_SLOW_MS: u64 = 250;

#[derive(Parser)]
#[command(version, about, arg_required_else_help = true, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, ValueEnum)]
enum Mode {
    /// Run swiftly
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

        #[arg(value_enum)]
        mode: Mode,
    },
}

/// Establish if a port is open or closed.
fn scan_port(port: u16, address: &str, mode: &Mode) -> bool {
    let socket = format!("{address}:{port}");
    let timeout = {
        match mode {
            Mode::Fast => PORT_FAST_MS,
            Mode::Slow => PORT_SLOW_MS,
        }
    };
    if let Ok(mut addrs) = socket.to_socket_addrs() {
        if let Some(addr) = addrs.next() {
            return TcpStream::connect_timeout(&addr, Duration::from_millis(timeout)).is_ok();
        }
    }
    false
}

pub fn main() {
    let cli = Cli::parse();

    let mut open_ports: Vec<u16> = Vec::new();

    match &cli.command {
        Some(Commands::Scan { mode, address }) => {
            for i in 1..PORT_LIMIT {
                let is_open = scan_port(i, address, mode);
                if is_open {
                    println!("{i}: OPEN");
                    open_ports.push(i);
                } else {
                    println!("{i}: Closed");
                }
            }
        }
        None => {}
    }

    for port in open_ports {
        println!("{port} ..... OPEN")
    }
}
