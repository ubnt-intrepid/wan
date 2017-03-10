use serde;
use serde_json;
use serde_yaml;
use std::io::Write;


#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Either<L, R> {
  Left(L),
  Right(R),
}

impl<L, R> Either<L, R> {
  #[allow(dead_code)]
  pub fn into_left(self) -> Option<L> {
    match self {
      Either::Left(l) => Some(l),
      Either::Right(_) => None,
    }
  }

  #[allow(dead_code)]
  pub fn into_right(self) -> Option<R> {
    match self {
      Either::Left(_) => None,
      Either::Right(r) => Some(r),
    }
  }
}

pub fn str_join<I, S>(iter: I, join: &str) -> String
  where I: IntoIterator<Item = S>,
        S: AsRef<str>
{
  iter.into_iter().fold(String::new(), |mut acc, s| {
    if !acc.is_empty() {
      acc.push_str(join);
    }
    acc.push_str(s.as_ref());
    acc
  })
}

pub fn dump_to_json<S: serde::Serialize>(value: &S) -> ::Result<()> {
  ::std::io::stdout().write_all(serde_json::to_string_pretty(value)?.as_bytes())?;
  Ok(())
}

pub fn dump_to_yaml<S: serde::Serialize>(value: &S) -> ::Result<()> {
  ::std::io::stdout().write_all(serde_yaml::to_string(value)?.as_bytes())?;
  Ok(())
}
