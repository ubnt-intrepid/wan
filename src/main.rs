extern crate curl;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::io::{self, Read, Write};
use std::io::{BufRead, BufReader};
use std::fs::File;

#[derive(Debug, Serialize)]
struct Request {
  code: String,
  compiler: String,
  runtime_option_raw: String,
}

#[derive(Debug, Deserialize)]
struct Response {
  program_message: Option<String>,
  program_output: Option<String>,
  compiler_message: Option<String>,
  status: i32,
}

fn make_request() -> Request {
  let command = std::env::args().next().unwrap();
  let args: Vec<_> = std::env::args().skip(1).collect();
  if args.len() < 2 {
    let _ = writeln!(&mut io::stderr(),
                     "Usage: {} [compiler] [file] [arguments...]",
                     command);
    std::process::exit(1);
  }

  let mut code = String::new();
  if args[1] != "-" {
    let mut f = BufReader::new(File::open(&args[1]).unwrap());
    let mut dummy = String::new();
    f.read_line(&mut dummy).unwrap();
    f.read_to_string(&mut code).unwrap();
  } else {
    let _ = std::io::stdin().read_to_string(&mut code).unwrap();
  }

  let compiler = args[0].clone();
  let runtime_option_raw = args[2..].join("\n");

  Request {
    code: code,
    compiler: compiler,
    runtime_option_raw: runtime_option_raw,
  }
}

fn get_response(request: Request) -> Response {
  let request_str = serde_json::ser::to_string(&request).unwrap();

  let mut headers = curl::easy::List::new();
  headers.append("Content-Type: application/json").unwrap();

  let chunk = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
  let write_callback = {
    let c = chunk.clone();
    move |data: &[u8]| {
      use std::borrow::Borrow;
      c.lock().unwrap().push_str(String::from_utf8_lossy(data).borrow());
      Ok(data.len())
    }
  };

  let mut easy = curl::easy::Easy::new();
  easy.http_headers(headers).unwrap();
  easy.url("http://melpon.org/wandbox/api/compile.json").unwrap();
  easy.post(true).unwrap();
  easy.post_fields_copy(request_str.as_bytes()).unwrap();
  easy.write_function(write_callback).unwrap();
  easy.perform().unwrap();
  let _ = easy.response_code().unwrap();

  let response = serde_json::de::from_str(chunk.lock().unwrap().as_str()).unwrap();

  response
}

fn main() {
  env_logger::init().unwrap();

  let request = make_request();
  trace!("request = {:?}", request);

  let response = get_response(request);
  trace!("response = {:?}", response);

  if let Some(message) = response.program_message {
    println!("{}", message);
  } else {
    println!("{}", response.compiler_message.unwrap());
  }
  std::process::exit(response.status);
}
