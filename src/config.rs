use std::{ffi::OsStr, fs};

use color_eyre::eyre::{bail, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub rules: Vec<FollowRule>,
}

#[derive(Clone, Deserialize)]
pub struct FollowRule {
    pub input: String,
    pub follows: String,
    pub exclude: Option<Vec<String>>,
}

pub fn get(config_path: &OsStr) -> Result<Config> {
    let config_content = fs::read_to_string(config_path)?;
    let parse_result = toml::from_str::<Config>(&config_content);
    match parse_result {
        Ok(config) => Ok(config),
        Err(report) => bail!("{}", report),
    }
}
