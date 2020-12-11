#[macro_use]
extern crate serde;

#[macro_use]
extern crate tracing;

pub mod error;
pub mod model;

use crate::error::*;
use crate::model::*;

use actix_identity::{Identity, CookieIdentityPolicy, IdentityService};
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use actix_web::http::header;
use time::Duration;
use handlebars::Handlebars;
use darkredis::ConnectionPool;
use toml::Value;

use tokio::fs::File;
use tokio::prelude::*;

#[derive(Deserialize, Serialize, Debug)]
struct UserInfoLogin {
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct OAuthCode {
    code: String,
}

async fn index(id: Identity, hb: web::Data<Handlebars<'_>>, pool: web::Data<ConnectionPool>) -> HttpResponse {
    let mut conn = pool.get().await;

    if let Some(token) = id.identity() {
        if conn.get(&token).await.unwrap().is_some() {
            let client = reqwest::Client::new();
            let user = client.get(&format!("{}/users/@me", API_ENDPOINT))
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

            return HttpResponse::Ok().body(body)
        }
    }

    HttpResponse::Found().header(header::LOCATION, "/login").finish()
}

async fn login(id: Identity, hb: web::Data<Handlebars<'_>>, pool: web::Data<ConnectionPool>, config: web::Data<Config>) -> HttpResponse {
    let mut conn = pool.get().await;

    if let Some(token) = id.identity() {
        if conn.get(&token).await.unwrap().is_some() {
            return HttpResponse::Found().header(header::LOCATION, "/").finish()
        }
    }

    let auth_url = config.oauth2_url.to_string();

    let data = serde_json::json!({
        "auth_url": auth_url,
    });

    let body = hb.render("discord_login", &data).unwrap();

    HttpResponse::Ok().body(&body)
}

async fn logout(id: Identity, pool: web::Data<ConnectionPool>) -> HttpResponse {
    if let Some(token) = id.identity() {
        let mut conn = pool.get().await;

        let _ = conn.del(token).await;
    }

    id.forget();
    HttpResponse::Found().header(header::LOCATION, "/").finish()
}

async fn oauth(code: web::Query<OAuthCode>, id: Identity, pool: web::Data<ConnectionPool>, config: web::Data<Config>) -> ServiceResult<HttpResponse> {

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
    let resp = match client.post(&format!("{}/oauth2/token", API_ENDPOINT))
        .form(&data)
        .send()
        .await
        .unwrap()
        .json::<OAuthResponse>()
        .await {
            Ok(x) => x,
            Err(why) => {
                return Err(ServiceError::BadRequest(why.to_string()))
            }
        };

    id.remember(resp.access_token.to_string());
    let mut conn = pool.get().await;
    conn.set_and_expire_seconds(&resp.access_token, &resp.refresh_token, resp.expires_in).await.unwrap();

    Ok(HttpResponse::Found().header(header::LOCATION, "/").finish())
}

#[actix_web::main]
#[instrument]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("Config.toml").await?;
    let mut content = String::new();

    file.read_to_string(&mut content).await?;

    let values = content.parse::<Value>().unwrap();

    let values = if cfg!(debug_assertions) {
        values["debug"].as_table().unwrap()
    } else {
        values["release"].as_table().unwrap()
    };

    let config = Config {
        address: values["address"].as_str().unwrap_or("127.0.0.1").to_string(),
        port: values["port"].as_integer().unwrap_or(8000) as u16,
        workers: values["workers"].as_integer().unwrap_or(1) as usize,
        keep_alive: values["keep_alive"].as_integer().unwrap_or(30) as usize,
        log: values["log"].as_str().unwrap_or("actix_web=info").to_string(),

        secret_key: values["secret_key"].as_str().unwrap().to_string(),

        oauth2_url: values["oauth2_url"].as_str().unwrap().to_string(),
        client_id: values["client_id"].as_integer().unwrap() as u64,
        client_secret: values["client_secret"].as_str().unwrap().to_string(),
        redirect_uri: values["redirect_uri"].as_str().unwrap().to_string(),
    };

    std::env::set_var("RUST_LOG", &config.log);
    tracing_subscriber::fmt::init();

    // Handlebars for templating. 
    let mut handlebars = Handlebars::new();
    handlebars.register_templates_directory(".html.hbs", "./templates")?;
    let handlebars_ref = web::Data::new(handlebars);
    
    let pool = ConnectionPool::create("127.0.0.1:6379".into(), None, 2).await?;
    let pool_ref = web::Data::new(pool);

    let config_ref = web::Data::new(config.clone());

    let secret_key = config.secret_key.clone();

    info!("Binding to http://{}:{}", &config.address, &config.port);

    HttpServer::new(move || {
        App::new()
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&hex::decode(&secret_key).unwrap())
                    .name("auth")
                    .path("/")
                    .max_age_time(Duration::days(1))
                    .secure(false),
            ))
            .app_data(handlebars_ref.clone())
            .app_data(pool_ref.clone())
            .app_data(config_ref.clone())
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/login").route(web::get().to(login)))
            .service(web::resource("/logout").to(logout))
            .service(web::resource("/discord/oauth2").route(web::get().to(oauth)))
    })
    .bind(&format!("{}:{}", &config.address, &config.port))?
    .workers(config.workers)
    .keep_alive(config.keep_alive)
    .run()
    .await?;

    Ok(())
}
