extern crate hyper;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate shlex;
extern crate clap;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;

pub mod app;
pub mod language;
pub mod wandbox;
pub mod util;

error_chain! {
  foreign_links {
    Io(::std::io::Error);
    Hyper(::hyper::error::Error);
    Regex(::regex::Error);
    SerdeJson(::serde_json::Error);
  }
}
