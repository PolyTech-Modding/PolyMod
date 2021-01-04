use crate::error::*;
use crate::model::*;

use actix_identity::Identity;
use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse};

use darkredis::ConnectionPool;
use handlebars::Handlebars;
use sqlx::postgres::PgPool;

#[derive(Serialize, Deserialize, Debug)]
pub struct MeResponseData {
    roles: u32,
    user_id: u64,
    user_id_string: String,
    is_banned: bool,
    token: String,

    discord: UserResponse,
}

pub async fn get_user_data(token: &str) -> ServiceResult<UserResponse> {
    let client = reqwest::Client::new();

    let user = client
        .get(&format!("{}/users/@me", API_ENDPOINT))
        .bearer_auth(&token)
        .send()
        .await?
        .json::<UserResponse>()
        .await?;

    Ok(user)
}

pub async fn index(
    id: Identity,
    hb: web::Data<Handlebars<'_>>,
    redis: web::Data<ConnectionPool>,
) -> ServiceResult<HttpResponse> {
    let mut conn = redis.get().await;

    if let Some(user_id) = id.identity() {
        if let Ok(Some(token)) = conn.get(&user_id).await {
            let token = String::from_utf8(token).unwrap();
            let user = get_user_data(&token).await?;

            let data = serde_json::json!({
                "name": user.username,
                "discriminator": user.discriminator,
            });

            let body = hb.render("discord_user", &data).unwrap();

            return Ok(HttpResponse::Ok().body(body));
        }
    }

    Ok(HttpResponse::Found()
        .header(header::LOCATION, "/login")
        .finish())
}

pub async fn me(
    req: HttpRequest,
    id: Identity,
    redis: web::Data<ConnectionPool>,
    db: web::Data<PgPool>,
) -> ServiceResult<HttpResponse> {
    let pool = &**db;
    let mut conn = redis.get().await;

    if let Some(user_id) = id.identity() {
        if let Ok(Some(token)) = conn.get(&user_id).await {
            let token = String::from_utf8(token).unwrap();
            let user = get_user_data(&token).await?;

            let query = sqlx::query!("SELECT * FROM tokens WHERE user_id = $1", user.id as i64)
                .fetch_optional(pool)
                .await?;

            if let Some(data) = query {
                let roles = Roles::from_bits_truncate(data.roles as u32);

                let data = MeResponseData {
                    roles: roles.bits(),
                    user_id: data.user_id as u64,
                    user_id_string: data.user_id.to_string(),
                    is_banned: data.is_banned,
                    token: data.token.to_string(),
                    discord: user,
                };

                return Ok(HttpResponse::Ok().json(data));
            }

            return Ok(HttpResponse::NoContent().finish());
        }
    }

    if let Some(token) = req.headers().get("Authorization") {
        let token = token.to_str().unwrap();

        let query = sqlx::query!("SELECT * FROM tokens WHERE token = $1", token)
            .fetch_optional(pool)
            .await?;

        if let Some(data) = query {
            if let Ok(Some(oauth_token)) = conn.get(&data.user_id.to_string()).await {
                let user = get_user_data(&String::from_utf8(oauth_token).unwrap()).await?;
                let roles = Roles::from_bits_truncate(data.roles as u32);

                let data = MeResponseData {
                    roles: roles.bits(),
                    user_id: data.user_id as u64,
                    user_id_string: data.user_id.to_string(),
                    is_banned: data.is_banned,
                    token: token.to_string(),
                    discord: user,
                };

                return Ok(HttpResponse::Ok().json(data));
            } else {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "OAuth2 Session Expired"
                })));
            }
        }
    }

    Ok(HttpResponse::NoContent().finish())
}
