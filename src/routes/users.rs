use crate::error::*;
use crate::model::*;
use crate::routes::search::{QueryData, SearchModsResponse};

use std::collections::HashMap;

use actix_identity::Identity;
use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse};

use darkredis::ConnectionPool;
use futures::StreamExt;
use handlebars::Handlebars;
use semver::Version;
use sqlx::postgres::PgPool;

#[derive(Serialize, Deserialize, Debug)]
pub struct MeResponseData {
    roles: u32,
    user_id: u64,
    user_id_string: String,
    is_banned: bool,
    token: String,

    discord: UserResponse,
    mods: Vec<SearchModsResponse>,
    teams: HashMap<String, i32>,
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
    let mut resp_data = None;

    if let Some(user_id) = id.identity() {
        if let Ok(Some(token)) = conn.get(&user_id).await {
            let token = String::from_utf8(token).unwrap();
            let user = get_user_data(&token).await?;

            let query = sqlx::query!("SELECT * FROM tokens WHERE owner_id = $1", user.id as i64)
                .fetch_optional(pool)
                .await?;

            if let Some(data) = query {
                let roles = Roles::from_bits_truncate(data.roles as u32);

                let mut mods = vec![];

                let query = sqlx::query!("SELECT * FROM owners WHERE owner_id = $1", data.owner_id)
                    .fetch_all(pool)
                    .await?;

                for i in query {
                    let mut query = sqlx::query_as!(
                        QueryData,
                        r#"
                            SELECT
                                checksum,
                                name,
                                version,
                                description,
                                keywords,
                                verification as "verification: Verification",
                                downloads,
                                uploaded
                            FROM
                                mods
                            WHERE
                                name = $1
                            ORDER BY
                                uploaded
                                ASC
                        "#,
                        &i.mod_name
                    )
                    .fetch(pool)
                    .boxed();

                    let mut le_mod: Option<SearchModsResponse> = None;

                    while let Some(Ok(value)) = query.next().await {
                        if let Some(ref m) = le_mod {
                            let ver = Version::parse(&value.version).unwrap();
                            let m = Version::parse(&m.version).unwrap();

                            if ver > m {
                                le_mod = Some(SearchModsResponse {
                                    checksum: value.checksum.to_string(),
                                    name: value.name.to_string(),
                                    version: value.version.to_string(),
                                    description: value.description.to_string(),
                                    keywords: value.keywords.clone().unwrap_or_default(),
                                    verification: value.verification.clone().unwrap_or_default(),
                                    downloads: value.downloads,
                                    uploaded: value.uploaded.to_rfc3339(),
                                });
                            }
                        } else {
                            le_mod = Some(SearchModsResponse {
                                checksum: value.checksum.to_string(),
                                name: value.name.to_string(),
                                version: value.version.to_string(),
                                description: value.description.to_string(),
                                keywords: value.keywords.clone().unwrap_or_default(),
                                verification: value.verification.clone().unwrap_or_default(),
                                downloads: value.downloads,
                                uploaded: value.uploaded.to_rfc3339(),
                            });
                        }
                    }

                    if let Some(ref m) = le_mod {
                        mods.push(m.clone());
                    }
                }

                let mut teams = HashMap::new();

                let query = sqlx::query!(
                    "SELECT * FROM team_members WHERE member = $1",
                    data.owner_id
                )
                .fetch_all(pool)
                .await?;

                for i in query {
                    teams.insert(i.team_id.to_string(), i.roles);
                }

                resp_data = Some(MeResponseData {
                    roles: roles.bits(),
                    user_id: data.owner_id as u64,
                    user_id_string: data.owner_id.to_string(),
                    is_banned: data.is_banned,
                    token: token.to_string(),
                    discord: user,
                    mods,
                    teams,
                });
            } else {
                return Ok(HttpResponse::NoContent().finish());
            }
        }
    }

    if let Some(token) = req.headers().get("Authorization") {
        let token = token.to_str().unwrap();

        let query = sqlx::query!("SELECT * FROM tokens WHERE token = $1", token)
            .fetch_optional(pool)
            .await?;

        if let Some(data) = query {
            if let Ok(Some(oauth_token)) = conn.get(&data.owner_id.to_string()).await {
                let user = get_user_data(&String::from_utf8(oauth_token).unwrap()).await?;
                let roles = Roles::from_bits_truncate(data.roles as u32);

                let mut mods = vec![];

                let query = sqlx::query!("SELECT * FROM owners WHERE owner_id = $1", data.owner_id)
                    .fetch_all(pool)
                    .await?;

                for i in query {
                    let mut query = sqlx::query_as!(
                        QueryData,
                        r#"
                            SELECT
                                checksum,
                                name,
                                version,
                                description,
                                keywords,
                                verification as "verification: Verification",
                                downloads,
                                uploaded
                            FROM
                                mods
                            WHERE
                                name = $1
                            ORDER BY
                                uploaded
                                ASC
                        "#,
                        &i.mod_name
                    )
                    .fetch(pool)
                    .boxed();

                    let mut le_mod: Option<SearchModsResponse> = None;

                    while let Some(Ok(value)) = query.next().await {
                        if let Some(ref m) = le_mod {
                            let ver = Version::parse(&value.version).unwrap();
                            let m = Version::parse(&m.version).unwrap();

                            if ver > m {
                                le_mod = Some(SearchModsResponse {
                                    checksum: value.checksum.to_string(),
                                    name: value.name.to_string(),
                                    version: value.version.to_string(),
                                    description: value.description.to_string(),
                                    keywords: value.keywords.clone().unwrap_or_default(),
                                    verification: value.verification.clone().unwrap_or_default(),
                                    downloads: value.downloads,
                                    uploaded: value.uploaded.to_rfc3339(),
                                });
                            }
                        } else {
                            le_mod = Some(SearchModsResponse {
                                checksum: value.checksum.to_string(),
                                name: value.name.to_string(),
                                version: value.version.to_string(),
                                description: value.description.to_string(),
                                keywords: value.keywords.clone().unwrap_or_default(),
                                verification: value.verification.clone().unwrap_or_default(),
                                downloads: value.downloads,
                                uploaded: value.uploaded.to_rfc3339(),
                            });
                        }
                    }

                    if let Some(ref m) = le_mod {
                        mods.push(m.clone());
                    }
                }

                let mut teams = HashMap::new();

                let query = sqlx::query!(
                    "SELECT * FROM team_members WHERE member = $1",
                    data.owner_id
                )
                .fetch_all(pool)
                .await?;

                for i in query {
                    teams.insert(i.team_id.to_string(), i.roles);
                }

                resp_data = Some(MeResponseData {
                    roles: roles.bits(),
                    user_id: data.owner_id as u64,
                    user_id_string: data.owner_id.to_string(),
                    is_banned: data.is_banned,
                    token: token.to_string(),
                    discord: user,
                    mods,
                    teams,
                });
            } else {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "OAuth2 Session Expired"
                })));
            }
        }
    }

    if let Some(data) = resp_data {
        Ok(HttpResponse::Ok().json(data))
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
}
