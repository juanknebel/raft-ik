use axum::response::{IntoResponse, Response};
use log::error;
use reqwest::StatusCode;
use serde::Serialize;

/// This kind of Result encapsulates a success Ok or a failure Err, but the
/// failure has an error that is match with a htttp status and an internal
/// error.
pub type Result<T> = core::result::Result<T, Error>;

/// The application errors.
/// As the application grows this must be split in specific errors per module.
/// But at the moment, is better to centralize the definition.
#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
  ParseError { kind: String },
  InteralError,
  LockError,
}

impl Error {
  pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
    match self {
      Self::ParseError {
        ..
      } => (
        StatusCode::BAD_REQUEST,
        ClientError::InvalidParams,
      ),
      Self::InteralError => (
        StatusCode::INTERNAL_SERVER_ERROR,
        ClientError::ServiceError,
      ),
      Self::LockError => (StatusCode::LOCKED, ClientError::LockError),
      _ => (
        // Fallback error
        StatusCode::INTERNAL_SERVER_ERROR,
        ClientError::ServiceError,
      ),
    }
  }
}

impl core::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    let (status_code, client_error) = self.client_status_and_error();
    error!("[client_error: {client_error}] [trace: {self:?}]");
    let mut response = status_code.into_response();

    response.extensions_mut().insert(self);

    response
  }
}

#[derive(Debug, strum_macros::AsRefStr)]
pub enum ClientError {
  ServiceError,
  InvalidParams,
  LockError,
}

impl core::fmt::Display for ClientError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}
