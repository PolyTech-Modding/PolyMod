use crate::error::ServiceResult;
use crate::model::Verification;
use actix_web::{web, HttpResponse};
use futures::StreamExt;
use semver::{Version, VersionReq};
use sqlx::PgPool;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryModInfo {
    name: String,
    version: Option<String>,
    #[serde(default = "Verification::lowest")]
    verification: Verification,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetModsResponse {
    name: String,
    version: String,
    description: String,

    #[serde(default = "Verification::lowest")]
    verification: Verification,
    files: Vec<String>,
    downloads: usize,
    uploaded: String,

    repository_git: Option<String>,
    repository_hg: Option<String>,

    authors: Option<Vec<String>>,
    documentation: Option<String>,
    readme: Option<String>,
    homepage: Option<String>,
    license: Option<String>,
    keywords: Option<Vec<String>>,
    // categories,
    build_script: Option<String>,
    metadata: Option<Vec<String>>,
}

pub async fn get_mod(
    data: web::Query<QueryModInfo>,
    db: web::Data<PgPool>,
) -> ServiceResult<HttpResponse> {
    let pool = db.as_ref();
    let mut mods = BTreeMap::new();

    let mut query = sqlx::query!("SELECT checksum, name, version, description, repository_git, repository_hg, authors, documentation, readme, homepage, license, keywords, build_script, native_lib_checksums, dependencies_checksums, metadata, verification as \"verification: Verification\", downloads, uploaded FROM mods WHERE name = $1", &data.name)
        .fetch(pool)
        .boxed();

    while let Some(Ok(values)) = query.next().await {
        if data.verification > values.verification.clone().unwrap_or_default() {
            continue;
        }

        if let Some(ref version) = data.version {
            let v_user = match VersionReq::parse(&version) {
                Ok(x) => x,
                Err(why) => {
                    return Ok(HttpResponse::BadRequest()
                        .json(&format!("Invalid semver provided: {}", why)))
                }
            };
            let v_db = Version::parse(&values.version).unwrap();

            if v_user.matches(&v_db) {
                let native_lib_checksums = values.native_lib_checksums.unwrap_or_default();
                let dependencies_checksums = values.dependencies_checksums.unwrap_or_default();

                let mut files = native_lib_checksums
                    .iter()
                    .chain(dependencies_checksums.iter())
                    .map(|i| format!("/public_api/download/{}", i))
                    .collect::<Vec<String>>();

                files.push(format!("/public_api/download/{}", values.checksum));

                mods.insert(
                    v_db,
                    GetModsResponse {
                        name: values.name,
                        version: values.version,
                        description: values.description,

                        verification: values.verification.unwrap_or_default(),
                        files,
                        downloads: values.downloads as usize,
                        uploaded: values.uploaded.to_rfc3339(),

                        repository_git: values.repository_git,
                        repository_hg: values.repository_hg,

                        authors: values.authors,
                        documentation: values.documentation,
                        readme: values.readme,
                        homepage: values.homepage,
                        license: values.license,
                        keywords: values.keywords,
                        build_script: values.build_script,
                        metadata: values.metadata,
                    },
                );
            }
        } else {
            let v_db = Version::parse(&values.version).unwrap();
            let native_lib_checksums = values.native_lib_checksums.unwrap_or_default();
            let dependencies_checksums = values.dependencies_checksums.unwrap_or_default();

            let mut files = native_lib_checksums
                .iter()
                .chain(dependencies_checksums.iter())
                .map(|i| format!("/public_api/download/{}", i))
                .collect::<Vec<String>>();

            files.push(format!("/public_api/download/{}", values.checksum));

            mods.insert(
                v_db,
                GetModsResponse {
                    name: values.name,
                    version: values.version,
                    description: values.description,

                    verification: values.verification.unwrap_or_default(),
                    files,
                    downloads: values.downloads as usize,
                    uploaded: values.uploaded.to_rfc3339(),

                    repository_git: values.repository_git,
                    repository_hg: values.repository_hg,

                    authors: values.authors,
                    documentation: values.documentation,
                    readme: values.readme,
                    homepage: values.homepage,
                    license: values.license,
                    keywords: values.keywords,
                    build_script: values.build_script,
                    metadata: values.metadata,
                },
            );
        }
    }

    match mods.last_key_value() {
        Some(x) => Ok(HttpResponse::Ok().json(x.1)),
        None => Ok(HttpResponse::NoContent().finish()),
    }
}
