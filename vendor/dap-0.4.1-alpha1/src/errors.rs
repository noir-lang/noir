use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeserializationError {
  #[error("could not parse value '{value}' to enum variant of '{enum_name}'")]
  StringToEnumParseError { enum_name: String, value: String },
  #[error("Error while deserializing")]
  SerdeError(#[from] serde_json::Error),
  #[error("Error decoding character stream")]
  DecodingError(std::str::Utf8Error),
}

#[derive(Debug, Error)]
pub enum ServerError {
  #[error("I/O error")]
  IoError(std::io::Error),

  #[error("Unknown header: {header}")]
  UnknownHeader { header: String },

  #[error("Parse error")]
  ParseError(#[from] DeserializationError),

  #[error("Could not parse header line '{line}'")]
  HeaderParseError { line: String },

  #[error("Protocol error while reading line '{line}', reason: '{reason}'")]
  ProtocolError { reason: String, line: String },

  #[error("Serialization error")]
  SerializationError(#[from] serde_json::Error),

  #[error("Trying to construct a non-sense response (such as an ACK for a request that requires a response body")]
  ResponseConstructError,

  #[error("Output lock is poisoned")]
  OutputLockError,
}
