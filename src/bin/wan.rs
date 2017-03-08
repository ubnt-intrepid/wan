#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate wan;

use std::fs::File;
use std::io::{stdin, stderr, Read, Write, BufRead, BufReader};

fn run(compiler: &str, filename: &str, arguments: Vec<&str>) -> wan::Result<i32> {
  let mut code = String::new();
  if filename != "-" {
    let mut f = BufReader::new(File::open(filename)?);
    f.read_line(&mut String::new())?;
    f.read_to_string(&mut code)?;
  } else {
    stdin().read_to_string(&mut code)?;
  }

  let result = wan::Compile::new(code).compiler(compiler)
    .runtime_option(&arguments)
    .request()?;

  result.report();
  Ok(result.status())
}

fn main() {
  let m = app_from_crate!()
    .setting(clap::AppSettings::VersionlessSubcommands)
    .setting(clap::AppSettings::SubcommandRequiredElseHelp)
    .subcommand(clap::SubCommand::with_name("list").about("List compiler information"))
    .subcommand(clap::SubCommand::with_name("run")
      .about("Post a code to wandbox and get a result")
      .arg_from_usage("<compiler>       'compiler name'")
      .arg_from_usage("<filename>       'target filename'")
      .arg_from_usage("[<arguments>...] 'supplemental arguments to pass compiled binary'"))
    .get_matches();

  match m.subcommand() {
    ("list", _) => {
      for info in wan::get_compiler_info().unwrap() {
        println!("{}, \"{}\"", info.name(), info.display_compile_command());
      }
    }
    ("run", Some(m)) => {
      let compiler = m.value_of("compiler").unwrap();
      let filename = m.value_of("filename").unwrap();
      let arguments: Vec<_> = m.values_of("arguments").map(|v| v.collect()).unwrap_or_default();

      match run(compiler, filename, arguments) {
        Ok(code) => std::process::exit(code),
        Err(err) => writeln!(&mut stderr(), "failed with: {:?}", err).unwrap(),
      }
    }
    _ => unreachable!(),
  }
}
