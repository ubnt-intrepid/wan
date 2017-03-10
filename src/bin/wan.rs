#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate wan;
extern crate shlex;

use std::fs::File;
use std::io::{self, Read, Write};
use clap::{AppSettings, SubCommand};
use wan::compile::Code;


trait Register<'a, 'b: 'a> {
  fn register<T: MakeApp<'a, 'b>>(self) -> Self;
}

impl<'a, 'b: 'a> Register<'a, 'b> for clap::App<'a, 'b> {
  fn register<T: MakeApp<'a, 'b>>(self) -> Self {
    T::make_app(self)
  }
}

trait MakeApp<'a, 'b: 'a> {
  fn make_app(app: clap::App<'a, 'b>) -> clap::App<'a, 'b>;
}

trait Run {
  type Err;
  fn run(self) -> Result<i32, Self::Err>;
}



struct ListApp {
  dump: bool,
}

impl<'a, 'b: 'a> MakeApp<'a, 'b> for ListApp {
  fn make_app(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.about("List compiler information")
      .arg(clap::Arg::with_name("dump")
        .short("d")
        .long("dump")
        .help("Dump as raw JSON format"))
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for ListApp {
  fn from(m: &'b clap::ArgMatches<'a>) -> ListApp {
    ListApp { dump: m.is_present("dump") }
  }
}

impl Run for ListApp {
  type Err = wan::Error;

  fn run(self) -> Result<i32, Self::Err> {
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


struct RunApp<'a> {
  compiler: &'a str,
  filename: &'a str,
  files: Option<clap::Values<'a>>,
  options: Option<&'a str>,
  compiler_args: Option<&'a str>,
  runtime_args: Option<&'a str>,
  permlink: bool,
}

impl<'a, 'b: 'a, 'c> MakeApp<'a, 'b> for RunApp<'c> {
  fn make_app(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.about("Post a code to wandbox and get a result")
      .args_from_usage(r#"
        <compiler>                      'Compiler name'
        <filename>                      'Target filename'
        [files...]                      'Supplemental files'
        --options=[options]             'Used options (separated by comma)'
        --compile-args=[compiler-args]  'Arguments for compiler'
        --runtime-args=[runtime-args]   'Arguments for compiled binary or interpreter'
        --permlink                      'Generate permlink'
      "#)
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for RunApp<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> RunApp<'a> {
    RunApp {
      compiler: m.value_of("compiler").unwrap(),
      filename: m.value_of("filename").unwrap(),
      files: m.values_of("files"),
      options: m.value_of("options"),
      compiler_args: m.value_of("compiler-args"),
      runtime_args: m.value_of("runtime-args"),
      permlink: m.is_present("permlink"),
    }
  }
}

impl<'a> Run for RunApp<'a> {
  type Err = wan::Error;

  fn run(self) -> Result<i32, Self::Err> {
    let mut code = String::new();
    if self.filename != "-" {
      File::open(self.filename)?
        .read_to_string(&mut code)?;
    } else {
      io::stdin().read_to_string(&mut code)?;
    }

    let mut parameter = wan::compile::Parameter::new(code, self.compiler).save(self.permlink);

    if let Some(options) = self.options {
      parameter = parameter.options(options);
    }

    if let Some(args) = self.compiler_args.and_then(|s| shlex::split(&s)) {
      parameter = parameter.compiler_option(args);
    }

    if let Some(args) = self.runtime_args.and_then(|s| shlex::split(&s)) {
      parameter = parameter.runtime_option(args);
    }

    if let Some(files) = self.files {
      parameter = parameter.codes(files.map(|ref s| Code::new(s)));
    }

    let result = parameter.request()?;
    wan::util::dump_to_json(&result)?;

    Ok(result.status())
  }
}


struct PermlinkApp<'a> {
  link: &'a str,
}

impl<'a, 'b: 'a, 'c> MakeApp<'a, 'b> for PermlinkApp<'c> {
  fn make_app(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.about("Get a result specified a given permanent link")
      .arg_from_usage("<link> 'Link name'")
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for PermlinkApp<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> PermlinkApp<'a> {
    PermlinkApp { link: m.value_of("link").unwrap() }
  }
}

impl<'a> Run for PermlinkApp<'a> {
  type Err = wan::Error;

  fn run(self) -> Result<i32, Self::Err> {
    let result = wan::permlink::get_from_permlink(&self.link)?;
    wan::util::dump_to_json(&result)?;
    Ok(0)
  }
}


enum App<'a> {
  Run(RunApp<'a>),
  List(ListApp),
  Permlink(PermlinkApp<'a>),
}

impl<'a, 'b: 'a, 'c> MakeApp<'a, 'b> for App<'c> {
  fn make_app(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    let app = app.subcommand(SubCommand::with_name("list").register::<ListApp>());
    let app = app.subcommand(SubCommand::with_name("run").register::<RunApp>());
    let app = app.subcommand(SubCommand::with_name("permlink").register::<PermlinkApp>());
    app
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for App<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> App<'a> {
    match m.subcommand() {
      ("list", Some(m)) => App::List(m.into()),
      ("run", Some(m)) => App::Run(m.into()),
      ("permlink", Some(m)) => App::Permlink(m.into()),
      _ => unreachable!(),
    }
  }
}

impl<'a> Run for App<'a> {
  type Err = wan::Error;

  fn run(self) -> Result<i32, Self::Err> {
    match self {
      App::List(a) => a.run(),
      App::Run(a) => a.run(),
      App::Permlink(a) => a.run(),
    }
  }
}


fn main() {
  let ref matches = app_from_crate!()
    .setting(AppSettings::VersionlessSubcommands)
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .register::<App>()
    .get_matches();

  let app: App = matches.into();
  match app.run() {
    Ok(code) => std::process::exit(code),
    Err(err) => writeln!(&mut io::stderr(), "failed with: {:?}", err).unwrap(),
  }
}
