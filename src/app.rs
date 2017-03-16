use std::fs::File;
use std::io::{self, Read};
use clap;
use shlex;
use regex::Regex;

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

pub struct ListApp<'a> {
  name_only: bool,
  name: Option<&'a str>,
  lang: Option<&'a str>,
}

impl<'c> MakeApp for ListApp<'c> {
  fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.about("List compiler information")
      .arg_from_usage("--name-only      'Display names only'")
      .arg_from_usage("--name=[name]    'Filter by name with Regex pattern'")
      .arg_from_usage("--lang=[lang]    'Filter by language with Regex pattern'")
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for ListApp<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> ListApp<'a> {
    ListApp {
      name_only: m.is_present("name-only"),
      name: m.value_of("name"),
      lang: m.value_of("lang"),
    }
  }
}

impl<'a> Run for ListApp<'a> {
  type Err = ::Error;

  fn run(self) -> Result<i32, Self::Err> {
    let ptn_name = match self.name {
      Some(name) => Some(Regex::new(&name)?),
      None => None,
    };

    let ptn_lang = match self.lang {
      Some(lang) => {
        if lang == "C++" {
          Some(Regex::new(r"C\+\+")?)
        } else {
          Some(Regex::new(&lang)?)
        }
      }
      None => None,
    };

    let info_list = list::get_compiler_info()?;
    let info_list = info_list.into_iter()
      .filter(move |info| {
        ptn_name.as_ref()
          .map(|m| m.is_match(&info.name))
          .unwrap_or(true) &&
        ptn_lang.as_ref()
          .map(|m| m.is_match(&format!("{}", info.language)))
          .unwrap_or(true)
      });

    if self.name_only {
      for info in info_list {
        println!("{}", info.name);
      }
    } else {
      util::dump_to_json(&info_list.collect::<Vec<_>>())?;
    }

    Ok(0)
  }
}


pub struct RunApp<'a> {
  filename: &'a str,
  files: Option<clap::Values<'a>>,
  compiler: Option<&'a str>,
  options: Option<&'a str>,
  compiler_args: Option<&'a str>,
  runtime_args: Option<&'a str>,
  permlink: bool,
}

impl<'c> MakeApp for RunApp<'c> {
  fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.about("Post a code to wandbox and get a result")
      .args_from_usage(r#"
        <filename>                      'Target filename'
        [files...]                      'Supplemental files'
        --compiler=[compiler]           'Compiler name'
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
      filename: m.value_of("filename").unwrap(),
      files: m.values_of("files"),
      compiler: m.value_of("compiler"),
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
    let mut lang = None;
    if self.filename != "-" {
      File::open(self.filename)?
        .read_to_string(&mut code)?;
      use std::path::PathBuf;
      use std::borrow::Borrow;
      lang = PathBuf::from(self.filename)
        .extension()
        .and_then(|ext| list::Language::from_extension(ext.to_string_lossy().borrow()).ok());
    } else {
      io::stdin().read_to_string(&mut code)?;
    }

    let compiler = self.compiler.or_else(|| lang.and_then(|ref lang| list::DEFAULT_COMPILERS.get(lang).map(|s| *s)))
      .unwrap_or("gcc-head");

    let mut parameter = compile::Parameter::new(code, compiler).save(self.permlink);

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
  List(ListApp<'a>),
  Run(RunApp<'a>),
  Permlink(PermlinkApp<'a>),
}


impl<'c> MakeApp for App<'c> {
  fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    let app = app.register_subcommand::<ListApp>("list");
    let app = app.register_subcommand::<RunApp>("run");
    let app = app.register_subcommand::<PermlinkApp>("permlink");
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
  type Err = ::Error;

  fn run(self) -> Result<i32, Self::Err> {
    match self {
      App::List(a) => a.run(),
      App::Run(a) => a.run(),
      App::Permlink(a) => a.run(),
    }
  }
}
