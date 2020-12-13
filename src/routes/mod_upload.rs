use crate::model::Config;

use actix_web::{web, HttpResponse, Result};
use futures::{StreamExt, TryStreamExt};
use actix_multipart::Multipart;

use tokio::fs::File;
use tokio::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadJsonData {
    test: String,
}

pub async fn upload(
    config: web::Data<Config>,
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
        } else {
            let filepath = format!("{}/{}", config.mods_path, sanitize_filename::sanitize(&filename));

            let mut f = File::create(filepath).await.unwrap();

            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                f.write_all(&data).await?;
            }
        }
    }

    dbg!(&contents);

    Ok("ok".into())
}
