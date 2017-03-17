#[allow(unused_imports)]
use serde;
#[cfg(test)]
use serde_json;
use std::collections::HashMap;
use Result;
use util::Either;
use http;

macro_rules! enum_str {
  ($name:ident { $($variant:ident : $value:expr, )* }) => {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum $name {
      $($variant,)*
      Unknown(String),
    }

    impl ::std::fmt::Display for $name {
      fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
          $(
            $name :: $variant => write!(f, $value),
          )*
          $name :: Unknown(ref s) => write!(f, "{}", s),
        }
      }
    }

    impl ::std::str::FromStr for $name {
      type Err = String;
      fn from_str(s: &str) -> ::std::result::Result<$name, Self::Err> {
        match s {
          $(
            $value => Ok($name :: $variant),
          )*
          s => Ok($name :: Unknown(s.to_owned())),
        }
      }
    }

    impl ::serde::Serialize for $name {
      fn serialize<S>(&self, s: S) -> ::std::result::Result<S::Ok, S::Error>
        where S: ::serde::Serializer {
        s.serialize_str(&self.to_string())
      }
    }

    impl ::serde::Deserialize for $name {
      fn deserialize<D>(d: D) -> ::std::result::Result<$name, D::Error>
        where D: ::serde::Deserializer {
        struct Visitor;
        impl ::serde::de::Visitor for Visitor {
          type Value = $name;
          fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            formatter.write_str(concat!("enum ", stringify!($name)))
          }
          fn visit_str<E>(self, s: &str) -> ::std::result::Result<Self::Value, E>
            where E: ::serde::de::Error {
            ::std::str::FromStr::from_str(s).map_err(|e| E::custom(e))
          }
        }
        d.deserialize(Visitor)
      }
    }
  }
}

enum_str!(Language {
  BashScript: "Bash script",
  C: "C",
  Csharp: "C#",
  Cplusplus: "C++",
  CoffeeScript: "CoffeeScript",
  CPP: "CPP",
  D: "D",
  Elixir: "Elixir",
  Erlang: "Erlang",
  Groovy: "Groovy",
  Haskell: "Haskell",
  Java: "Java",
  JavaScript: "JavaScript",
  LazyK: "Lazy K",
  Lisp: "Lisp",
  Lua: "Lua",
  OCaml: "OCaml",
  Pascal: "Pascal",
  Perl: "Perl",
  PHP: "PHP",
  Python: "Python",
  Rill: "Rill",
  Ruby: "Ruby",
  Rust: "Rust",
  Scala: "Scala",
  SQL: "SQL",
  Swift: "Swift",
  VimScript: "Vim script",
});

impl Language {
  pub fn from_extension(ext: &str) -> ::Result<Language> {
    match ext {
      "sh" => Ok(Language::BashScript),
      "c" | "h" => Ok(Language::C),
      "cs" => Ok(Language::Csharp),
      "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "hh" => Ok(Language::Cplusplus),
      "coffee" => Ok(Language::CoffeeScript),
      "d" => Ok(Language::D),
      "ex" | "exs" => Ok(Language::Elixir),
      "erl" => Ok(Language::Erlang),
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

lazy_static!{
  static ref DEFAULT_COMPILERS: HashMap<Language, &'static str> = {
    let mut mapping = HashMap::new();
    mapping.insert(Language::BashScript, "bash");
    mapping.insert(Language::C, "gcc-head-c");
    mapping.insert(Language::Csharp, "mono-head");
    mapping.insert(Language::Cplusplus, "gcc-head");
    mapping.insert(Language::CoffeeScript, "coffeescript-head");
    mapping.insert(Language::CPP, "gcc-head-pp");
    mapping.insert(Language::D, "ldc-head");
    mapping.insert(Language::Elixir, "elixir-head");
    mapping.insert(Language::Erlang, "erlang-head");
    mapping.insert(Language::Groovy, "groovy-head");
    mapping.insert(Language::Haskell, "ghc-head");
    mapping.insert(Language::Java, "openjdk-head");
    mapping.insert(Language::JavaScript, "nodejs-head");
    mapping.insert(Language::LazyK, "lazyk");
    mapping.insert(Language::Lisp, "clisp-2.49");
    mapping.insert(Language::Lua, "lua-5.3.4");
    mapping.insert(Language::OCaml, "ocaml-head");
    mapping.insert(Language::Pascal, "fpc-head");
    mapping.insert(Language::Perl, "perl-head");
    mapping.insert(Language::PHP, "php-head");
    mapping.insert(Language::Python, "cpython-head");
    mapping.insert(Language::Rill, "rill-head");
    mapping.insert(Language::Ruby, "ruby-head");
    mapping.insert(Language::Rust, "rust-head");
    mapping.insert(Language::Scala, "scala-head");
    mapping.insert(Language::SQL, "sqlite-head");
    mapping.insert(Language::Swift, "swift-head");
    mapping.insert(Language::VimScript, "vim-head");
    mapping
  };
}

pub fn get_default_compiler(lang: &Language) -> Option<&'static str> {
  DEFAULT_COMPILERS.get(lang).map(|s| *s)
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
