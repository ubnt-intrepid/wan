mod compile;
mod list;

use serde;
use serde_json;
use hyper;

// re-exports
pub use self::compile::Code;
pub use self::compile::Parameter as CompileParameter;
pub use self::compile::Result as CompileResult;
pub use self::list::CompilerInfo;
pub use self::list::FromExtension;
pub use self::list::GetDefaultCompiler;
pub use self::list::Language;


/// Represents a Wandbox service.
pub struct Wandbox {
  url: String,
}

impl Wandbox {
  pub fn new() -> Wandbox {
    Wandbox { url: "http://melpon.org/wandbox/".into() }
  }

  pub fn compile(&self, request: CompileParameter) -> ::Result<CompileResult> {
    self.post("compile.json", request)
  }

  pub fn get_compiler_info(&self) -> ::Result<Vec<CompilerInfo>> {
    self.get("list.json?from=wan")
  }

  pub fn get_from_permlink(&self, link: &str) -> ::Result<PermlinkResult> {
    self.get(&format!("permlink/{}", link))
  }


  fn get<Res>(&self, path: &str) -> ::Result<Res>
    where Res: serde::Deserialize
  {
    let url = format!("{}api/{}", self.url, path);

    let client = hyper::Client::new();
    let res: hyper::client::Response = client.get(&url).send()?;
    serde_json::from_reader(res).map_err(Into::into)
  }

  fn post<Body, Res>(&self, path: &str, body: Body) -> ::Result<Res>
    where Body: serde::Serialize,
          Res: serde::Deserialize
  {
    let url = format!("{}api/{}", self.url, path);

    let body = serde_json::to_string(&body)?;

    let client = hyper::Client::new();
    let res = client.post(&url)
      .header(hyper::header::ContentType::json())
      .body(&body)
      .send()?;

    serde_json::from_reader(res).map_err(Into::into)
  }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct PermlinkResult {
  parameter: CompileParameter,
  result: CompileResult,
}
