extern crate hyper;
extern crate hyper_native_tls;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate shellexpand;
extern crate shlex;
extern crate clap;
extern crate url;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;

pub mod app;
pub mod config;
pub mod language;
pub mod util;
pub mod wandbox;

error_chain! {
  foreign_links {
    Io(::std::io::Error);
    Hyper(::hyper::error::Error);
    HyperNativeTls(::hyper_native_tls::native_tls::Error);
    Regex(::regex::Error);
    SerdeJson(::serde_json::Error);
    UrlParse(::url::ParseError);
    ShellExpand(::shellexpand::LookupError<::std::env::VarError>);
  }
}
