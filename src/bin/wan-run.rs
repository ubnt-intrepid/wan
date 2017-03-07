#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate wan;

use std::io::{Read, BufRead};

fn main() {
  env_logger::init().unwrap();

  let m = app_from_crate!()
    .arg_from_usage("<compiler>       'compiler name'")
    .arg_from_usage("<filename>       'target filename'")
    .arg_from_usage("[<arguments>...] 'supplemental arguments to pass compiled binary'")
    .get_matches();
  let compiler = m.value_of("compiler").unwrap();
  let filename = m.value_of("filename").unwrap();
  let arguments: Vec<_> = m.values_of("arguments").map(|v| v.collect()).unwrap_or_default();

  let mut code = String::new();
  if filename != "-" {
    let mut f = std::io::BufReader::new(std::fs::File::open(filename).unwrap());
    f.read_line(&mut String::new()).unwrap();
    f.read_to_string(&mut code).unwrap();
  } else {
    std::io::stdin().read_to_string(&mut code).unwrap();
  }

  let response = wan::compile_request(code, compiler, &arguments).unwrap();
  if let Some(message) = response.program_message {
    println!("{}", message);
  } else {
    println!("{}", response.compiler_message.unwrap());
  }

  std::process::exit(response.status);
}
