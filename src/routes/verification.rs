use crate::error::*;
use crate::model::*;
use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::{postgres::PgDatabaseError, PgPool};

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
        "SELECT owner_id, roles FROM tokens WHERE token = $1",
        req.headers()
            .get("Authorization")
            // unwrap is safe this method only runs when the /api token check has been done.
            .unwrap()
            .to_str()
            .unwrap()
    )
    .fetch_optional(pool)
    .await?;

    if let Some(query_data) = query {
        let roles = Roles::from_bits_truncate(query_data.roles as u32);

        if roles.contains(Roles::VERIFYER) {
            let query = sqlx::query!(
                r#"SELECT verification as "verification: Verification" FROM mods WHERE checksum = $1"#,
                &data.checksum,
            )
            .fetch_optional(pool)
            .await?;

            if let Some(x) = query {
                if let Some(verification) = x.verification {
                    if verification == Verification::Core {
                        return Ok(HttpResponse::BadRequest().body("Cannot verify Core mods."));
                    } else if verification == Verification::Unsafe {
                        return Ok(HttpResponse::BadRequest()
                            .body("This mod has already been verified as Unsafe."));
                    } else if verification == Verification::Manual {
                        return Ok(HttpResponse::BadRequest()
                            .body("This mod has already been manually verified."));
                    }
                }
            } else {
                return Ok(HttpResponse::BadRequest().body("This mod does not exist."));
            }

            if !data.is_good && data.reason.is_none() {
                return Ok(HttpResponse::BadRequest()
                    .body("Unable to submit failed verification without a reason."));
            }

            if let Some(reason) = &data.reason {
                if !reason.contains(' ') || reason.len() < 60 {
                    return Ok(HttpResponse::BadRequest().body("Invalid or too short of a reason."));
                }
            }

            if let Err(why) = sqlx::query!(
                "INSERT INTO verification (checksum, verifier_id, is_good, reason) VALUES ($1, $2, $3, $4)",
                &data.checksum,
                &query_data.owner_id,
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
            return Ok(HttpResponse::Unauthorized().body("User not allowed to verify."));
        }
    } else {
        return Ok(HttpResponse::Unauthorized().body("Token provided not bound to a user."));
    }

    let query = sqlx::query!(
        "SELECT id, is_good FROM verification WHERE checksum = $1",
        &data.checksum,
    )
    .fetch_all(pool)
    .await?;

    let (good, bad): (Vec<_>, Vec<_>) = query.iter().partition(|i| i.is_good);

    if bad.len() >= 2 {
        sqlx::query!(
            "UPDATE mods SET verification = 'Unsafe' WHERE checksum = $1",
            &data.checksum,
        )
        .execute(pool)
        .await?;
        Ok(HttpResponse::Ok().body("Successfully verified mod as Unsafe."))
    } else if good.len() >= 2 {
        sqlx::query!(
            "UPDATE mods SET verification = 'Manual' WHERE checksum = $1",
            &data.checksum,
        )
        .execute(pool)
        .await?;
        Ok(HttpResponse::Ok().body("Successfully verified mod as Safe."))
    } else {
        Ok(HttpResponse::Ok().body("Successfully added mod verification."))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YankData {
    checksum: String,
    reason: Option<String>,
}

pub async fn yank(
    req: HttpRequest,
    data: web::Query<YankData>,
    db: web::Data<PgPool>,
) -> ServiceResult<HttpResponse> {
    let pool = &**db;

    let query = sqlx::query!(
        "SELECT owner_id, roles FROM tokens WHERE token = $1",
        req.headers()
            .get("Authorization")
            // unwrap is safe this method only runs when the /api token check has been done.
            .unwrap()
            .to_str()
            .unwrap()
    )
    .fetch_optional(pool)
    .await?;

    if let Some(query_data) = query {
        let query = sqlx::query!(
            "SELECT name FROM mods WHERE checksum = $1",
            &data.checksum,
        )
        .fetch_optional(pool)
        .await?;

        if let Some(x) = query {
            let query = sqlx::query!(
                "SELECT * FROM owners WHERE mod_name = $1 AND owner_id = $2",
                &x.name,
                query_data.owner_id,
            )
            .fetch_optional(pool)
            .await?;

            if query.is_some() {
                if let Err(why) = sqlx::query!(
                    "INSERT INTO verification (checksum, verifier_id, reason) VALUES ($1, $2, $3)",
                    &data.checksum,
                    &query_data.owner_id,
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

                sqlx::query!(
                    "UPDATE mods SET verification = 'Yanked' WHERE checksum = $1",
                    &data.checksum,
                )
                .execute(pool)
                .await?;
                return Ok(HttpResponse::Ok().body("Successfully yanked mod."))
            }
        }
    }

    Err(ServiceError::Unauthorized)
}
