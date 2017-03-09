#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate wan;
extern crate shlex;

use std::env;
use std::fs::File;
use std::io::{self, Read, Write, BufRead, BufReader};

struct App<'a> {
  filename: &'a str,
  args: Vec<&'a str>,
}

impl<'a> App<'a> {
  fn register<'b, 'c: 'b>(app: clap::App<'b, 'c>) -> clap::App<'b, 'c> {
    app.args_from_usage(r#"
      <filename>   'target filename'
      [args...]    'runtime options'
    "#)
  }

  fn from_matches<'b, 'c: 'b>(m: &'b clap::ArgMatches<'c>) -> App<'b> {
    App {
      filename: m.value_of("filename").unwrap(),
      args: m.values_of("args").map(|s| s.collect()).unwrap_or_default(),
    }
  }

  fn run(self) -> wan::Result<i32> {
    let compiler = env::var("WAN_COMPILER").unwrap_or("gcc-head".to_owned());
    let options = env::var("WAN_OPTIONS").ok().unwrap_or_default();
    let compiler_options = env::var("WAN_COMPILER_OPTIONS")
      .ok()
      .and_then(|ref s| shlex::split(s))
      .unwrap_or_default();

    let mut code = String::new();
    let mut f = BufReader::new(File::open(self.filename)?);
    f.read_line(&mut String::new())?;
    f.read_to_string(&mut code)?;

    let parameter = wan::compile::Parameter::new(code, compiler)
      .options(options)
      .compiler_option(compiler_options)
      .runtime_option(&self.args);
    let result = parameter.request()?;
    result.report();

    Ok(result.status())
  }
}

fn main() {
  env_logger::init().unwrap();

  let m = App::register(app_from_crate!()).get_matches();
  let app = App::from_matches(&m);

  match app.run() {
    Ok(code) => std::process::exit(code),
    Err(err) => writeln!(&mut io::stderr(), "failed with: {:?}", err).unwrap(),
  }
}
