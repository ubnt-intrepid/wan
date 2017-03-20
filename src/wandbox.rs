use util;
use std::path::Path;
#[allow(unused_imports)]
use serde;
#[cfg(test)]
use serde_json;
use util::Either;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Code {
  file: String,
  code: String,
}

impl Code {
  pub fn new<P: AsRef<Path> + Copy>(path: P) -> Code {
    let file = path.as_ref().file_name().unwrap().to_string_lossy().into_owned();

    let mut f = ::std::fs::File::open(path).unwrap();
    use std::io::Read;
    let mut code = String::new();
    f.read_to_string(&mut code).unwrap();

    Code {
      file: file,
      code: code,
    }
  }
}


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Parameter {
  pub code: String,
  pub compiler: String,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub stdin: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub options: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub codes: Option<Vec<Code>>,

  #[serde(rename = "compiler-option-raw")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub compiler_option_raw: Option<String>,

  #[serde(rename = "runtime-option-raw")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub runtime_option_raw: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub save: Option<bool>,

  #[serde(rename = "created-at")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub created_at: Option<String>,
}

impl Parameter {
  pub fn new<S1: Into<String>, S2: Into<String>>(code: S1, compiler: S2) -> Self {
    let mut ret = Self::default();
    ret.code = code.into();
    ret.compiler = compiler.into();
    ret
  }

  pub fn options<S: Into<String>>(&mut self, options: S) -> &mut Self {
    self.options = Some(options.into());
    self
  }

  pub fn code<S>(&mut self, file: S) -> &mut Self
    where S: AsRef<str>
  {
    self.codes(vec![file])
  }

  pub fn codes<I, S>(&mut self, files: I) -> &mut Self
    where I: IntoIterator<Item = S>,
          S: AsRef<str>
  {
    if self.codes.is_none() {
      self.codes = Some(Vec::new());
    }
    self.codes.as_mut().unwrap().extend(files.into_iter().map(|s| Code::new(s.as_ref())));
    self
  }

  pub fn stdin<S: Into<String>>(&mut self, stdin: S) -> &mut Self {
    self.stdin = Some(stdin.into());
    self
  }

  pub fn compiler_option<I, S>(&mut self, options: I) -> &mut Self
    where I: IntoIterator<Item = S>,
          S: AsRef<str>
  {
    self.compiler_option_raw = Some(util::str_join(options, "\n"));
    self
  }

  pub fn runtime_option<I, S>(&mut self, options: I) -> &mut Self
    where I: IntoIterator<Item = S>,
          S: AsRef<str>
  {
    self.runtime_option_raw = Some(util::str_join(options, "\n"));
    self
  }

  pub fn save_permlink(&mut self, save: bool) -> &mut Self {
    self.save = Some(save);
    self
  }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
  pub status: i32,
  pub signal: Option<String>,
  pub compiler_output: Option<String>,
  pub compiler_error: Option<String>,
  pub compiler_message: Option<String>,
  pub program_output: Option<String>,
  pub program_error: Option<String>,
  pub program_message: Option<String>,
  pub permlink: Option<String>,
  pub url: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CompilerInfo {
  pub name: String,
  pub version: String,
  pub language: String,

  #[serde(rename = "display-name")]
  pub display_name: String,

  #[serde(rename = "compiler-option-raw")]
  pub compiler_option_raw: bool,

  #[serde(rename = "runtime-option-raw")]
  pub runtime_option_raw: bool,

  #[serde(rename = "display-compile-command")]
  pub display_compile_command: String,

  pub switches: Vec<Either<CompilerSwitch, CompilerSwitchMultiOptions>>,
}

impl ::std::fmt::Display for CompilerInfo {
  fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    write!(f, "{} {}", self.name, self.language)
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CompilerSwitch {
  pub default: bool,
  pub name: String,
  #[serde(rename = "display-name")]
  pub display_name: String,
  #[serde(rename = "display-flags")]
  pub display_flags: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CompilerSwitchMultiOptions {
  pub default: String,
  pub options: Vec<CompilerOption>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CompilerOption {
  pub name: String,
  #[serde(rename = "display-name")]
  pub display_name: String,
  #[serde(rename = "display-flags")]
  pub display_flags: String,
}

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
