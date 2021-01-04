use crate::error::*;
use crate::model::*;
use crate::routes::users::get_user_data;
use crate::utils::tokens::gen_token;

use actix_identity::Identity;
use actix_web::http::header;
use actix_web::{web, HttpResponse};

use darkredis::ConnectionPool;
use handlebars::Handlebars;
use sqlx::postgres::PgPool;

#[derive(Deserialize, Serialize, Debug)]
pub struct UserInfoLogin {
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OAuthCode {
    code: String,
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

pub async fn login(
    id: Identity,
    hb: web::Data<Handlebars<'_>>,
    redis: web::Data<ConnectionPool>,
    config: web::Data<Config>,
) -> HttpResponse {
    let mut conn = redis.get().await;

    if let Some(user_id) = id.identity() {
        if let Ok(Some(_token)) = conn.get(&user_id).await {
            return HttpResponse::Found()
                .header(header::LOCATION, "/")
                .finish();
        }
    }

    let auth_url = config.oauth2_url.to_string();

    let data = serde_json::json!({
        "auth_url": auth_url,
    });

    let body = hb.render("discord_login", &data).unwrap();

    HttpResponse::Ok().body(&body)
}

pub async fn get_token(
    id: Identity,
    redis: web::Data<ConnectionPool>,
    db: web::Data<PgPool>,
    config: web::Data<Config>,
) -> ServiceResult<HttpResponse> {
    let pool = &**db;
    let mut conn = redis.get().await;

    if let Some(user_id) = id.identity() {
        if let Ok(Some(token)) = conn.get(&user_id).await {
            let token = String::from_utf8(token).unwrap();
            let user = get_user_data(&token).await?;

            let query = sqlx::query!(
                "SELECT token, is_banned FROM tokens WHERE user_id = $1 AND email = $2",
                user.id as i64,
                &user.email
            )
            .fetch_optional(pool)
            .await
            .unwrap();

            if let Some(data) = query {
                if data.is_banned {
                    return Ok(HttpResponse::Ok().body("Account has been banned."));
                } else {
                    return Ok(HttpResponse::Ok().body(data.token));
                }
            } else {
                let token = gen_token(
                    user.id,
                    &user.email,
                    &hex::decode(&config.secret_key).unwrap(),
                    &hex::decode(&config.iv_key).unwrap(),
                )
                .unwrap_or_else(|| "null".to_string());

                sqlx::query!(
                    "INSERT INTO tokens (user_id, email, token) VALUES ($1, $2, $3)",
                    user.id as i64,
                    &user.email,
                    &token
                )
                .execute(pool)
                .await
                .unwrap();

                return Ok(HttpResponse::Ok().body(token));
            }
        }
    }

    Ok(HttpResponse::Ok().body("null"))
}

pub async fn logout(id: Identity, redis: web::Data<ConnectionPool>) -> HttpResponse {
    if let Some(user_id) = id.identity() {
        let mut conn = redis.get().await;

        if let Err(why) = conn.del(user_id).await {
            error!("Error deleting cookie: {}", why);
        }
    }

    id.forget();
    HttpResponse::Found()
        .header(header::LOCATION, "/")
        .finish()
}

pub async fn oauth(
    code: web::Query<OAuthCode>,
    id: Identity,
    redis: web::Data<ConnectionPool>,
    config: web::Data<Config>,
) -> ServiceResult<HttpResponse> {
    let code = code.code.to_string();
    let mut conn = redis.get().await;

    let client_id = config.client_id;
    let client_secret = config.client_secret.to_string();
    let redirect_uri = config.redirect_uri.to_string();

    let data = OAuthTokenData {
        client_id,
        client_secret,
        code,
        redirect_uri,
        scope: "identify email guilds".to_string(),
        grant_type: "authorization_code".to_string(),
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(&format!("{}/oauth2/token", API_ENDPOINT))
        .form(&data)
        .send()
        .await?
        .json::<OAuthResponse>()
        .await?;

    let user = get_user_data(&resp.access_token).await?;

    id.remember(user.id.to_string());
    conn.set_and_expire_seconds(&user.id.to_string(), &resp.access_token, resp.expires_in)
        .await
        .unwrap();

    Ok(HttpResponse::Found()
        .header(header::LOCATION, "/")
        .finish())
}

pub async fn get_oauth2(config: web::Data<Config>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "url" : config.oauth2_url,
    }))
}
