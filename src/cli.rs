use std::{borrow::Cow, path::PathBuf};

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Action {
    Extract {
        source: String,
        destination: String,
    },
    Pack {
        source: String,
        destination: String,
    },
    Translate {
        source: String,
        destination: String,
    },
    ReadLines {
        source: PathBuf,
        #[arg(short, long, default_value_t = Cow::Borrowed("> "))]
        prefix: Cow<'static, str>,
    },
    EditTranslate {
        source: PathBuf,
    },
}

#[derive(Debug, Clone, clap::Parser)]
pub struct CliArgs {
    #[command(subcommand)]
    pub action: Action,
}
