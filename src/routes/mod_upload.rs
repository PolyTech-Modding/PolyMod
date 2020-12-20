use crate::error::ServiceResult;
use crate::model::Config;

use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};

use futures::stream::{self, StreamExt, TryStreamExt};
use semver::Version;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tokio::fs::File;
use tokio::prelude::*;
//use sqlx::postgres::{Postgres, PgTypeInfo, PgArgumentBuffer};
//use sqlx::types::Type;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::Type)]
#[sqlx(rename = "categories")]
pub enum Categories {
    #[serde(rename = "API")]
    #[sqlx(rename = "API")]
    Api,
    Editor,
    Cheat,
    Models,
    Utilities,
    Physics,
    Fun,
    Cosmetic,
}

//#[derive(Serialize, Deserialize, Debug, Clone, sqlx::Type)]
//#[derive(Serialize, Deserialize, Debug, Clone)]
//pub struct CategoriesList(Vec<Categories>);
//
//impl Type<Postgres> for CategoriesList {
//    fn type_info() -> PgTypeInfo {
//        //PgTypeInfo(
//        //    PgType::Custom(
//        //        Arc::new(PgCustomType {
//        //            // fields are private
//        //        })
//        //    )
//        //)
//        <CategoriesList as Type<Postgres>>::type_info()
//    }
//
//    fn compatible(ty: &PgTypeInfo) -> bool {
//        <CategoriesList as Type<Postgres>>::compatible(ty)
//    }
//}
//
//use sqlx::encode::{Encode, IsNull};
//impl Encode<'_, Postgres> for CategoriesList {
//    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
//        <CategoriesList as Encode<Postgres>>::encode(self.clone(), buf)
//    }
//}

#[derive(Serialize, Deserialize, Debug)]
pub struct MiniMod {
    name: String,
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModJsonData {
    name: String,
    version: String,
    description: String,

    repository_git: Option<String>,
    repository_hg: Option<String>,

    #[serde(default)]
    authors: Vec<String>,
    documentation: Option<String>, // URL
    readme: Option<String>,        // the readme contents
    homepage: Option<String>,      // URL
    license: Option<String>,       // OSI Licence or text for license-file
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    categories: Vec<Categories>,
    //categories: CategoriesList,
    build_script: Option<String>, // Build shell script

    #[serde(default)]
    dependencies: Vec<MiniMod>,

    #[serde(default)]
    metadata: Vec<String>, // Extra metadata
}

/// curl -X POST http://localhost:8000/api/upload -i -H 'Authorization: asdasdasd' --form "mod=@mod.zip" --form "data=@data.json"
/// TODO: Check for the owner of the mod before submitting.
pub async fn upload(
    config: web::Data<Config>,
    db: web::Data<PgPool>,
    mut payload: Multipart,
) -> ServiceResult<HttpResponse> {
    let db = &**db.clone();

    let mut contents = String::new();
    let mut checksum = String::with_capacity(64);
    let mut mod_checksum_path = String::with_capacity(81);
    let mut got_file = false;
    let mut filepath = String::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap().to_string();

        if filename.ends_with(".json") {
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap().to_vec();
                contents += &String::from_utf8(data).unwrap();
            }
        } else if filename.ends_with(".zip") {
            if !got_file {
                got_file = true;
            } else {
                if let Err(why) = tokio::fs::remove_file(&filepath).await {
                    error!(
                        "Could not delete file `{}` due to a failed upload.\n{:#?}",
                        &filename, why
                    );
                };

                return Ok(
                    HttpResponse::BadRequest().body("Cannot send more than 1 file to upload")
                );
            }

            filepath = format!(
                "{}/{}",
                config.mods_path,
                sanitize_filename::sanitize(&filename)
            );

            {
                let mut f = File::create(&filepath).await?;

                // Field in turn is stream of *Bytes* object
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    f.write_all(&data).await?;
                }
            }

            checksum = {
                let mut sh = Sha256::default();

                let mut file = File::open(&filepath).await?;
                let mut buffer = [0u8; 1024];

                loop {
                    let n = file.read(&mut buffer).await?;

                    sh.update(&buffer[..n]);
                    if n == 0 || n < 1024 {
                        break;
                    }
                }

                sh.finalize()
                    .iter()
                    .map(|byte| format!("{:02x}", byte))
                    .collect::<String>()
            };

            warn!("{}", &checksum);

            let first = checksum.chars().next().unwrap();
            let second = {
                let mut x = first.to_string();
                x.push(checksum.chars().nth(1).unwrap());
                x
            };

            mod_checksum_path = format!("./files/{}/{}/{}.zip", first, second, checksum);
        }
    }

    dbg!(&contents);
    dbg!(&checksum);

    if contents.is_empty() {
        if let Err(why) = tokio::fs::remove_file(&filepath).await {
            error!(
                "Could not delete file `{}` due to a failed upload.\n{:#?}",
                &mod_checksum_path, why
            );
        };

        return Ok(HttpResponse::BadRequest().body("Missing `data.json` file"));
    }

    if !got_file {
        return Ok(HttpResponse::BadRequest().body("Missing `mod.zip` file"));
    }

    let data: ModJsonData = match serde_json::from_str(&contents) {
        Ok(x) => x,
        Err(why) => {
            if let Err(why) = tokio::fs::remove_file(&filepath).await {
                error!(
                    "Could not delete file `{}` due to a failed upload.\n{:#?}",
                    &mod_checksum_path, why
                );
            };

            return Ok(HttpResponse::BadRequest()
                .body(&format!("Invalid format found on the data json: {}", why)));
        }
    };

    let stream = stream::iter(data.dependencies.iter());

    let dependencies_data = stream
        .filter_map(|i| async move {
            sqlx::query!(
                "SELECT checksum FROM mods WHERE name = $1 AND version = $2",
                i.name,
                i.version,
            )
            .fetch_optional(db)
            .await
            .unwrap()
        })
        .collect::<Vec<_>>()
        .await;

    if data.dependencies.len() != dependencies_data.len() {
        if let Err(why) = tokio::fs::remove_file(&filepath).await {
            error!(
                "Could not delete file `{}` due to a failed upload.\n{:#?}",
                &mod_checksum_path, why
            );
        };

        return Ok(HttpResponse::BadRequest()
            .body("At least one of the depencencies is missing or invalid."));
    }

    if let Err(why) = Version::parse(&data.version) {
        if let Err(why) = tokio::fs::remove_file(&filepath).await {
            error!(
                "Could not delete file `{}` due to a failed upload.\n{:#?}",
                &mod_checksum_path, why
            );
        };

        return Ok(
            HttpResponse::BadRequest().body(&format!("The version is not a valid semver: {}", why))
        );
    }

    let dependencies_checksums = dependencies_data
        .iter()
        .map(|i| i.checksum.to_string())
        .collect::<Vec<String>>();

    // FIX: https://github.com/launchbadge/sqlx/issues/906

    //let query = sqlx::query_unchecked!(
    //    "INSERT INTO mods
    //    (name, version, description, repository_git, repository_hg, authors, documentation, readme, homepage, license, keywords, categories, build_script, dependencies_checksums, metadata, checksum)
    //    VALUES
    //    ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::categories[], $13, $14, $15, $16)",
    //    &data.name,
    //    &data.version,
    //    &data.description,
    //    data.repository_git,
    //    data.repository_hg,
    //    &data.authors,
    //    data.documentation,
    //    data.readme,
    //    data.homepage,
    //    data.license,
    //    &data.keywords,
    //    &data.categories,
    //    //CategoriesList(data.categories.clone()) as _,
    //    data.build_script,
    //    &dependencies_checksums,
    //    &data.metadata,
    //    &checksum,
    //    )
    //    .execute(db)
    //    .await;

    let query = sqlx::query!(
        "INSERT INTO mods
        (name, version, description, repository_git, repository_hg, authors, documentation, readme, homepage, license, keywords, build_script, dependencies_checksums, metadata, checksum)
        VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
        &data.name,
        &data.version,
        &data.description,
        data.repository_git,
        data.repository_hg,
        &data.authors,
        data.documentation,
        data.readme,
        data.homepage,
        data.license,
        &data.keywords,
        data.build_script,
        &dependencies_checksums,
        &data.metadata,
        &checksum,
        )
        .execute(db)
        .await;

    if let Err(why) = query {
        if let Err(why) = tokio::fs::remove_file(&filepath).await {
            error!(
                "Could not delete file `{}` due to a failed upload.\n{:#?}",
                &mod_checksum_path, why
            );
        };

        return Ok(HttpResponse::BadRequest().body(&format!("Database error: {}", why)));
    }

    tokio::fs::rename(&filepath, &mod_checksum_path).await?;

    Ok("ok".into())
}
