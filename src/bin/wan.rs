#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate wan;
extern crate shlex;

use std::fs::File;
use std::io::{self, Read, Write, BufRead, BufReader};

fn run(compiler: &str,
       filename: &str,
       options: &str,
       compiler_args: Vec<String>,
       runtime_args: Vec<String>,
       permlink: bool)
       -> wan::Result<i32> {
  let mut code = String::new();
  if filename != "-" {
    let mut f = BufReader::new(File::open(filename)?);
    f.read_line(&mut String::new())?;
    f.read_to_string(&mut code)?;
  } else {
    io::stdin().read_to_string(&mut code)?;
  }

  let result = wan::Compile::new(code).compiler(compiler.to_owned())
    .options(options.to_owned())
    .compiler_option(compiler_args)
    .runtime_option(runtime_args)
    .save(permlink)
    .request()?;

  result.dump()?;
  Ok(result.status())
}

fn main() {
  let m = app_from_crate!()
    .setting(clap::AppSettings::VersionlessSubcommands)
    .setting(clap::AppSettings::SubcommandRequiredElseHelp)
    .subcommand(clap::SubCommand::with_name("list").about("List compiler information"))
    .subcommand(clap::SubCommand::with_name("run")
      .about("Post a code to wandbox and get a result")
      .args_from_usage(r#"
        <compiler>                      'Compiler name'
        <filename>                      'Target filename'
        --options=[options]             'Used options (separated by comma)'
        --compile-args=[compiler-args]  'Arguments for compiler'
        --runtime-args=[runtime-args]   'Arguments for compiled binary or interpreter'
        --permlink                      'Generate permlink'
      "#))
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
      let options = m.value_of("options").unwrap_or_default();
      let compiler_args =
        m.value_of("compiler-args").and_then(|s| shlex::split(s)).unwrap_or_default();
      let runtime_args =
        m.value_of("runtime-args").and_then(|s| shlex::split(s)).unwrap_or_default();
      let permlink = m.is_present("permlink");

      match run(compiler,
                filename,
                options,
                compiler_args,
                runtime_args,
                permlink) {
        Ok(code) => std::process::exit(code),
        Err(err) => writeln!(&mut io::stderr(), "failed with: {:?}", err).unwrap(),
      }
    }
    _ => unreachable!(),
  }
}
