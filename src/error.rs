use std::error::Error;
use std::fmt::{Display, Formatter};

use actix_web::{error::ResponseError, HttpResponse};

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Debug, Deserialize, Serialize)]
pub enum ServiceError {
    BadRequest(String),
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
            ServiceError::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
            ServiceError::Unauthorized => HttpResponse::Unauthorized().json("Unauthorized"),
        }
    }
}

