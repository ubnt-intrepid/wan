#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate wan;

fn main() {
  env_logger::init().unwrap();

  let m = app_from_crate!()
    .arg_from_usage("<compiler>       'compiler name'")
    .arg_from_usage("<filename>       'target filename'")
    .arg_from_usage("[<arguments>...] 'supplemental arguments to pass compiled binary'")
    .get_matches();
  let compiler = m.value_of("compiler").unwrap();
  let filename = m.value_of("filename").unwrap();
  let arguments: Vec<_> = m.values_of("arguments").map(|v| v.collect()).unwrap_or_default();

  let response = wan::compile_request(compiler, &filename, &arguments).unwrap();
  if let Some(message) = response.program_message {
    println!("{}", message);
  } else {
    println!("{}", response.compiler_message.unwrap());
  }

  std::process::exit(response.status);
}
