extern crate hyper;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate shlex;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;

pub mod app;
pub mod compile;
pub mod list;
pub mod http;
pub mod util;
pub mod permlink;

error_chain! {
  foreign_links {
    Io(::std::io::Error);
    Hyper(::hyper::error::Error);
    Regex(::regex::Error);
    SerdeJson(::serde_json::Error);
  }
}
