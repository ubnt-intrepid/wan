#[derive(Debug, Deserialize)]
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
