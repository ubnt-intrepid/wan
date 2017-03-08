use curl::easy::{Easy, List};
use serde_json;
use Result;
use std::io::Write;

#[derive(Debug, Default, Serialize)]
pub struct Compile {
  code: String,
  compiler: String,
  save: bool,

  #[serde(skip_serializing_if = "Vec::is_empty")]
  codes: Vec<CompileCode>,

  #[serde(skip_serializing_if = "String::is_empty")]
  options: String,

  #[serde(skip_serializing_if = "String::is_empty")]
  stdin: String,

  #[serde(rename = "compiler-option-raw")]
  #[serde(skip_serializing_if = "String::is_empty")]
  compiler_option_raw: String,

  #[serde(rename = "runtime-option-raw")]
  #[serde(skip_serializing_if = "String::is_empty")]
  runtime_option_raw: String,
}

#[derive(Debug, Default, Serialize)]
pub struct CompileCode {
  file: String,
  code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompileResult {
  status: i32,
  signal: Option<String>,
  compiler_output: Option<String>,
  compiler_error: Option<String>,
  compiler_message: Option<String>,
  program_output: Option<String>,
  program_error: Option<String>,
  program_message: Option<String>,
  permlink: Option<String>,
  url: Option<String>,
}

impl Compile {
  pub fn new(code: String) -> Self {
    let mut ret = Compile::default();
    ret.code = code;
    ret
  }

  pub fn compiler(mut self, compiler: String) -> Self {
    self.compiler = compiler.into();
    self
  }

  pub fn options(mut self, options: String) -> Self {
    self.options = options.into();
    self
  }

  pub fn code(mut self, code: CompileCode) -> Self {
    self.codes.push(code);
    self
  }

  pub fn codes<I>(mut self, codes: I) -> Self
    where I: IntoIterator<Item = CompileCode>
  {
    self.codes.extend(codes);
    self
  }

  pub fn stdin(mut self, stdin: String) -> Self {
    self.stdin = stdin.into();
    self
  }

  pub fn compiler_option<I, S>(mut self, options: I) -> Self
    where I: IntoIterator<Item = S>,
          S: AsRef<str>
  {
    self.compiler_option_raw = options.into_iter().fold(String::new(), |mut acc, s| {
      if !acc.is_empty() {
        acc.push('\n');
      }
      acc.push_str(s.as_ref());
      acc
    });
    self
  }

  pub fn runtime_option<I, S>(mut self, options: I) -> Self
    where I: IntoIterator<Item = S>,
          S: AsRef<str>
  {
    self.runtime_option_raw = options.into_iter().fold(String::new(), |mut acc, s| {
      if !acc.is_empty() {
        acc.push('\n');
      }
      acc.push_str(s.as_ref());
      acc
    });
    self
  }

  pub fn save(mut self, save: bool) -> Self {
    self.save = save;
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

  pub fn dump(&self) -> Result<()> {
    ::std::io::stdout().write_all(serde_json::to_string_pretty(self)?.as_bytes())?;
    Ok(())
  }
}
