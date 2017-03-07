extern crate wan;
extern crate clap;
extern crate env_logger;

fn main() {
  let app = clap::App::new("wan-run")
    .arg(clap::Arg::with_name("compiler").required(true).takes_value(true))
    .arg(clap::Arg::with_name("filename").required(true).takes_value(true))
    .arg(clap::Arg::with_name("arguments").required(false).takes_value(true).multiple(true));
  let matches = app.get_matches();
  let compiler = matches.value_of("compiler").unwrap();
  let filename = matches.value_of("filename").unwrap();
  let arguments: Vec<_> = matches.values_of("arguments").map(|v| v.collect()).unwrap_or_default();

  env_logger::init().unwrap();
  let response = wan::compile_request(compiler, &filename, &arguments).unwrap();
  if let Some(message) = response.program_message {
    println!("{}", message);
  } else {
    println!("{}", response.compiler_message.unwrap());
  }
  std::process::exit(response.status);
}
