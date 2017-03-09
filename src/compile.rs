use http;
use util;
use std::path::Path;

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
    if self.codes.is_none() {
      self.codes = Some(Vec::new());
    }
    self.codes.as_mut().unwrap().push(code);
    self
  }

  pub fn codes<I>(mut self, codes: I) -> Self
    where I: IntoIterator<Item = Code>
  {
    if self.codes.is_none() {
      self.codes = Some(Vec::new());
    }
    self.codes.as_mut().unwrap().extend(codes);
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
