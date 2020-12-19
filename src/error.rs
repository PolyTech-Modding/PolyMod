use std::error::Error;
use std::convert::From;
use std::fmt::{Display, Formatter};

use actix_web::{error::ResponseError, HttpResponse};

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Debug, Deserialize, Serialize)]
pub enum ServiceError {
    BadRequest(String),
    InternalServerError(String),
    Unauthorized,
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ServiceError {}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::BadRequest(ref message) => HttpResponse::BadRequest().body(message),
            ServiceError::InternalServerError(ref message) => HttpResponse::InternalServerError().body(message),
            ServiceError::Unauthorized => HttpResponse::Unauthorized().body("Unauthorized"),
        }
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(err: std::io::Error) -> ServiceError {
        ServiceError::InternalServerError(format!("IO Error Happened: {}", err))
    }
}
