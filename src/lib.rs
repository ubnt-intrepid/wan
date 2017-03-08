extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;

#[cfg(not(feature = "http-hyper"))]
extern crate curl;

#[cfg(feature = "http-hyper")]
extern crate hyper;

mod compile;
mod list;
mod util;
mod http;

pub use compile::{Compile, CompileResult};
pub use list::get_compiler_info;

#[cfg(not(feature = "http-hyper"))]
error_chain! {
  foreign_links {
    Io(::std::io::Error);
    Curl(::curl::Error);
    SerdeJson(::serde_json::Error);
  }
}

#[cfg(feature = "http-hyper")]
error_chain! {
  foreign_links {
    Io(::std::io::Error);
    Hyper(::hyper::error::Error);
    SerdeJson(::serde_json::Error);
  }
}
