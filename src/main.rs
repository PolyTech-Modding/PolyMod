#![feature(map_first_last)]

#[macro_use]
extern crate serde;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate bitflags;

pub mod error;
pub mod model;
pub mod routes;
pub mod utils;

use crate::model::*;
use crate::routes::*;

use std::env;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_ratelimit::{RateLimiter, RedisStore, RedisStoreActor};
use actix_web::dev::Service;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};

use darkredis::ConnectionPool;
use handlebars::Handlebars;
use sqlx::postgres::{PgPool, PgPoolOptions};
use time::Duration;
use toml::Value;

use tokio::fs::File;
use tokio::prelude::*;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;

    let mut file = File::open("Config.toml").await?;
    let mut content = String::new();

    for i in HEX_BASE.iter() {
        tokio::fs::create_dir_all(&format!("./files/{}", i)).await?;
        for j in HEX_BASE.iter() {
            tokio::fs::create_dir_all(&format!("./files/{}/{}{}", i, i, j)).await?;
        }
    }

    file.read_to_string(&mut content).await?;

    let values = content.parse::<Value>()?;

    let values = if cfg!(debug_assertions) {
        Value::Table(values["debug"].as_table().cloned().unwrap())
    } else {
        Value::Table(values["release"].as_table().cloned().unwrap())
    };

    let config = values.try_into::<Config>()?;

    std::env::set_var("RUST_LOG", &config.log);
    tracing_subscriber::fmt::init();

    let config_ref = web::Data::new(config.clone());

    // Handlebars for templating.
    let mut handlebars = Handlebars::new();
    handlebars.register_templates_directory(".html.hbs", "./templates")?;
    let handlebars_ref = web::Data::new(handlebars);

    // Redis Cache
    let redis = ConnectionPool::create((&config.redis_uri).into(), None, 2).await?;
    let redis_ref = web::Data::new(redis);

    // Redis Rate Limiter
    let store = RedisStore::connect(&format!("redis://{}", &config.redis_uri));

    // Postgresql Database
    let db = PgPoolOptions::new()
        .max_connections(config.workers as u32)
        .connect(&env::var("DATABASE_URL")?)
        .await?;
    let db_ref = web::Data::new(db.clone());

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
            .app_data(redis_ref.clone())
            .app_data(db_ref.clone())
            .app_data(config_ref.clone())
            .wrap(
                // TODO: https://github.com/TerminalWitchcraft/actix-ratelimit/issues/10
                RateLimiter::new(RedisStoreActor::from(store.clone()).start())
                    .with_interval(std::time::Duration::from_secs(120))
                    .with_max_requests(60)
                    .with_identifier(|req| {
                        let key = match req.headers().get("Authorization") {
                            Some(x) => x,
                            None => {
                                if let Some(ip) = &req.headers().get("x-real-ip") {
                                    return Ok(ip.to_str().unwrap().to_string());
                                } else {
                                    return Ok(req.peer_addr().unwrap().to_string());
                                }
                            }
                        };
                        let key = key.to_str().unwrap();
                        Ok(key.to_string())
                    }),
            )
            .service(web::resource("/").route(web::get().to(|| async move {
                actix_files::NamedFile::open("./static/homepage.html").unwrap()
            })))
            .service(web::resource("/search").route(web::get().to(|| async move {
                actix_files::NamedFile::open("./static/search_results.html").unwrap()
            })))
            .service(web::resource("/submit").route(web::get().to(|| async move {
                actix_files::NamedFile::open("./static/submit.html").unwrap()
            })))
            .service(web::resource("/mod").route(web::get().to(get_mod::front_end)))
            .service(web::resource("/user").route(web::get().to(login::index)))
            .service(web::resource("/login").route(web::get().to(login::login)))
            .service(web::resource("/logout").to(login::logout))
            .service(web::resource("/token").route(web::get().to(login::get_token)))
            .service(web::resource("/discord/oauth2").route(web::get().to(login::oauth)))
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .service(actix_files::Files::new("/templates", "./templates").show_files_listing())
            .service(web::resource("/favicon.ico").to(|| async move {
                actix_files::NamedFile::open("./static/PolyTech.svg").unwrap()
            }))
            .service(
                web::scope("/public_api")
                    .service(
                        web::resource("/download/{checksum}")
                            .route(web::get().to(download::download)),
                    )
                    .service(web::resource("/get_mod").route(web::get().to(get_mod::get_mod)))
                    .service(web::resource("/search").route(web::get().to(search::search))),
            )
            .service(
                web::scope("/api")
                    .wrap_fn(|req, srv| {
                        let db = req.app_data::<web::Data<PgPool>>().cloned().unwrap();
                        let token = match &req.headers().get("Authorization") {
                            Some(x) => x.to_str().unwrap().to_string(),
                            None => String::new(),
                        };

                        let fut = srv.call(req);

                        async move {
                            if token.is_empty() {
                                return Err(HttpResponse::Unauthorized()
                                    .body("No Authorization Token provided")
                                    .into());
                            }

                            let query = sqlx::query!(
                                "SELECT is_banned FROM tokens WHERE token = $1",
                                &token,
                            )
                            .fetch_optional(&**db.clone())
                            .await
                            .unwrap();

                            if let Some(data) = query {
                                if data.is_banned {
                                    Err(HttpResponse::Unauthorized()
                                        .body("Banned User")
                                        .into())
                                } else {
                                    let res = fut.await?;
                                    Ok(res)
                                }
                            } else {
                                Err(HttpResponse::Unauthorized()
                                    .body("Invalid Token")
                                    .into())
                            }
                        }
                    })
                    .service(web::resource("/upload").route(web::post().to(upload::upload)))
                    .service(web::resource("/verify").to(verification::verify)),
            )
            .default_service(web::to(|| {
                HttpResponse::NotFound().body("404 - Route Not Found")
            }))
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
    })
    .bind(&format!("{}:{}", &config.address, &config.port))?
    .workers(config.workers)
    .keep_alive(config.keep_alive)
    .run()
    .await?;

    Ok(())
}
