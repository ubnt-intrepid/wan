use std::collections::HashMap;

lazy_static!{
  static ref LANGUAGES: HashMap<&'static str, (&'static str, Vec<&'static str>)> = {
    make_languages()
  };
}

fn make_languages() -> HashMap<&'static str, (&'static str, Vec<&'static str>)> {
  let mut mappings = HashMap::new();
  mappings.insert("Bash script", ("bash", vec!["sh", "bash"]));
  mappings.insert("C", ("gcc-head-c", vec!["c", "h"]));
  mappings.insert("C#", ("mono-head", vec!["cs"]));
  mappings.insert("C++", ("gcc-head", vec!["cpp,cxx,cc,hpp,hxx,hh"]));
  mappings.insert("CoffeeScript", ("coffeescript-head", vec!["coffee"]));
  mappings.insert("CPP", ("gcc-head-pp", vec![]));
  mappings.insert("D", ("ldc-head", vec!["d"]));
  mappings.insert("Elixir", ("elixir-head", vec!["ex", "exs"]));
  mappings.insert("Erlang", ("erlang-head", vec!["erl"]));
  mappings.insert("Go", ("go-head", vec!["go"]));
  mappings.insert("Groovy", ("groovy-head", vec!["groovy"]));
  mappings.insert("Haskell", ("ghc-head", vec!["hs"]));
  mappings.insert("Java", ("openjdk-head", vec!["java"]));
  mappings.insert("JavaScript", ("nodejs-head", vec!["js"]));
  mappings.insert("Lazy K", ("lazyk", vec!["lazy"]));
  mappings.insert("Lisp", ("clisp-2.49", vec!["lisp"]));
  mappings.insert("Lua", ("lua-5.3.4", vec!["lua"]));
  mappings.insert("OCaml", ("ocaml-head", vec!["ml"]));
  mappings.insert("Pascal", ("fpc-head", vec!["pas"]));
  mappings.insert("Perl", ("perl-head", vec!["pl"]));
  mappings.insert("PHP", ("php-head", vec!["php"]));
  mappings.insert("Pony", ("pony-head", vec!["pony"]));
  mappings.insert("Python", ("cpython-head", vec!["py"]));
  mappings.insert("Rill", ("rill-head", vec!["rill"]));
  mappings.insert("Ruby", ("ruby-head", vec!["rb"]));
  mappings.insert("Rust", ("rust-head", vec!["rs"]));
  mappings.insert("Scala", ("scala-head", vec!["scala"]));
  mappings.insert("SQL", ("sqlite-head", vec!["sql"]));
  mappings.insert("Swift", ("swift-head", vec!["swift"]));
  mappings.insert("Vim script", ("vim-head", vec!["vim"]));
  mappings
}

pub fn get_compiler_from_ext(ext: &str) -> Option<&'static str> {
  LANGUAGES.iter()
    .find(|&(_, val)| val.1.contains(&ext))
    .map(|(_, val)| val.0)
}
