#[derive(Debug, Serialize)]
pub struct CompileRequest {
  pub compiler: String,
  pub code: String,

  #[serde(rename = "runtime-option-raw")]
  #[serde(skip_serializing_if = "String::is_empty")]
  pub runtime_option_raw: String,
}

#[derive(Debug, Deserialize)]
pub struct CompileResponse {
  pub status: i32,
  pub program_message: Option<String>,
  pub program_output: Option<String>,
  pub compiler_message: Option<String>,
}
