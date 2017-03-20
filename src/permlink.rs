use compile;

#[derive(Debug, Serialize, Deserialize)]
pub struct Result {
  parameter: compile::Parameter,
  result: compile::Result,
}
