use shellexpand;
use serde_json;
use std::borrow::Borrow;

#[cfg(windows)]
const CONFIG_DIR: &'static str = "~/AppData/Roaming/wan";
#[cfg(not(windows))]
const CONFIG_DIR: &'static str = "~/.config/wan";

#[derive(Debug, Default, Deserialize)]
pub struct Config {
  pub url: Option<String>,
}

impl Config {
  pub fn load() -> ::Result<Config> {
    let path = format!("{}/config.json", CONFIG_DIR);
    let path = shellexpand::full(&path)?;
    if !::std::path::PathBuf::from(path.borrow() as &str).is_file() {
      return Ok(Default::default());
    }
    let reader = ::std::fs::OpenOptions::new().read(true).open(path.borrow() as &str)?;
    let config = serde_json::from_reader(reader)?;
    Ok(config)
  }
}
