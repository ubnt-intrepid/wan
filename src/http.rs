use serde;
use serde_json;
use Result;

#[cfg(not(feature = "http-hyper"))]
use curl;
#[cfg(feature = "http-hyper")]
use hyper;


#[cfg(not(feature = "http-hyper"))]
pub fn get_json<Res>(url: &str, _headers: &[&str]) -> Result<Res>
  where Res: serde::Deserialize
{
  let mut headers = curl::easy::List::new();
  for header in _headers {
    headers.append(header)?;
  }

  let mut easy = curl::easy::Easy::new();
  easy.http_headers(headers)?;
  easy.url(url)?;
  easy.get(true)?;

  let mut buf = Vec::new();
  {
    let mut transfer = easy.transfer();
    transfer.write_function(|data: &[u8]| {
        buf.extend_from_slice(data);
        Ok(data.len())
      })?;
    transfer.perform()?;
  }

  serde_json::de::from_slice(&buf).map_err(Into::into)
}

#[cfg(feature = "http-hyper")]
pub fn get_json<Res>(url: &str, _headers: &[&str]) -> Result<Res>
  where Res: serde::Deserialize
{
  let client = hyper::Client::new();
  let res: hyper::client::Response = client.get(url).send()?;
  serde_json::from_reader(res).map_err(Into::into)
}


#[cfg(not(feature = "http-hyper"))]
pub fn post<Req, Res>(url: &str, request: Req) -> Result<Res>
  where Req: serde::Serialize,
        Res: serde::Deserialize
{
  let mut headers = curl::easy::List::new();
  headers.append("Content-Type: application/json")?;

  let mut easy = curl::easy::Easy::new();
  easy.http_headers(headers)?;
  easy.url(url)?;
  easy.post(true)?;
  easy.post_fields_copy(&serde_json::to_vec(&request)?)?;

  let mut buf = Vec::new();
  {
    let mut transfer = easy.transfer();
    transfer.write_function(|data: &[u8]| {
        buf.extend_from_slice(data);
        Ok(data.len())
      })?;
    transfer.perform()?;
  }

  let result = serde_json::from_slice(&buf)?;
  Ok(result)
}

#[cfg(feature = "http-hyper")]
pub fn post<Req, Res>(url: &str, request: Req) -> Result<Res>
  where Req: serde::Serialize,
        Res: serde::Deserialize
{
  let client = hyper::Client::new();
  let res = client.post(url)
    .header(hyper::header::ContentType::json())
    .body(&serde_json::to_string(&request)?)
    .send()?;
  serde_json::from_reader(res).map_err(Into::into)
}
