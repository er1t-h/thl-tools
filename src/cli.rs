#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Action {
    Extract,
    Pack,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct CliArgs {
    pub action: Action,
    pub source: String,
    pub destination: String,
}
