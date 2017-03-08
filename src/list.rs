use Result;
use util::Either;
use http;

#[derive(Debug, Deserialize)]
pub struct CompilerInfo {
  name: String,
  version: String,
  language: String,

  #[serde(rename = "display-name")]
  display_name: String,

  #[serde(rename = "compiler-option-raw")]
  compiler_option_raw: bool,

  #[serde(rename = "runtime-option-raw")]
  runtime_option_raw: bool,

  #[serde(rename = "display-compile-command")]
  display_compile_command: String,

  switches: Vec<Either<CompilerSwitch, CompilerSwitchMultiOptions>>,
}

impl CompilerInfo {
  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn display_compile_command(&self) -> &str {
    &self.display_compile_command
  }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CompilerSwitch {
  default: bool,
  name: String,
  #[serde(rename = "display-name")]
  display_name: String,
  #[serde(rename = "display-flags")]
  display_flags: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CompilerSwitchMultiOptions {
  default: String,
  options: Vec<CompilerOption>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CompilerOption {
  name: String,
  #[serde(rename = "display-name")]
  display_name: String,
  #[serde(rename = "display-flags")]
  display_flags: String,
}

pub fn get_compiler_info() -> Result<Vec<CompilerInfo>> {
  http::get_json("http://melpon.org/wandbox/api/list.json", &[])
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json;

  #[test]
  fn test_compiler_info() {
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/list.json"));
    let dst: Vec<CompilerInfo> = serde_json::from_str(src).unwrap();
    println!("{:?}", dst);
  }

  #[test]
  fn test_compiler_switch() {
    let src = r#"{
    "default":true,
    "name":"sprout",
    "display-flags":"-I/usr/local/sprout",
    "display-name":"Sprout"
  }"#;
    let dst = serde_json::from_str::<Either<CompilerSwitch, CompilerSwitchMultiOptions>>(src)
      .unwrap();

    let dst = dst.into_left().expect("invalid type");
    assert_eq!(dst.default, true);
    assert_eq!(dst.name, "sprout");
    assert_eq!(dst.display_name, "Sprout");
    assert_eq!(dst.display_flags, "-I/usr/local/sprout");
  }

  #[test]
  fn test_compiler_switch_multi() {
    let src = r#"{
    "default":"boost-1.55",
    "options":[{
      "name":"boost-nothing",
      "display-flags":"",
      "display-name":"Don't Use Boost"
    },{
      "name":"boost-1.47",
      "display-flags":"-I/usr/local/boost-1.47.0/include",
      "display-name":"Boost 1.47.0"
    }]
  }"#;
    let dst = serde_json::from_str::<Either<CompilerSwitch, CompilerSwitchMultiOptions>>(src)
      .unwrap();
    let dst = dst.into_right().unwrap();

    assert_eq!(dst.default, "boost-1.55");
    assert_eq!(dst.options,
               [CompilerOption {
                  name: "boost-nothing".to_owned(),
                  display_name: "Don't Use Boost".to_owned(),
                  display_flags: "".to_owned(),
                },
                CompilerOption {
                  name: "boost-1.47".to_owned(),
                  display_name: "Boost 1.47.0".to_owned(),
                  display_flags: "-I/usr/local/boost-1.47.0/include".to_owned(),
                }]);
  }
}
