#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate wan;
extern crate shlex;

use std::fs::File;
use std::io::{self, Read, Write};
use clap::AppSettings;


#[derive(Debug)]
struct ListApp {
  dump: bool,
}

impl ListApp {
  fn subcommand<'a, 'b: 'a>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("list")
      .about("List compiler information")
      .arg_from_usage("-d, --dump  'Dump as raw JSON format'")
  }

  fn from_matches<'a>(m: &clap::ArgMatches<'a>) -> ListApp {
    ListApp { dump: m.is_present("dump") }
  }

  fn run(self) -> wan::Result<i32> {
    let info_list = wan::list::get_compiler_info()?;

    if self.dump {
      wan::util::dump_to_json(&info_list)?;
    } else {
      for info in info_list {
        println!("{}", info);
      }
    }

    Ok(0)
  }
}


#[derive(Debug)]
struct RunApp<'a> {
  compiler: &'a str,
  filename: &'a str,
  options: &'a str,
  compiler_args: Vec<String>,
  runtime_args: Vec<String>,
  permlink: bool,
}

impl<'a> RunApp<'a> {
  fn subcommand<'b, 'c: 'b>() -> clap::App<'b, 'c> {
    clap::SubCommand::with_name("run")
      .about("Post a code to wandbox and get a result")
      .args_from_usage(r#"
        <compiler>                      'Compiler name'
        <filename>                      'Target filename'
        --options=[options]             'Used options (separated by comma)'
        --compile-args=[compiler-args]  'Arguments for compiler'
        --runtime-args=[runtime-args]   'Arguments for compiled binary or interpreter'
        --permlink                      'Generate permlink'
      "#)
  }

  fn from_matches<'b, 'c: 'b>(m: &'b clap::ArgMatches<'c>) -> RunApp<'b> {
    RunApp {
      compiler: m.value_of("compiler").unwrap(),
      filename: m.value_of("filename").unwrap(),
      options: m.value_of("options").unwrap_or_default(),
      compiler_args: m.value_of("compiler-args").and_then(|s| shlex::split(s)).unwrap_or_default(),
      runtime_args: m.value_of("runtime-args").and_then(|s| shlex::split(s)).unwrap_or_default(),
      permlink: m.is_present("permlink"),
    }
  }

  fn run(self) -> wan::Result<i32> {
    let mut code = String::new();
    if self.filename != "-" {
      File::open(self.filename)?
        .read_to_string(&mut code)?;
    } else {
      io::stdin().read_to_string(&mut code)?;
    }

    let result = wan::compile::Parameter::new(code, self.compiler).options(self.options)
      .compiler_option(self.compiler_args)
      .runtime_option(self.runtime_args)
      .save(self.permlink)
      .request()?;
    wan::util::dump_to_json(&result)?;

    Ok(result.status())
  }
}


#[derive(Debug)]
struct PermlinkApp<'a> {
  link: &'a str,
}

impl<'a> PermlinkApp<'a> {
  fn subcommand<'b, 'c: 'b>() -> clap::App<'b, 'c> {
    clap::SubCommand::with_name("permlink")
      .about("Get a result specified a given permanent link")
      .arg_from_usage("<link> 'Link name'")
  }

  fn from_matches<'b, 'c: 'b>(m: &'b clap::ArgMatches<'c>) -> PermlinkApp<'b> {
    PermlinkApp { link: m.value_of("link").unwrap() }
  }

  fn run(self) -> wan::Result<i32> {
    let result = wan::permlink::get_from_permlink(self.link)?;
    wan::util::dump_to_json(&result)?;
    Ok(0)
  }
}


fn main() {
  let m = app_from_crate!()
    .setting(AppSettings::VersionlessSubcommands)
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .subcommand(ListApp::subcommand())
    .subcommand(RunApp::subcommand())
    .subcommand(PermlinkApp::subcommand())
    .get_matches();
  let result = match m.subcommand() {
    ("list", Some(m)) => ListApp::from_matches(m).run(),
    ("run", Some(m)) => RunApp::from_matches(m).run(),
    ("permlink", Some(m)) => PermlinkApp::from_matches(m).run(),
    _ => unreachable!(),
  };

  match result {
    Ok(code) => std::process::exit(code),
    Err(err) => writeln!(&mut io::stderr(), "failed with: {:?}", err).unwrap(),
  }
}
