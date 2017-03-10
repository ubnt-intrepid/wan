use std::env;
use std::fs::File;
use std::io::{self, Read, BufRead};
use clap;
use shlex;

use compile;
use list;
use util;
use permlink;


pub trait MakeApp {
  fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b>;
}

pub trait Run {
  type Err;
  fn run(self) -> Result<i32, Self::Err>;
}

pub trait Register {
  fn register<T: MakeApp>(self) -> Self;
}

impl<'a, 'b: 'a> Register for clap::App<'a, 'b> {
  fn register<T: MakeApp>(self) -> Self {
    T::make_app(self)
  }
}

pub trait RegisterSubcommand {
  fn register_subcommand<T: MakeApp>(self, name: &str) -> Self;
}

impl<'a, 'b: 'a> RegisterSubcommand for clap::App<'a, 'b> {
  fn register_subcommand<T: MakeApp>(self, name: &str) -> Self {
    self.subcommand(T::make_app(clap::SubCommand::with_name(name)))
  }
}


pub struct ListApp {
  dump: bool,
}

impl MakeApp for ListApp {
  fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.about("List compiler information")
      .arg_from_usage("-d, --dump  'Dump information as raw JSON format'")
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for ListApp {
  fn from(m: &'b clap::ArgMatches<'a>) -> ListApp {
    ListApp { dump: m.is_present("dump") }
  }
}

impl Run for ListApp {
  type Err = ::Error;

  fn run(self) -> Result<i32, Self::Err> {
    let info_list = list::get_compiler_info()?;

    if self.dump {
      util::dump_to_json(&info_list)?;
    } else {
      for info in info_list {
        println!("{}", info);
      }
    }

    Ok(0)
  }
}


pub struct RunApp<'a> {
  compiler: &'a str,
  filename: &'a str,
  files: Option<clap::Values<'a>>,
  options: Option<&'a str>,
  compiler_args: Option<&'a str>,
  runtime_args: Option<&'a str>,
  permlink: bool,
}

impl<'c> MakeApp for RunApp<'c> {
  fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
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
  type Err = ::Error;

  fn run(self) -> Result<i32, Self::Err> {
    let mut code = String::new();
    if self.filename != "-" {
      File::open(self.filename)?
        .read_to_string(&mut code)?;
    } else {
      io::stdin().read_to_string(&mut code)?;
    }

    let mut parameter = compile::Parameter::new(code, self.compiler).save(self.permlink);

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
      parameter = parameter.codes(files.map(|ref s| compile::Code::new(s)));
    }

    let result = parameter.request()?;
    util::dump_to_json(&result)?;

    Ok(result.status())
  }
}


pub struct ScriptApp<'a> {
  filename: &'a str,
  args: Option<clap::Values<'a>>,
}

impl<'c> MakeApp for ScriptApp<'c> {
  fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.args_from_usage(r#"
      <filename>   'target filename'
      [args...]    'runtime options'
    "#)
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for ScriptApp<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> ScriptApp<'a> {
    ScriptApp {
      filename: m.value_of("filename").unwrap(),
      args: m.values_of("args"),
    }
  }
}

impl<'a> Run for ScriptApp<'a> {
  type Err = ::Error;
  fn run(self) -> ::Result<i32> {
    let mut code = String::new();
    let mut f = ::std::io::BufReader::new(File::open(self.filename)?);
    f.read_line(&mut String::new())?;
    f.read_to_string(&mut code)?;

    let compiler = env::var("WAN_COMPILER").unwrap_or("gcc-head".to_owned());

    let mut parameter = compile::Parameter::new(code, compiler);

    if let Ok(options) = env::var("WAN_OPTIONS") {
      parameter = parameter.options(options);
    }

    let compiler_args = env::var("WAN_COMPILER_OPTIONS")
      .ok()
      .and_then(|ref s| shlex::split(s));
    if let Some(args) = compiler_args {
      parameter = parameter.compiler_option(args);
    }

    if let Some(args) = self.args {
      parameter = parameter.runtime_option(args);
    }

    let result = parameter.request()?;
    result.report();

    Ok(result.status())
  }
}


pub struct PermlinkApp<'a> {
  link: &'a str,
}

impl<'c> MakeApp for PermlinkApp<'c> {
  fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
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
  type Err = ::Error;

  fn run(self) -> Result<i32, Self::Err> {
    let result = permlink::get_from_permlink(&self.link)?;
    util::dump_to_json(&result)?;
    Ok(0)
  }
}


pub enum App<'a> {
  List(ListApp),
  Run(RunApp<'a>),
  Script(ScriptApp<'a>),
  Permlink(PermlinkApp<'a>),
}


impl<'c> MakeApp for App<'c> {
  fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    let app = app.register_subcommand::<ListApp>("list");
    let app = app.register_subcommand::<RunApp>("run");
    let app = app.register_subcommand::<ScriptApp>("script");
    let app = app.register_subcommand::<PermlinkApp>("permlink");
    app
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for App<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> App<'a> {
    match m.subcommand() {
      ("list", Some(m)) => App::List(m.into()),
      ("run", Some(m)) => App::Run(m.into()),
      ("script", Some(m)) => App::Script(m.into()),
      ("permlink", Some(m)) => App::Permlink(m.into()),
      _ => unreachable!(),
    }
  }
}

impl<'a> Run for App<'a> {
  type Err = ::Error;

  fn run(self) -> Result<i32, Self::Err> {
    match self {
      App::List(a) => a.run(),
      App::Run(a) => a.run(),
      App::Script(a) => a.run(),
      App::Permlink(a) => a.run(),
    }
  }
}
