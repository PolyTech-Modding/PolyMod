use crate::error::{ServiceResult, ServiceError};
use actix_files::NamedFile;
use actix_web::web;
use sqlx::PgPool;

pub async fn download(
    checksum: web::Path<String>,
    db: web::Data<PgPool>,
) -> ServiceResult<NamedFile> {
    match hex::decode(&*checksum) {
        Ok(x) if x.len() != 32 => {
            return Err(ServiceError::BadRequest("Invalid checksum length".into()))
        }
        Err(_) => return Err(ServiceError::BadRequest("Invalid characters found in checksum".into())),
        _ => (),
    }

    sqlx::query!(
        "UPDATE mods SET downloads = downloads + 1 WHERE checksum = $1",
        &*checksum
    )
    .execute(&**db)
    .await?;

    let first = checksum.chars().next().unwrap();
    let second = {
        let mut x = first.to_string();
        x.push(checksum.chars().nth(1).unwrap());
        x
    };

    let mod_checksum_path = format!("./files/{}/{}/{}.zip", first, second, checksum);

    Ok(NamedFile::open(&mod_checksum_path)?)
}
