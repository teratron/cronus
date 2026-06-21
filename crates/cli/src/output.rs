use clap::ValueEnum;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

pub struct Context {
    pub format: OutputFormat,
}

impl Context {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn is_json(&self) -> bool {
        self.format == OutputFormat::Json
    }
}
