use std::convert::From;
use std::error::Error;
use std::fmt::{Display, Formatter};

use actix_web::{error::ResponseError, HttpResponse};

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Debug, Deserialize, Serialize)]
pub enum ServiceError {
    BadRequest(String),
    InternalServerError(String),
    Unauthorized,
    NoContent,
    Timeout,
}

impl Error for ServiceError {}

impl Display for ServiceError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::BadRequest(ref message) => HttpResponse::BadRequest().body(message),
            ServiceError::InternalServerError(ref message) => {
                HttpResponse::InternalServerError().body(message)
            }
            ServiceError::Unauthorized => HttpResponse::Unauthorized().body("Unauthorized"),
            ServiceError::NoContent => HttpResponse::NoContent().finish(),
            ServiceError::Timeout => HttpResponse::RequestTimeout().finish(),
        }
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(err: std::io::Error) -> ServiceError {
        ServiceError::InternalServerError(format!("IO Error Happened: {}", err))
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> ServiceError {
        use sqlx::Error as E;

        match err {
            E::Database(why) => Self::BadRequest(why.to_string()),
            E::Decode(why) => {
                Self::InternalServerError(format!("Error occurred while decoding a value: {}", why))
            }
            E::PoolTimedOut => {
                error!("Database Pool Timed Out");
                Self::InternalServerError("A handled database error has happened".into())
            }
            E::PoolClosed => {
                error!("Database Pool Closed");
                Self::InternalServerError("A handled database error has happened".into())
            }
            E::WorkerCrashed => {
                error!("Database Worker Crashed");
                Self::InternalServerError("A handled database error has happened".into())
            }

            _ => {
                error!("{:#?}", err);
                Self::InternalServerError("Unhandled database error".into())
            }
        }
    }
}

impl From<serde_json::Error> for ServiceError {
    fn from(err: serde_json::Error) -> ServiceError {
        ServiceError::InternalServerError(err.to_string())
    }
}

impl From<reqwest::Error> for ServiceError {
    fn from(err: reqwest::Error) -> ServiceError {
        error!("Error happened with Reqwests: {}", err);

        if err.is_status() {
            ServiceError::InternalServerError("Bad Status Received".into())
        } else if err.is_timeout() {
            ServiceError::Timeout
        } else if err.is_connect() {
            ServiceError::InternalServerError("Unable to connect to remote server".into())
        } else if err.is_decode() {
            ServiceError::InternalServerError("Failed to decode response from remote server".into())
        } else {
            ServiceError::InternalServerError("Unhandled Remote Server error has happened".into())
        }
    }
}
