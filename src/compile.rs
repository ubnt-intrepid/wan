use http;
use util;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Parameter {
  pub code: String,
  pub compiler: String,
  pub stdin: Option<String>,
  pub options: Option<String>,

  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub codes: Vec<Code>,

  #[serde(rename = "compiler-option-raw")]
  pub compiler_option_raw: Option<String>,

  #[serde(rename = "runtime-option-raw")]
  pub runtime_option_raw: Option<String>,

  pub save: Option<bool>,

  #[serde(rename = "created-at")]
  pub created_at: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Code {
  file: String,
  code: String,
}

impl Parameter {
  pub fn new<S1: Into<String>, S2: Into<String>>(code: S1, compiler: S2) -> Self {
    let mut ret = Self::default();
    ret.code = code.into();
    ret.compiler = compiler.into();
    ret
  }

  pub fn options<S: Into<String>>(mut self, options: S) -> Self {
    self.options = Some(options.into());
    self
  }

  pub fn code(mut self, code: Code) -> Self {
    self.codes.push(code);
    self
  }

  pub fn codes<I>(mut self, codes: I) -> Self
    where I: IntoIterator<Item = Code>
  {
    self.codes.extend(codes);
    self
  }

  pub fn stdin<S: Into<String>>(mut self, stdin: S) -> Self {
    self.stdin = Some(stdin.into());
    self
  }

  pub fn compiler_option<I, S>(mut self, options: I) -> Self
    where I: IntoIterator<Item = S>,
          S: AsRef<str>
  {
    self.compiler_option_raw = Some(util::str_join(options, "\n"));
    self
  }

  pub fn runtime_option<I, S>(mut self, options: I) -> Self
    where I: IntoIterator<Item = S>,
          S: AsRef<str>
  {
    self.runtime_option_raw = Some(util::str_join(options, "\n"));
    self
  }

  pub fn save(mut self, save: bool) -> Self {
    self.save = Some(save);
    self
  }

  pub fn request(self) -> ::Result<Result> {
    http::post("http://melpon.org/wandbox/api/compile.json", self)
  }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Result {
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

impl Result {
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
