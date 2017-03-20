use std::borrow::Borrow;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

use clap;
use hyper;
use hyper::header::ContentType;
use serde_json;
use shlex;
use regex::Regex;

use wandbox;
use util;

pub struct ListApp<'a> {
  name_only: bool,
  name: Option<&'a str>,
  lang: Option<&'a str>,
}

impl<'c> ListApp<'c> {
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

impl<'a> ListApp<'a> {
  fn run(self) -> Result<i32, ::Error> {
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

    let info_list: Vec<wandbox::CompilerInfo> = {
      let url = "http://melpon.org/wandbox/api/list.json";
      let res = hyper::Client::new().get(url).send()?;
      serde_json::from_reader(res)?
    };

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
  stdin: Option<&'a str>,
  permlink: bool,
}

impl<'c> RunApp<'c> {
  fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.about("Post a code to wandbox and get a result")
      .args_from_usage(r#"
        <filename>                      'Target filename'
        [files...]                      'Supplemental files'
        --compiler=[compiler]           'Compiler name'
        --options=[options]             'Used options (separated by comma)'
        --compile-args=[compiler-args]  'Arguments for compiler'
        --runtime-args=[runtime-args]   'Arguments for compiled binary or interpreter'
        --stdin=[stdin]                 'Standard input'
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
      stdin: m.value_of("stdin"),
      permlink: m.is_present("permlink"),
    }
  }
}

impl<'a> RunApp<'a> {
  fn run(self) -> Result<i32, ::Error> {
    let code = self.read_code()?;
    let compiler = self.guess_compiler().unwrap_or("gcc-head".into());

    let mut parameter = wandbox::Parameter::new(code, compiler);
    parameter.save_permlink(self.permlink);

    if let Some(options) = self.options {
      parameter.options(options);
    }

    if let Some(args) = self.compiler_args.and_then(|s| shlex::split(&s)) {
      parameter.compiler_option(args);
    }

    if let Some(args) = self.runtime_args.and_then(|s| shlex::split(&s)) {
      parameter.runtime_option(args);
    }

    if let Some(files) = self.files {
      parameter.codes(files);
    }

    if let Some(stdin) = self.stdin {
      parameter.stdin(stdin);
    }

    // Show request parameter.
    println!("Request Parameter:");
    println!("{}\n", serde_json::to_string_pretty(&parameter)?);

    // Post compile request to Wandbox
    let result: wandbox::Response = {
      let url = "http://melpon.org/wandbox/api/compile.json";
      let res = hyper::Client::new().post(url)
        .header(ContentType::json())
        .body(&serde_json::to_string(&parameter)?)
        .send()?;
      serde_json::from_reader(res)?
    };

    // util::dump_to_json(&result)?;
    if let Some(ref message) = result.program_message {
      println!("Program message:\n{}", message);
    } else {
      println!("Compiler message:\n{}",
               result.compiler_message.as_ref().unwrap());
    }

    if let Some(url) = result.url {
      println!("Permlink URL:\n{}", url);
    }

    Ok(result.status)
  }

  fn read_code(&self) -> ::Result<String> {
    let mut code = String::new();
    if self.filename != "-" {
      File::open(self.filename)?
        .read_to_string(&mut code)?;
    } else {
      io::stdin().read_to_string(&mut code)?;
    }
    Ok(code)
  }

  fn guess_compiler(&self) -> Option<String> {
    use wandbox::FromExtension;
    use wandbox::GetDefaultCompiler;
    self.compiler
      .or_else(|| if self.filename != "-" {
        PathBuf::from(self.filename)
          .extension()
          .map(|ext| ext.to_string_lossy())
          .and_then(|ext| wandbox::Language::from_extension(ext.borrow()).ok())
          .and_then(|ref lang| lang.get_default_compiler())
      } else {
        None
      })
      .map(ToOwned::to_owned)
  }
}


pub struct PermlinkApp<'a> {
  link: &'a str,
}

impl<'c> PermlinkApp<'c> {
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

impl<'a> PermlinkApp<'a> {
  fn run(self) -> Result<i32, ::Error> {
    let result: wandbox::PermlinkResult = {
      let url = format!("http://melpon.org/wandbox/api/permlink/{}", self.link);
      let res = hyper::Client::new().get(&url).send()?;
      serde_json::from_reader(res)?
    };
    util::dump_to_json(&result)?;
    Ok(0)
  }
}


pub enum App<'a> {
  List(ListApp<'a>),
  Run(RunApp<'a>),
  Permlink(PermlinkApp<'a>),
}

impl<'c> App<'c> {
  pub fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.subcommand(ListApp::make_app(clap::SubCommand::with_name("list")))
      .subcommand(RunApp::make_app(clap::SubCommand::with_name("run")))
      .subcommand(PermlinkApp::make_app(clap::SubCommand::with_name("permlink")))
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

impl<'a> App<'a> {
  pub fn run(self) -> Result<i32, ::Error> {
    match self {
      App::List(a) => a.run(),
      App::Run(a) => a.run(),
      App::Permlink(a) => a.run(),
    }
  }
}
