#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate wan;
extern crate shlex;

use std::env;
use std::fs::File;
use std::io::{self, Read, Write, BufRead, BufReader};

fn run(filename: &str) -> wan::Result<i32> {
  let mut code = String::new();
  let mut f = BufReader::new(File::open(filename)?);
  f.read_line(&mut String::new())?;
  f.read_to_string(&mut code)?;

  let compiler = env::var("WAN_COMPILER").unwrap_or("gcc-head".to_owned());

  let options = env::var("WAN_OPTIONS").ok().unwrap_or_default();

  let compiler_options = env::var("WAN_COMPILER_OPTIONS")
    .ok()
    .and_then(|ref s| shlex::split(s))
    .unwrap_or_default();

  let runtime_options = env::var("WAN_RUNTIME_OPTIONS")
    .ok()
    .and_then(|ref s| shlex::split(s))
    .unwrap_or_default();

  let result = wan::Compile::new(code).compiler(compiler)
    .options(options)
    .compiler_option(compiler_options)
    .runtime_option(runtime_options)
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
  match run(filename) {
    Ok(code) => std::process::exit(code),
    Err(err) => writeln!(&mut io::stderr(), "failed with: {:?}", err).unwrap(),
  }
}
