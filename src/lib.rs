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
extern crate wan_derive;

pub mod app;
pub mod compile;
pub mod list;
pub mod permlink;
pub mod util;

error_chain! {
  foreign_links {
    Io(::std::io::Error);
    Hyper(::hyper::error::Error);
    Regex(::regex::Error);
    SerdeJson(::serde_json::Error);
  }
}
