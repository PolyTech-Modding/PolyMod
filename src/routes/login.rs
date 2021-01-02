use crate::error::*;
use crate::model::*;
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
) -> HttpResponse {
    let mut conn = redis.get().await;

    if let Some(token) = id.identity() {
        if conn.get(&token).await.unwrap().is_some() {
            let client = reqwest::Client::new();
            let user = client
                .get(&format!("{}/users/@me", API_ENDPOINT))
                .bearer_auth(&token)
                .send()
                .await
                .unwrap()
                .json::<UserResponse>()
                .await
                .unwrap();

            let data = serde_json::json!({
                "name": user.username,
                "discriminator": user.discriminator,
            });

            let body = hb.render("discord_user", &data).unwrap();

            return HttpResponse::Ok().body(body);
        }
    }

    HttpResponse::Found()
        .header(header::LOCATION, "/login")
        .finish()
}

pub async fn login(
    id: Identity,
    hb: web::Data<Handlebars<'_>>,
    redis: web::Data<ConnectionPool>,
    config: web::Data<Config>,
) -> HttpResponse {
    let mut conn = redis.get().await;

    if let Some(token) = id.identity() {
        if conn.get(&token).await.unwrap().is_some() {
            return HttpResponse::Found()
                .header(header::LOCATION, "/user")
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
) -> HttpResponse {
    let pool = &**db;
    let mut conn = redis.get().await;

    if let Some(token) = id.identity() {
        if conn.get(&token).await.unwrap().is_some() {
            let client = reqwest::Client::new();
            let user = client
                .get(&format!("{}/users/@me", API_ENDPOINT))
                .bearer_auth(&token)
                .send()
                .await
                .unwrap()
                .json::<UserResponse>()
                .await
                .unwrap();

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
                    return HttpResponse::Ok().body("Account has been banned.");
                } else {
                    return HttpResponse::Ok().body(data.token);
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

                return HttpResponse::Ok().body(token);
            }
        }
    }

    HttpResponse::Ok().body("null")
}

pub async fn logout(id: Identity, redis: web::Data<ConnectionPool>) -> HttpResponse {
    if let Some(token) = id.identity() {
        let mut conn = redis.get().await;

        let _ = conn.del(token).await;
    }

    id.forget();
    HttpResponse::Found()
        .header(header::LOCATION, "/user")
        .finish()
}

pub async fn oauth(
    code: web::Query<OAuthCode>,
    id: Identity,
    redis: web::Data<ConnectionPool>,
    config: web::Data<Config>,
) -> ServiceResult<HttpResponse> {
    let code = code.code.to_string();

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
    let resp = match client
        .post(&format!("{}/oauth2/token", API_ENDPOINT))
        .form(&data)
        .send()
        .await
        .unwrap()
        .json::<OAuthResponse>()
        .await
    {
        Ok(x) => x,
        Err(why) => return Err(ServiceError::BadRequest(why.to_string())),
    };

    id.remember(resp.access_token.to_string());
    let mut conn = redis.get().await;
    conn.set_and_expire_seconds(&resp.access_token, &resp.refresh_token, resp.expires_in)
        .await
        .unwrap();

    Ok(HttpResponse::Found()
        .header(header::LOCATION, "/user")
        .finish())
}

pub async fn get_oauth2(config: web::Data<Config>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "url" : config.oauth2_url,
    }))
}
