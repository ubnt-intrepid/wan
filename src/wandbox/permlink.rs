use super::compile;
use super::http;

#[derive(Debug, Serialize, Deserialize)]
pub struct Result {
  parameter: compile::Parameter,
  result: compile::Result,
}

pub fn get_from_permlink(link: &str) -> ::Result<Result> {
  get_from_url(&format!("http://melpon.org/wandbox/api/permlink/{}", link))
}

pub fn get_from_url(url: &str) -> ::Result<Result> {
  http::get_json(url, &[])
}
