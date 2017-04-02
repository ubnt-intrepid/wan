use std::borrow::Borrow;
use std::fs::File;
use std::io::{self, Read};
use std::marker::PhantomData;
use std::path::PathBuf;

use clap;
use serde_json;
use shlex;
use url::Url;

use config;
use language;
use wandbox::{self, Wandbox};
use util;

pub struct ListApp<'a> {
  dump: bool,
  marker: PhantomData<&'a usize>,
}

impl<'c> ListApp<'c> {
  fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.about("List compiler information")
       .arg_from_usage("-d, --dump  'Dump to raw JSON'")
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for ListApp<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> ListApp<'a> {
    ListApp {
      dump: m.is_present("dump"),
      marker: PhantomData,
    }
  }
}

impl<'a> ListApp<'a> {
  fn run(self) -> Result<i32, ::Error> {
    let config = config::Config::load()?;
    let cli = Wandbox::new(config.url);

    if self.dump {
      let mut res = cli.get_compiler_info_raw()?;
      io::copy(&mut res, &mut io::stdout())?;
    } else {
      use std::collections::BTreeMap;
      use wandbox::CompilerInfo;
      use util::Either;

      let info = cli.get_compiler_info()?;

      let mut langs = BTreeMap::new();
      for compiler in &info {
        let compiler: &CompilerInfo = &compiler;
        let language = compiler.language.clone();
        if !langs.contains_key(&language) {
          langs.insert(language.clone(), Vec::new());
        }
        langs.get_mut(&language).unwrap().push(compiler.clone());
      }

      for (lang, compilers) in langs {
        println!("{:?}:", lang);
        for compiler in compilers {
          println!("- name: {}", compiler.name);
          if compiler.switches.len() > 0 {
            println!("  switches:");
            for switch in &compiler.switches {
              match *switch {
                Either::Left(ref switch) => {
                  println!("  - name: {:?}", switch.name);
                  println!("    default: {:?}", switch.default);
                }
                Either::Right(ref switch) => {
                  println!("  - default: {:?}", switch.default);
                  println!("    options:");
                  for option in &switch.options {
                    println!("    - name: {:?}", option.name);
                  }
                }
              }
            }
          }
        }
        println!();
      }
    }

    Ok(0)
  }
}


pub struct CompileApp<'a> {
  filename: &'a str,
  files: Option<clap::Values<'a>>,
  compiler: Option<&'a str>,
  options: Option<&'a str>,
  compiler_args: Option<&'a str>,
  runtime_args: Option<&'a str>,
  stdin: Option<&'a str>,
  permlink: bool,
  browse: bool,
  verbose: bool,
}

impl<'c> CompileApp<'c> {
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
        --permlink                      'Generate permlink and output URL at end'
        --browse                        'Open permlink URL'
        -v, --verbose                   'Display verbose output'
      "#)
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for CompileApp<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> CompileApp<'a> {
    CompileApp {
      filename: m.value_of("filename").unwrap(),
      files: m.values_of("files"),
      compiler: m.value_of("compiler"),
      options: m.value_of("options"),
      compiler_args: m.value_of("compiler-args"),
      runtime_args: m.value_of("runtime-args"),
      stdin: m.value_of("stdin"),
      permlink: m.is_present("permlink"),
      browse: m.is_present("browse"),
      verbose: m.is_present("verbose"),
    }
  }
}

impl<'a> CompileApp<'a> {
  fn run(self) -> Result<i32, ::Error> {
    let config = config::Config::load()?;

    let code = self.read_code()?;
    let compiler = self.guess_compiler().unwrap_or("gcc-head".into());

    let mut parameter = wandbox::Parameter::new(code, compiler);
    parameter.save_permlink(self.browse || self.permlink);

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

    // Show request information
    println!("[Request info]");
    println!("compiler = {:?}", parameter.compiler);
    if let Some(ref options) = parameter.options {
      println!("options = {:?}", options);
    }
    if let Some(ref option_raw) = parameter.compiler_option_raw {
      println!("compiler_options = {:?}",
               option_raw.split("\n").collect::<Vec<_>>());
    }
    if let Some(ref option_raw) = parameter.runtime_option_raw {
      println!("runtime_options = {:?}",
               option_raw.split("\n").collect::<Vec<_>>());
    }
    println!("");

    // Send request
    let wandbox = Wandbox::new(config.url);
    let response = wandbox.compile(parameter, self.verbose)?;

    // Show compile response
    if let Some(ref message) = response.program_message {
      println!("[Program message]");
      println!("{}", message);
    } else {
      println!("[Compiler message]");
      println!("{}", response.compiler_message.as_ref().unwrap());
    }
    println!("[Program exited with status {}]", response.status);

    if let Some(url) = response.url {
      println!("[Permlink URL]");
      println!("{}", url);
      if self.browse {
        open_browser(url)?;
      }
    }

    Ok(response.status)
  }

  fn read_code(&self) -> ::Result<String> {
    let mut code = String::new();
    if self.filename != "-" {
      File::open(self.filename)?.read_to_string(&mut code)?;
    } else {
      io::stdin().read_to_string(&mut code)?;
    }
    Ok(code)
  }

  fn guess_compiler(&self) -> Option<String> {
    self.compiler
        .or_else(|| if self.filename != "-" {
                   PathBuf::from(self.filename)
                     .extension()
                     .map(|ext| ext.to_string_lossy())
                     .and_then(|ext| language::get_compiler_from_ext(ext.borrow()))
                 } else {
                   None
                 })
        .map(ToOwned::to_owned)
  }
}


pub struct PermlinkApp<'a> {
  link: &'a str,
  dump: bool,
  browse: bool,
}

impl<'c> PermlinkApp<'c> {
  fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.about("Get a result specified a given permanent link")
       .arg_from_usage("<link>        'Link name'")
       .arg_from_usage("-d, --dump    'Show Raw JSON'")
       .arg_from_usage("-b, --browse  'Open browser'")
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for PermlinkApp<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> PermlinkApp<'a> {
    PermlinkApp {
      link: m.value_of("link").unwrap(),
      dump: m.is_present("dump"),
      browse: m.is_present("browse"),
    }
  }
}

impl<'a> PermlinkApp<'a> {
  fn run(self) -> Result<i32, ::Error> {
    let config = config::Config::load()?;

    let wandbox = Wandbox::new(config.url);
    let s = wandbox.get_permlink(self.link)?;
    let result: PermlinkResult = serde_json::from_str(&s)?;

    if self.dump {
      println!("{}", s);
    } else {
      println!("{}", result);
    }

    if self.browse {
      let url = wandbox.permlink_url(self.link);
      open_browser(url)?;
    }

    Ok(0)
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct PermlinkResult {
  parameter: wandbox::Parameter,
  result: wandbox::Response,
}

impl ::std::fmt::Display for PermlinkResult {
  fn fmt(&self, w: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    // Show request information
    writeln!(w, "[Request info]")?;
    writeln!(w, "compiler = {:?}", self.parameter.compiler)?;
    if let Some(ref options) = self.parameter.options {
      writeln!(w, "options = {:?}", options)?;
    }
    if let Some(ref option_raw) = self.parameter.compiler_option_raw {
      writeln!(w,
               "compiler_options = {:?}",
               option_raw.split("\n").collect::<Vec<_>>())?;
    }
    if let Some(ref option_raw) = self.parameter.runtime_option_raw {
      writeln!(w,
               "runtime_options = {:?}",
               option_raw.split("\n").collect::<Vec<_>>())?;
    }
    writeln!(w)?;

    // Show compile response
    if let Some(ref message) = self.result.program_message {
      writeln!(w, "[Program message]")?;
      writeln!(w, "{}", message)?;
    } else {
      writeln!(w, "[Compiler message]")?;
      writeln!(w, "{}", self.result.compiler_message.as_ref().unwrap())?;
    }
    writeln!(w, "[Program exited with status {}]", self.result.status)
  }
}


pub enum App<'a> {
  List(ListApp<'a>),
  Compile(CompileApp<'a>),
  Permlink(PermlinkApp<'a>),
}

impl<'c> App<'c> {
  pub fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.subcommand(ListApp::make_app(clap::SubCommand::with_name("list")))
       .subcommand(CompileApp::make_app(clap::SubCommand::with_name("compile")))
       .subcommand(PermlinkApp::make_app(clap::SubCommand::with_name("permlink")))
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for App<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> App<'a> {
    match m.subcommand() {
      ("list", Some(m)) => App::List(m.into()),
      ("compile", Some(m)) => App::Compile(m.into()),
      ("permlink", Some(m)) => App::Permlink(m.into()),
      _ => unreachable!(),
    }
  }
}

impl<'a> App<'a> {
  pub fn run(self) -> Result<i32, ::Error> {
    match self {
      App::List(a) => a.run(),
      App::Compile(a) => a.run(),
      App::Permlink(a) => a.run(),
    }
  }
}

#[cfg(target_os = "windows")]
fn open_browser<S: AsRef<str>>(s: S) -> ::Result<()> {
  let url = Url::parse(s.as_ref())?;
  ::std::process::Command::new("explorer").arg(url.as_str())
    .status()?;
  Ok(())
}

#[cfg(target_os = "macos")]
fn open_browser<S: AsRef<str>>(s: S) -> ::Result<()> {
  let url = Url::parse(s.as_ref())?;
  ::std::process::Command::new("open").arg(url.as_str())
    .status()?;
  Ok(())
}

#[cfg(target_os = "linux")]
fn open_browser<S: AsRef<str>>(s: S) -> ::Result<()> {
  let url = Url::parse(s.as_ref())?;
  ::std::process::Command::new("xdg-open").arg(url.as_str())
    .status()?;
  Ok(())
}
