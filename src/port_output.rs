use clap::ValueEnum;

#[derive(Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, ValueEnum)]
pub enum PortOutput {
    /// Show only TCP
    #[default]
    Tcp,
    /// Show only UDP
    Udp,
    /// Show both TCP, UDP, and any other protocols
    All,
}
