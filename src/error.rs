use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use std::io;
use thiserror::Error;

/// An `Error` that may occur in this `crate`.
#[derive(Error, Debug)]
pub enum Error {
  #[error(transparent)]
  HttpError(#[from] reqwest::Error),
  /// Connection pool error
  #[error(transparent)]
  PoolError(#[from] r2d2::Error),
  /// Migration error
  #[error(transparent)]
  MigrationError(#[from] diesel_migrations::RunMigrationsError),
  /// Error propagated from the MySQL backend
  #[error(transparent)]
  MysqlError(#[from] diesel::result::Error),
  #[error("Resource not found")]
  NotFound,
  #[error("Command is already completed")]
  CommandAlreadyCompleted,
  #[error("Unknown API route")]
  UnknownRoute,
  #[error("Invalid monitoring: {0}")]
  InvalidMonitoring(String),
}

#[derive(Serialize)]
pub struct ErrorMessage {
  pub code: u16,
  pub error: String,
  pub message: String,
}

impl Error {
  pub fn name(&self) -> String {
    match *self {
      Self::NotFound | Self::UnknownRoute => String::from("Not found"),
      Self::InvalidMonitoring(..) => String::from("Bad request"),
      Self::CommandAlreadyCompleted => String::from("Conflict"),
      _ => String::from("Internal server error"),
    }
  }
}

impl From<Error> for io::Error {
  fn from(error: Error) -> Self {
    Self::new(io::ErrorKind::Other, error)
  }
}

impl ResponseError for Error {
  fn status_code(&self) -> StatusCode {
    match *self {
      Self::NotFound | Self::UnknownRoute => StatusCode::NOT_FOUND,
      Self::CommandAlreadyCompleted => StatusCode::CONFLICT,
      Self::InvalidMonitoring(..) => StatusCode::BAD_REQUEST,
      _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }

  fn error_response(&self) -> HttpResponse {
    let status_code = self.status_code();

    HttpResponse::build(status_code).json(ErrorMessage {
      code: status_code.as_u16(),
      error: self.name(),
      message: self.to_string(),
    })
  }
}
