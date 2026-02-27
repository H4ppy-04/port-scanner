use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Service {
    pub name: String,
    pub port: u16,
    pub protocol: String,
    pub comment: Option<String>,
}

impl Service {
    pub fn output_text(&self) {
        println!(
            "{},{},{},{}",
            self.port,
            self.protocol,
            self.name,
            self.comment.clone().unwrap_or(String::from("..."))
        );
    }

    pub fn output_csv(&self) {
        println!(
            "{}/{} - {}\t{}",
            self.port,
            self.protocol,
            self.name,
            self.comment.clone().unwrap_or_default()
        );
    }
}
