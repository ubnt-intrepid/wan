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
  #[wan(value="Bash script", compiler="bash", ext="sh")]
  BashScript,
  #[wan(compiler="gcc-head-c", ext="c,h")]
  C,
  #[wan(value="C#", compiler="mono-head", ext="cs")]
  Csharp,
  #[wan(value="C++", compiler="gcc-head", ext="cpp,cxx,cc,hpp,hxx,hh")]
  Cplusplus,
  #[wan(ext="coffee")]
  CoffeeScript,
  #[wan(compiler="gcc-head-pp")]
  CPP,
  #[wan(compiler="ldc-head", ext="d")]
  D,
  #[wan(ext="ex,exs")]
  Elixir,
  #[wan(ext="erl")]
  Erlang,
  #[wan(ext="go")]
  Go,
  #[wan(ext="groovy")]
  Groovy,
  #[wan(compiler="ghc-head", ext="hs")]
  Haskell,
  #[wan(compiler="openjdk-head", ext="java")]
  Java,
  #[wan(compiler="nodejs-head", ext="js")]
  JavaScript,
  #[wan(value="Lazy K", compiler="lazyk", ext="lazy")]
  LazyK,
  #[wan(compiler="clisp-2.49", ext="lisp")]
  Lisp,
  #[wan(compiler="lua-5.3.4", ext="lua")]
  Lua,
  #[wan(ext="ml")]
  OCaml,
  #[wan(compiler="fpc-head", ext="pas")]
  Pascal,
  #[wan(ext="pl")]
  Perl,
  #[wan(ext="php")]
  PHP,
  #[wan(compiler="cpython-head", ext="py")]
  Python,
  #[wan(ext="rill")]
  Rill,
  #[wan(ext="rb")]
  Ruby,
  #[wan(ext="rs")]
  Rust,
  #[wan(ext="scala")]
  Scala,
  #[wan(compiler="sqlite-head", ext="sql")]
  SQL,
  #[wan(ext="swift")]
  Swift,
  #[wan(value="Vim script", compiler="vim-head", ext="vim")]
  VimScript,
  #[wan(ignore)]
  Unknown(String),
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
