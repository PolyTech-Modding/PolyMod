use crate::error::*;
use crate::model::*;
use crate::utils::tokens::gen_token;
use actix_web::{web, HttpResponse};
use actix_identity::Identity;
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTeamData {
    name: String,
}

pub async fn create_team(
    id: Identity,
    data: web::Form<CreateTeamData>,
    db: web::Data<PgPool>,
) -> ServiceResult<HttpResponse> {
    let pool = &**db;

    let query = sqlx::query!("SELECT * FROM teams WHERE name = $1", &data.name)
        .fetch_optional(pool)
        .await?;

    if query.is_some() {
        return Ok(HttpResponse::BadRequest().body("A team with the same name already exists"));
    }

    if let Some(user_id) = id.identity() {
        let query = sqlx::query!("INSERT INTO teams (name) VALUES ($1) RETURNING id", &data.name)
            .fetch_one(pool)
            .await?;

        sqlx::query!(
            "INSERT INTO team_members (team_id, member, roles) VALUES ($1, $2, $3)",
            query.id,
            user_id.parse::<i64>().unwrap(),
            TeamRoles::OWNER.bits() as i32,
        )
        .execute(pool)
        .await?;

        Ok(HttpResponse::Ok().body(&format!("Created the team `{}` with the id `{}`", &data.name, query.id)))
    } else {
        Ok(HttpResponse::BadRequest().body("No valid identity provided"))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTokenData {
    id: u32,
}

pub async fn get_token(
    id: Identity,
    data: web::Query<GetTokenData>,
    db: web::Data<PgPool>,
    config: web::Data<Config>,
) -> ServiceResult<HttpResponse> {
    let pool = &**db;

    if let Some(user_id) = id.identity() {
        let user_id = user_id.parse::<i64>().unwrap();
        let query = sqlx::query!(
            "SELECT * FROM team_members WHERE team_id = $1 AND member = $2",
            data.id as i32,
            user_id,
        )
        .fetch_optional(pool)
        .await?;

        if query.is_some() {
            let query = sqlx::query!(
                "SELECT token, is_banned FROM tokens WHERE owner_id = $1 AND is_team = true",
                data.id as i64,
            )
            .fetch_optional(pool)
            .await?;

            if let Some(data) = query {
                if data.is_banned {
                    return Ok(HttpResponse::Ok().body("Team has been banned."));
                } else {
                    return Ok(HttpResponse::Ok().body(data.token));
                }
            } else {
                let token = gen_token(
                    user_id as u64,
                    &format!("{}@local", user_id),
                    &hex::decode(&config.secret_key).unwrap(),
                    &hex::decode(&config.iv_key).unwrap(),
                )
                .unwrap_or_else(|| "null".to_string());

                sqlx::query!(
                    "INSERT INTO tokens (owner_id, email, token, is_team) VALUES ($1, $2, $3, true)",
                    data.id as i64,
                    &format!("{}@local", user_id),
                    &token
                )
                .execute(pool)
                .await?;

                return Ok(HttpResponse::Ok().body(token));
            }
        }
    }

    Ok(HttpResponse::Ok().body("null"))
}
