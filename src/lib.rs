extern crate curl;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;

mod wandbox;

use std::io::{self, Read, BufRead};
use std::sync;
use curl::easy;
use wandbox::{CompileRequest, CompileResponse};

error_chain! {
  foreign_links {
    Io(::std::io::Error);
    Curl(::curl::Error);
    SerdeJson(::serde_json::Error);
  }
}

pub fn compile_request(compiler: &str,
                       filename: &str,
                       options: &[&str])
                       -> Result<CompileResponse> {
  let mut code = String::new();
  if filename != "-" {
    let mut f = io::BufReader::new(std::fs::File::open(filename)?);
    f.read_line(&mut String::new())?;
    f.read_to_string(&mut code)?;
  } else {
    io::stdin().read_to_string(&mut code)?;
  }

  let request = CompileRequest {
    compiler: compiler.to_owned(),
    code: code,
    runtime_option_raw: options.join("\n"),
  };

  let mut easy = easy::Easy::new();

  let mut headers = easy::List::new();
  headers.append("Content-Type: application/json")?;
  easy.http_headers(headers)?;

  easy.url("http://melpon.org/wandbox/api/compile.json")?;
  easy.post(true)?;

  let request_str = serde_json::ser::to_string(&request)?;
  easy.post_fields_copy(request_str.as_bytes())?;

  let chunk = sync::Arc::new(sync::Mutex::new(String::new()));
  {
    let c = chunk.clone();
    easy.write_function(move |data: &[u8]| {
        use std::borrow::Borrow;
        c.lock().unwrap().push_str(String::from_utf8_lossy(data).borrow());
        Ok(data.len())
      })?;
  }

  easy.perform()?;

  let response = serde_json::de::from_str(chunk.lock().unwrap().as_str())?;
  Ok(response)
}
