use crate::error::{ServiceError, ServiceResult};
use crate::model::Verification;
use actix_web::{web, HttpResponse};
use futures::StreamExt;
use handlebars::Handlebars;
use semver::{Version, VersionReq};
use sqlx::PgPool;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryModInfo {
    name: String,
    version: Option<String>,
    #[serde(default)]
    verification: Verification,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModData {
    data: GetModsResponse,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GetModsResponse {
    name: String,
    version: String,
    description: String,

    #[serde(default)]
    verification: Verification,
    files: Vec<String>,
    downloads: usize,
    uploaded: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    repository_git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repository_hg: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    documentation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    readme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    readme_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    license_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keywords: Option<Vec<String>>,
    //#[serde(skip_serializing_if = "Option::is_none")]
    //categories: Option<Vec<Categories>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    build_script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Vec<String>>,
}

pub async fn get_mods_data(data: &QueryModInfo, pool: &PgPool) -> ServiceResult<GetModsResponse> {
    let mut mods = BTreeMap::new();

    let mut query = sqlx::query!(
        r#"
        SELECT
            checksum,
            name,
            version,
            description,
            repository_git,
            repository_hg,
            authors,
            documentation,
            readme,
            readme_filename,
            license,
            license_filename,
            homepage,
            keywords,
            build_script,
            native_lib_checksums,
            dependencies_checksums,
            metadata,
            verification as "verification: Verification",
            downloads,
            uploaded
        FROM
            mods
        WHERE
            name = $1
        "#,
        &data.name
    )
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
                    return Err(ServiceError::BadRequest(format!(
                        "Invalid semver provided: {}",
                        why
                    )))
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
                        readme_filename: values.readme_filename,
                        license: values.license,
                        license_filename: values.license_filename,
                        homepage: values.homepage,
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
                    readme_filename: values.readme_filename,
                    license: values.license,
                    license_filename: values.license_filename,
                    homepage: values.homepage,
                    keywords: values.keywords,
                    build_script: values.build_script,
                    metadata: values.metadata,
                },
            );
        }
    }

    match mods.last_key_value() {
        Some(x) => Ok(x.1.clone()),
        None => Err(ServiceError::NoContent),
    }
}

pub async fn get_mod(
    data: web::Query<QueryModInfo>,
    db: web::Data<PgPool>,
) -> ServiceResult<HttpResponse> {
    let pool = db.as_ref();
    let mod_data = get_mods_data(&*data, pool).await?;

    Ok(HttpResponse::Ok().json(mod_data))
}

pub async fn front_end(
    data: web::Query<QueryModInfo>,
    db: web::Data<PgPool>,
    hb: web::Data<Handlebars<'_>>,
) -> ServiceResult<HttpResponse> {
    let pool = db.as_ref();
    let data = get_mods_data(&*data, pool).await?;

    let mod_data = ModData { data };

    let body = hb.render("mod_view_page", &mod_data).unwrap();

    Ok(HttpResponse::Ok().body(&body))
}
