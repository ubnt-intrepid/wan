use hyper;
use serde;
use serde_json;
use Result;


pub fn get_json<Res>(url: &str, _headers: &[&str]) -> Result<Res>
  where Res: serde::Deserialize
{
  let client = hyper::Client::new();
  let res: hyper::client::Response = client.get(url).send()?;
  serde_json::from_reader(res).map_err(Into::into)
}

pub fn post<Req, Res>(url: &str, request: Req) -> Result<Res>
  where Req: serde::Serialize,
        Res: serde::Deserialize
{
  let body = serde_json::to_string(&request)?;

  let client = hyper::Client::new();
  let res = client.post(url)
    .header(hyper::header::ContentType::json())
    .body(&body)
    .send()?;

  serde_json::from_reader(res).map_err(Into::into)
}
