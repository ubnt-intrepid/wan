use curl::easy::{Easy, List};
use serde;
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
  switches: Vec<CompilerSwitch>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CompilerSwitch {
  #[serde(deserialize_with = "bool_or_string")]
  default: BoolOrString,

  #[serde(deserialize_with = "one_or_more")]
  options: Vec<CompilerOption>, // array of CompilerOption or CompilerOption
}

#[derive(Debug, PartialEq)]
pub enum BoolOrString {
  Bool(bool),
  Str(String),
}

fn bool_or_string<D>(d: D) -> ::std::result::Result<BoolOrString, D::Error>
  where D: serde::Deserializer
{
  struct BoolOrStringVisitor;
  impl serde::de::Visitor for BoolOrStringVisitor {
    type Value = BoolOrString;

    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
      formatter.write_str("bool or string")
    }

    fn visit_bool<E>(self, value: bool) -> ::std::result::Result<BoolOrString, E>
      where E: serde::de::Error
    {
      Ok(BoolOrString::Bool(value))
    }

    fn visit_str<E>(self, value: &str) -> ::std::result::Result<BoolOrString, E>
      where E: serde::de::Error
    {
      Ok(BoolOrString::Str(value.to_owned()))
    }
  }

  d.deserialize(BoolOrStringVisitor)
}

fn one_or_more<T, D>(d: D) -> ::std::result::Result<Vec<T>, D::Error>
  where T: serde::Deserialize,
        D: serde::Deserializer
{
  struct OneOrMoreVisitor<T>(::std::marker::PhantomData<T>);

  impl<T> serde::de::Visitor for OneOrMoreVisitor<T>
    where T: serde::de::Deserialize
  {
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
      formatter.write_str("one or more")
    }

    fn visit_seq<V>(self, visitor: V) -> ::std::result::Result<Self::Value, V::Error>
      where V: serde::de::SeqVisitor
    {
      serde::Deserialize::deserialize(serde::de::value::SeqVisitorDeserializer::new(visitor))
    }

    fn visit_map<M>(self, visitor: M) -> ::std::result::Result<Self::Value, M::Error>
      where M: serde::de::MapVisitor
    {
      let value =
        serde::Deserialize::deserialize(serde::de::value::MapVisitorDeserializer::new(visitor))?;
      Ok(vec![value])
    }
  }

  d.deserialize(OneOrMoreVisitor(::std::marker::PhantomData))
}


#[derive(Debug, Deserialize, PartialEq)]
pub struct CompilerOption {
  name: String,
  #[serde(rename = "display-name")]
  display_name: String,
  #[serde(rename = "display-flags")]
  display_flags: String,
}


#[test]
fn test_compiler_switch_1() {
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
  let dst = serde_json::from_str::<CompilerSwitch>(src).unwrap();

  assert_eq!(dst.default, BoolOrString::Str("boost-1.55".to_owned()));
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

#[test]
fn test_compiler_switch_2() {
  let src = r#"{
    "default":true,
    "name":"sprout",
    "display-flags":"-I/usr/local/sprout",
    "display-name":"Sprout"
  }"#;
  let dst = serde_json::from_str::<CompilerSwitch>(src).unwrap();

  assert_eq!(dst.default, BoolOrString::Bool(true));
  assert_eq!(dst.options,
             [CompilerOption {
                name: "sprout".to_owned(),
                display_name: "Sprout".to_owned(),
                display_flags: "-I/usr/local/sprout".to_owned(),
              }]);
}
