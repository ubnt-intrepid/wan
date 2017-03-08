use curl::easy::{Easy, List};
use serde_json;
use super::Result;

#[derive(Debug, Serialize)]
pub struct Compile {
  code: String,
  compiler: String,

  #[serde(rename = "runtime-option-raw")]
  #[serde(skip_serializing_if = "String::is_empty")]
  runtime_option_raw: String,
}

#[derive(Debug, Deserialize)]
pub struct CompileResult {
  status: i32,
  program_message: Option<String>,
  program_output: Option<String>,
  compiler_message: Option<String>,
}

impl Compile {
  pub fn new(code: String) -> Self {
    Compile {
      code: code,
      compiler: String::new(),
      runtime_option_raw: String::new(),
    }
  }

  pub fn compiler(mut self, compiler: &str) -> Self {
    self.compiler = compiler.to_owned();
    self
  }

  pub fn runtime_option(mut self, options: &[&str]) -> Self {
    self.runtime_option_raw = options.join("\n");
    self
  }

  pub fn request(self) -> Result<CompileResult> {
    let mut headers = List::new();
    headers.append("Content-Type: application/json")?;

    let mut easy = Easy::new();
    easy.http_headers(headers)?;
    easy.url("http://melpon.org/wandbox/api/compile.json")?;
    easy.post(true)?;
    easy.post_fields_copy(&serde_json::to_vec(&self)?)?;

    let mut buf = Vec::new();
    {
      let mut transfer = easy.transfer();
      transfer.write_function(|data: &[u8]| {
          buf.extend_from_slice(data);
          Ok(data.len())
        })?;
      transfer.perform()?;
    }

    let result = serde_json::from_slice(&buf)?;
    Ok(result)
  }
}

impl CompileResult {
  pub fn status(&self) -> i32 {
    self.status
  }

  pub fn report(&self) {
    if let Some(ref message) = self.program_message {
      println!("{}", message);
    } else {
      println!("{}", self.compiler_message.as_ref().unwrap());
    }
  }
}

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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Either<L, R> {
  Left(L),
  Right(R),
}

impl<L, R> Either<L, R> {
  pub fn into_left(self) -> Option<L> {
    match self {
      Either::Left(l) => Some(l),
      Either::Right(_) => None,
    }
  }

  pub fn into_right(self) -> Option<R> {
    match self {
      Either::Left(_) => None,
      Either::Right(r) => Some(r),
    }
  }
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
