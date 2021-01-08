use crate::error::*;
use crate::model::*;
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
