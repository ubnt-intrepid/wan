extern crate wan;
extern crate env_logger;

use std::io::{self, Write};

fn main() {
  env_logger::init().unwrap();

  let args: Vec<_> = std::env::args().skip(1).collect();
  if args.len() < 2 {
    let _ = writeln!(&mut io::stderr(),
                     "Usage: {} [compiler] [file] [arguments...]",
                     std::env::args().next().unwrap());
    std::process::exit(1);
  }

  let response = wan::compile_request(&args[0], &args[1], &args[2..]).unwrap();
  if let Some(message) = response.program_message {
    println!("{}", message);
  } else {
    println!("{}", response.compiler_message.unwrap());
  }
  std::process::exit(response.status);
}
