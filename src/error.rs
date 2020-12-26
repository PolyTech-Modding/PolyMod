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
