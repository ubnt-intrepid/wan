extern crate hyper;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;

mod compile;
mod list;
mod util;
mod http;

pub use compile::{Compile, CompileResult};
pub use list::get_compiler_info;

error_chain! {
  foreign_links {
    Io(::std::io::Error);
    Hyper(::hyper::error::Error);
    SerdeJson(::serde_json::Error);
  }
}
