extern crate curl;
extern crate env_logger;
extern crate serde_json;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;

use std::io::{self, Read, Write, BufRead};
use std::sync;
use curl::easy;

error_chain! {
  foreign_links {
    Io(::std::io::Error);
    Curl(::curl::Error);
    SerdeJson(::serde_json::Error);
  }
}

#[derive(Debug, Serialize)]
struct Request {
  compiler: String,
  code: String,
  runtime_option_raw: String,
}

#[derive(Debug, Deserialize)]
struct Response {
  program_message: Option<String>,
  program_output: Option<String>,
  compiler_message: Option<String>,
  status: i32,
}

fn make_request(compiler: &str, filename: &str, options: &[String]) -> Result<Request> {
  let mut code = String::new();
  if filename != "-" {
    let mut f = io::BufReader::new(std::fs::File::open(filename)?);
    f.read_line(&mut String::new())?;
    f.read_to_string(&mut code)?;
  } else {
    io::stdin().read_to_string(&mut code)?;
  }

  Ok(Request {
    compiler: compiler.to_owned(),
    code: code,
    runtime_option_raw: options.join("\n"),
  })
}

fn get_response(request: Request) -> Result<Response> {
  let request_str = serde_json::ser::to_string(&request)?;

  let mut headers = easy::List::new();
  headers.append("Content-Type: application/json")?;

  let chunk = sync::Arc::new(sync::Mutex::new(String::new()));
  let write_callback = {
    let c = chunk.clone();
    move |data: &[u8]| {
      use std::borrow::Borrow;
      c.lock().unwrap().push_str(String::from_utf8_lossy(data).borrow());
      Ok(data.len())
    }
  };

  let mut easy = easy::Easy::new();
  easy.http_headers(headers)?;
  easy.url("http://melpon.org/wandbox/api/compile.json")?;
  easy.post(true)?;
  easy.post_fields_copy(request_str.as_bytes())?;
  easy.write_function(write_callback)?;
  easy.perform()?;

  let response = serde_json::de::from_str(chunk.lock().unwrap().as_str())?;
  Ok(response)
}

fn main() {
  env_logger::init().unwrap();

  let args: Vec<_> = std::env::args().skip(1).collect();
  if args.len() < 2 {
    let _ = writeln!(&mut io::stderr(),
                     "Usage: {} [compiler] [file] [arguments...]",
                     std::env::args().next().unwrap());
    std::process::exit(1);
  }

  let request = make_request(&args[0], &args[1], &args[2..]).unwrap();
  trace!("request = {:?}", request);

  let response = get_response(request).unwrap();
  trace!("response = {:?}", response);

  if let Some(message) = response.program_message {
    println!("{}", message);
  } else {
    println!("{}", response.compiler_message.unwrap());
  }
  std::process::exit(response.status);
}
