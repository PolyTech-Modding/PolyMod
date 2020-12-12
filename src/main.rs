#[macro_use]
extern crate serde;

#[macro_use]
extern crate tracing;

pub mod error;
pub mod model;
pub mod utils;
pub mod routes;

use crate::model::Config;
use crate::routes::*;

use std::env;

use actix_web::{middleware, web, App, HttpServer};
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_ratelimit::{RateLimiter, RedisStore, RedisStoreActor};

use time::Duration;
use handlebars::Handlebars;
use darkredis::ConnectionPool;
use toml::Value;
use sqlx::postgres::PgPoolOptions;

use tokio::fs::File;
use tokio::prelude::*;

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
        iv_key: values["iv_key"].as_str().unwrap().to_string(),

        oauth2_url: values["oauth2_url"].as_str().unwrap().to_string(),
        client_id: values["client_id"].as_integer().unwrap() as u64,
        client_secret: values["client_secret"].as_str().unwrap().to_string(),
        redirect_uri: values["redirect_uri"].as_str().unwrap().to_string(),

        redis_uri: values["redis_uri"].as_str().unwrap_or("127.0.0.1:6379").to_string(),
    };

    std::env::set_var("RUST_LOG", &config.log);
    tracing_subscriber::fmt::init();

    // Handlebars for templating. 
    let mut handlebars = Handlebars::new();
    handlebars.register_templates_directory(".html.hbs", "./templates")?;
    let handlebars_ref = web::Data::new(handlebars);
    
    let redis = ConnectionPool::create((&config.redis_uri).into(), None, 2).await?;
    let redis_ref = web::Data::new(redis);
    let store = RedisStore::connect(&format!("redis://{}", &config.redis_uri));

    let db = PgPoolOptions::new()
        .max_connections(config.workers as u32)
        .connect(&env::var("DATABASE_URL")?).await?;
    let db_ref = web::Data::new(db);

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
            .app_data(redis_ref.clone())
            .app_data(db_ref.clone())
            .app_data(config_ref.clone())
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            .wrap(
                // TODO: https://github.com/TerminalWitchcraft/actix-ratelimit/issues/10
                RateLimiter::new(
                RedisStoreActor::from(store.clone()).start())
                    .with_interval(std::time::Duration::from_secs(120))
                    .with_max_requests(10)
                    .with_identifier(|req| {
                        let key = match req.headers().get("Authorization") {
                            Some(x) => x,
                            None => {
                                if let Some(ip) = &req.headers().get("x-real-ip") {
                                    return Ok(ip.to_str().unwrap().to_string())
                                } else {
                                    return Ok(req.peer_addr().unwrap().to_string())
                                }
                            },
                        };
                        let key = key.to_str().unwrap();
                        Ok(key.to_string())
                    })
            )
            .wrap(
                RateLimiter::new(
                RedisStoreActor::from(store.clone()).start())
                    .with_interval(std::time::Duration::from_secs(120))
                    .with_max_requests(60)
            )
            .service(web::resource("/").route(web::get().to(login::index)))
            .service(web::resource("/login").route(web::get().to(login::login)))
            .service(web::resource("/logout").to(login::logout))
            .service(web::resource("/token").route(web::get().to(login::get_token)))
            .service(web::resource("/discord/oauth2").route(web::get().to(login::oauth)))
    })
    .bind(&format!("{}:{}", &config.address, &config.port))?
    .workers(config.workers)
    .keep_alive(config.keep_alive)
    .run()
    .await?;

    Ok(())
}
