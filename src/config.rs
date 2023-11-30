use clap::Parser;
use clap_serde_derive::ClapSerde;
use serde::{Deserialize, Serialize};
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

#[derive(ClapSerde, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long = "config", short, help = "The config file to parse")]
    config_path: Option<PathBuf>,

    #[command(flatten)]
    pub config: <Config as ClapSerde>::Opt,
}

#[derive(ClapSerde)]
pub struct Config {
    #[default(8080)]
    #[arg(long, short, help = "The port to run on")]
    pub port: u16,

    #[arg(long = "session", short, help = "The session file to open")]
    pub session_file: Option<PathBuf>,
}

impl Config {
    pub fn retrieve() -> Self {
        let mut args = Args::parse();

        let config_str = args
            .config_path
            .or_else(Config::default_path)
            .and_then(|path| read_to_string(path).ok());

        match config_str {
            None => Config::from(&mut args.config),
            Some(config_str) => toml::from_str::<<Config as ClapSerde>::Opt>(&config_str)
                .map(|config| Config::from(config).merge(&mut args.config))
                .unwrap_or_else(|e| panic!("Error in configuration file: {e}")),
        }
    }

    pub fn default_path() -> Option<PathBuf> {
        dirs_next::config_dir().map(|mut path| {
            path.push("prxs");
            path.push("config.toml");
            path
        })
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Session {
    pub filter: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionSaveError {
    #[error("Couldn't serialize data: {0}")]
    Toml(#[from] toml::ser::Error),
    #[error("Couldn't save to file: {0}")]
    IO(#[from] std::io::Error),
}

impl Session {
    pub fn save(&self, file: impl AsRef<Path>) -> Result<(), SessionSaveError> {
        let toml_str = toml::to_string(&self)?;
        Ok(std::fs::write(file, toml_str)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SessionRestoreError {
    #[error("Couldn't read file: {0}")]
    IO(#[from] std::io::Error),
    #[error("Couldn't deserialize to struct: {0}")]
    Toml(#[from] toml::de::Error),
}

impl Session {
    pub fn restore(file: impl AsRef<Path>) -> Result<Self, SessionRestoreError> {
        let toml_str = std::fs::read_to_string(file)?;
        Ok(toml::from_str(&toml_str)?)
    }
}
