use std::path::PathBuf;

use clap::{Subcommand, value_parser};

use crate::{output_format::OutputFormat, port_output::PortOutput};

#[derive(Subcommand)]
pub enum Commands {
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
