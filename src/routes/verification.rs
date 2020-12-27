use crate::error::ServiceResult;
use crate::model::Roles;
use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::{PgPool, postgres::PgDatabaseError};

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyData {
    checksum: String,
    is_good: bool,
    reason: Option<String>,
}

pub async fn verify(
    req: HttpRequest,
    data: web::Query<VerifyData>,
    db: web::Data<PgPool>,
) -> ServiceResult<HttpResponse> {
    let pool = &**db;

    let query = sqlx::query!(
        "SELECT user_id, roles FROM tokens WHERE token = $1",
        // unwrap is safe this method only runs when the /api token check has been done.
        req.headers().get("Authorization").unwrap().to_str().unwrap()
    )
    .fetch_optional(pool)
    .await?;

    if let Some(query_data) = query {
        let roles = Roles::from_bits_truncate(query_data.roles as u32);
        dbg!(&query_data.roles);
        dbg!(&roles);

        if roles.contains(Roles::VERIFYER) {
            if !data.is_good && data.reason.is_none() {
                return Ok(HttpResponse::BadRequest().body("Unable to submit failed verification without a reason."))
            }

            if let Some(reason) = &data.reason {
                if !reason.contains(" ") {
                    return Ok(HttpResponse::BadRequest().body("Invalid reason."))
                }
            }

            if let Err(why) = sqlx::query!(
                "INSERT INTO verification (checksum, verifier_id, is_good, reason) VALUES ($1, $2, $3, $4)",
                &data.checksum,
                &query_data.user_id,
                &data.is_good,
                data.reason.as_ref(),
            )
            .execute(pool)
            .await {
                match why {
                    sqlx::Error::Database(x) => {
                        if let Some(_constraint) = x.downcast_ref::<PgDatabaseError>().constraint() {
                            return Ok(HttpResponse::BadRequest().body("You have already submitted a verification for this mod."))
                        }
                    }
                    _ => return Err(why.into())
                }
            };
        } else {
            return Ok(HttpResponse::Unauthorized().body("User not allowed to verify."))
        }
    } else {
        return Ok(HttpResponse::Unauthorized().body("Token provided not bound to a user."))
    }

    Ok(HttpResponse::Ok().body("Successfully verified mod"))
}
