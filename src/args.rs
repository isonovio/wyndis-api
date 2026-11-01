use std::path::PathBuf;

use clap::Parser;

use crate::APP;

#[derive(Parser)]
#[command(version, about = APP)]
pub struct Args {
    #[arg(default_value = "config.toml", value_name = "CONFIG")]
    pub config: PathBuf,
}

#[cfg(test)]
#[path = "tests/args/tests.rs"]
mod tests;
