#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate wan;
extern crate shlex;

use std::fs::File;
use std::io::{self, Read, Write};
use clap::{AppSettings, SubCommand};


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
  fn run(self) -> wan::Result<i32>;
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


struct RunApp {
  compiler: String,
  filename: String,
  files: Option<Vec<String>>,
  options: Option<String>,
  compiler_args: Option<String>,
  runtime_args: Option<String>,
  permlink: bool,
}

impl<'a, 'b: 'a> MakeApp<'a, 'b> for RunApp {
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

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for RunApp {
  fn from(m: &'b clap::ArgMatches<'a>) -> RunApp {
    RunApp {
      compiler: m.value_of("compiler").map(ToOwned::to_owned).unwrap(),
      filename: m.value_of("filename").map(ToOwned::to_owned).unwrap(),
      files: m.values_of("files").map(|v| v.into_iter().map(ToOwned::to_owned).collect()),
      options: m.value_of("options").map(ToOwned::to_owned),
      compiler_args: m.value_of("compiler-args").map(ToOwned::to_owned),
      runtime_args: m.value_of("runtime-args").map(ToOwned::to_owned),
      permlink: m.is_present("permlink"),
    }
  }
}

impl Run for RunApp {
  fn run(self) -> wan::Result<i32> {
    let mut code = String::new();
    if self.filename != "-" {
      File::open(self.filename)?
        .read_to_string(&mut code)?;
    } else {
      io::stdin().read_to_string(&mut code)?;
    }
    let compiler_args = self.compiler_args.and_then(|s| shlex::split(&s)).unwrap_or_default();
    let runtime_args = self.runtime_args.and_then(|s| shlex::split(&s)).unwrap_or_default();
    let codes: Option<Vec<_>> = self.files
      .map(|v| v.into_iter().map(|ref s| wan::compile::Code::new(s)).collect());

    let mut parameter = wan::compile::Parameter::new(code, self.compiler)
      .options(self.options.unwrap_or_default())
      .compiler_option(compiler_args)
      .runtime_option(runtime_args)
      .save(self.permlink);

    if let Some(codes) = codes {
      parameter = parameter.codes(codes);
    }

    let result = parameter.request()?;
    wan::util::dump_to_json(&result)?;

    Ok(result.status())
  }
}


struct PermlinkApp {
  link: String,
}

impl<'a, 'b: 'a> MakeApp<'a, 'b> for PermlinkApp {
  fn make_app(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.about("Get a result specified a given permanent link")
      .arg_from_usage("<link> 'Link name'")
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for PermlinkApp {
  fn from(m: &'b clap::ArgMatches<'a>) -> PermlinkApp {
    PermlinkApp { link: m.value_of("link").map(ToOwned::to_owned).unwrap() }
  }
}

impl Run for PermlinkApp {
  fn run(self) -> wan::Result<i32> {
    let result = wan::permlink::get_from_permlink(&self.link)?;
    wan::util::dump_to_json(&result)?;
    Ok(0)
  }
}


enum App {
  Run(RunApp),
  List(ListApp),
  Permlink(PermlinkApp),
}

impl<'a, 'b: 'a> MakeApp<'a, 'b> for App {
  fn make_app(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    let app = app.subcommand(SubCommand::with_name("list").register::<ListApp>());
    let app = app.subcommand(SubCommand::with_name("run").register::<RunApp>());
    let app = app.subcommand(SubCommand::with_name("permlink").register::<PermlinkApp>());
    app
  }
}

impl<'a> From<clap::ArgMatches<'a>> for App {
  fn from(m: clap::ArgMatches<'a>) -> App {
    match m.subcommand() {
      ("list", Some(m)) => App::List(m.into()),
      ("run", Some(m)) => App::Run(m.into()),
      ("permlink", Some(m)) => App::Permlink(m.into()),
      _ => unreachable!(),
    }
  }
}

impl Run for App {
  fn run(self) -> wan::Result<i32> {
    match self {
      App::List(a) => a.run(),
      App::Run(a) => a.run(),
      App::Permlink(a) => a.run(),
    }
  }
}


fn main() {
  let app: App = app_from_crate!()
    .setting(AppSettings::VersionlessSubcommands)
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .register::<App>()
    .get_matches()
    .into();

  match app.run() {
    Ok(code) => std::process::exit(code),
    Err(err) => writeln!(&mut io::stderr(), "failed with: {:?}", err).unwrap(),
  }
}
