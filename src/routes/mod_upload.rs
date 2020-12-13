use crate::model::Config;

use actix_web::{web, HttpResponse, Result};
use actix_multipart::Multipart;

use futures::{StreamExt, TryStreamExt};
use crypto::{digest::Digest, sha2::Sha256};
use tokio::fs::File;
use tokio::prelude::*;
use sqlx::PgPool;

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadJsonData {
    test: String,
}

pub async fn upload(
    config: web::Data<Config>,
    db: web::Data<PgPool>,
    mut payload: Multipart
) -> Result<HttpResponse> {
    let mut contents = String::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();

        if filename.ends_with(".json") {
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap().to_vec();
                contents += &String::from_utf8(data).unwrap();
            }
        } else if filename.ends_with(".zip") {
            let filepath = format!("{}/{}", config.mods_path, sanitize_filename::sanitize(&filename));
            {
                let mut f = File::create(&filepath).await?;

                // Field in turn is stream of *Bytes* object
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    f.write_all(&data).await?;
                }
            }

            let checksum = {
                let mut buffer = Vec::new();
                let mut f = File::open(&filepath).await?;

                f.read(&mut buffer).await?;

                let mut hasher = Sha256::new();
                hasher.input(&buffer);
                hasher.result_str()
            };

            let first = checksum.chars().next().unwrap();
            let second = {
                let mut x = first.to_string();
                x.push(checksum.chars().nth(1).unwrap());
                x
            };

            tokio::fs::rename(&filepath, &format!("./files/{}/{}/{}.zip", first, second, checksum)).await?;
        }
    }

    dbg!(&contents);

    Ok("ok".into())
}
