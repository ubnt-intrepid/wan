#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate wan;
extern crate shlex;

use std::env;
use std::fs::File;
use std::io::{stderr, Read, Write, BufRead, BufReader};

fn run(compiler: &str, filename: &str, arguments: Vec<String>) -> wan::Result<i32> {
  let mut code = String::new();
  let mut f = BufReader::new(File::open(filename)?);
  f.read_line(&mut String::new())?;
  f.read_to_string(&mut code)?;

  let arguments: Vec<&str> = arguments.iter().map(|ref s| s.as_str()).collect();

  let result = wan::Compile::new(code).compiler(compiler)
    .runtime_option(&arguments)
    .request()?;

  result.report();
  Ok(result.status())
}

fn main() {
  env_logger::init().unwrap();

  let m = app_from_crate!()
    .arg_from_usage("<filename> 'target filename'")
    .get_matches();

  let filename = m.value_of("filename").unwrap();

  let compiler = env::var("WAN_COMPILER").unwrap_or("gcc-head".to_owned());
  let arguments: Vec<_> = env::var("WAN_RUNTIME_ARGS")
    .ok()
    .and_then(|ref s| shlex::split(s))
    .unwrap_or_default();

  match run(&compiler, filename, arguments) {
    Ok(code) => std::process::exit(code),
    Err(err) => writeln!(&mut stderr(), "failed with: {:?}", err).unwrap(),
  }
}
