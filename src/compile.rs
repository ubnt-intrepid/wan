use curl::easy::{Easy, List};
use serde_json;
use Result;

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
