use std::sync::{Arc, RwLock};
use std::ops::Deref;
use curl::easy::{Easy, List};
use serde_json;
use super::Result;

#[derive(Debug, Serialize)]
pub struct CompileRequest {
  code: String,
  compiler: String,

  #[serde(rename = "runtime-option-raw")]
  #[serde(skip_serializing_if = "String::is_empty")]
  runtime_option_raw: String,
}

#[derive(Debug, Deserialize)]
pub struct CompileResponse {
  pub status: i32,
  pub program_message: Option<String>,
  pub program_output: Option<String>,
  pub compiler_message: Option<String>,
}

impl CompileRequest {
  pub fn new(code: String) -> CompileRequest {
    CompileRequest {
      code: code,
      compiler: String::new(),
      runtime_option_raw: String::new(),
    }
  }

  pub fn compiler(mut self, compiler: &str) -> CompileRequest {
    self.compiler = compiler.to_owned();
    self
  }

  pub fn runtime_option(mut self, options: &[&str]) -> CompileRequest {
    self.runtime_option_raw = options.join("\n");
    self
  }

  pub fn compile_request(self) -> Result<CompileResponse> {
    let chunk = Arc::new(RwLock::new(Vec::new()));

    let mut headers = List::new();
    headers.append("Content-Type: application/json")?;

    let mut easy = Easy::new();
    easy.http_headers(headers)?;
    easy.url("http://melpon.org/wandbox/api/compile.json")?;
    easy.post(true)?;
    easy.post_fields_copy(&serde_json::to_vec(&self)?)?;

    let c = chunk.clone();
    easy.write_function(move |data: &[u8]| {
        c.write().unwrap().extend(data);
        Ok(data.len())
      })?;

    easy.perform()?;

    let response = serde_json::from_slice(chunk.read().unwrap().deref())?;
    Ok(response)
  }
}
