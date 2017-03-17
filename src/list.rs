#[allow(unused_imports)]
use serde;
#[cfg(test)]
use serde_json;
use Result;
use util::Either;
use http;


pub trait FromExtension: Sized {
  type Err;
  fn from_extension(ext: &str) -> ::std::result::Result<Self, Self::Err>;
}

pub trait GetDefaultCompiler {
  fn get_default_compiler(&self) -> Option<&'static str>;
}


#[derive(Debug, Clone, PartialEq, WanLanguageList)]
pub enum Language {
  #[wan(value="Bash script", compiler="bash")]
  BashScript,
  #[wan(compiler="gcc-head-c")]
  C,
  #[wan(value="C#", compiler="mono-head")]
  Csharp,
  #[wan(value="C++", compiler="gcc-head")]
  Cplusplus,
  CoffeeScript,
  #[wan(compiler="gcc-head-pp")]
  CPP,
  #[wan(compiler="ldc-head")]
  D,
  Elixir,
  Erlang,
  Go,
  Groovy,
  #[wan(compiler="ghc-head")]
  Haskell,
  #[wan(compiler="openjdk-head")]
  Java,
  #[wan(compiler="nodejs-head")]
  JavaScript,
  #[wan(value="Lazy K", compiler="lazyk")]
  LazyK,
  #[wan(compiler="clisp-2.49")]
  Lisp,
  #[wan(compiler="lua-5.3.4")]
  Lua,
  OCaml,
  #[wan(compiler="fpc-head")]
  Pascal,
  Perl,
  PHP,
  #[wan(compiler="cpython-head")]
  Python,
  Rill,
  Ruby,
  Rust,
  Scala,
  #[wan(compiler="sqlite-head")]
  SQL,
  Swift,
  #[wan(value="Vim script", compiler="vim-head")]
  VimScript,
  
  #[wan(ignore)]
  Unknown(String),
}

impl FromExtension for Language {
  type Err = ::Error;
  fn from_extension(ext: &str) -> Result<Self> {
    match ext {
      "sh" => Ok(Language::BashScript),
      "c" | "h" => Ok(Language::C),
      "cs" => Ok(Language::Csharp),
      "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "hh" => Ok(Language::Cplusplus),
      "coffee" => Ok(Language::CoffeeScript),
      "d" => Ok(Language::D),
      "ex" | "exs" => Ok(Language::Elixir),
      "erl" => Ok(Language::Erlang),
      "go" => Ok(Language::Go),
      "groovy" => Ok(Language::Groovy),
      "hs" => Ok(Language::Haskell),
      "java" => Ok(Language::Java),
      "js" => Ok(Language::JavaScript),
      "lazy" => Ok(Language::LazyK),
      "lisp" => Ok(Language::Lisp),
      "lua" => Ok(Language::Lua),
      "ml" => Ok(Language::OCaml),
      "pas" => Ok(Language::Pascal),
      "pl" => Ok(Language::Perl),
      "php" => Ok(Language::PHP),
      "py" => Ok(Language::Python),
      "rill" => Ok(Language::Rill),
      "rb" => Ok(Language::Ruby),
      "rs" => Ok(Language::Rust),
      "scala" => Ok(Language::Scala),
      "sql" => Ok(Language::SQL),
      "swift" => Ok(Language::Swift),
      "vim" => Ok(Language::VimScript),
      ext => Err(format!("Failed to guess filetype: '{}' is unknown extension", ext).into()),
    }
  }
}


#[test]
fn test_extension() {
  assert_eq!(Language::from_extension("rs").unwrap(), Language::Rust);
  assert!(Language::from_extension("hoge").is_err());
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CompilerInfo {
  pub name: String,
  pub version: String,
  pub language: Language,

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

pub fn get_compiler_info() -> Result<Vec<CompilerInfo>> {
  http::get_json("http://melpon.org/wandbox/api/list.json", &[])
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
