use clap::ValueEnum;

#[derive(Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, ValueEnum)]
pub enum OutputFormat {
    /// Comma separated value format
    Csv,
    /// JavaScript object notation format
    Json,
    /// Plain text format
    #[default]
    Text,
}
