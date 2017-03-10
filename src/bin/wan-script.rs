#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate wan;

use std::io::Write;
use wan::app::{ScriptApp, Register, Run};

fn main() {
  env_logger::init().unwrap();

  let ref matches = app_from_crate!()
    .register::<ScriptApp>()
    .get_matches();
  let app: ScriptApp = matches.into();

  match app.run() {
    Ok(code) => std::process::exit(code),
    Err(err) => writeln!(&mut std::io::stderr(), "failed with: {:?}", err).unwrap(),
  }
}
