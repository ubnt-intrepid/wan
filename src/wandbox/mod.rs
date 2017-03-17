mod compile;
mod list;
mod http;
mod permlink;

// re-exports
pub use self::compile::Code;
pub use self::compile::Parameter as CompileParameter;
pub use self::compile::Result as CompileResult;
pub use self::list::CompilerInfo;
pub use self::list::FromExtension;
pub use self::list::GetDefaultCompiler;
pub use self::list::Language;
pub use self::permlink::Result as PermlinkResult;


pub struct Wandbox {
  url: String,
}

impl Wandbox {
  pub fn new() -> Wandbox {
    Wandbox {
      url: "http://melpon.org/wandbox/".into(),
    }
  }

  pub fn compile(&self, request: CompileParameter) -> ::Result<CompileResult> {
    request.request()
  }

  pub fn get_compiler_info(&self) -> ::Result<Vec<CompilerInfo>> {
    list::get_compiler_info()
  }

  pub fn get_from_permlink(&self, link:&str) -> ::Result<PermlinkResult> {
    permlink::get_from_permlink(link)
  }
}