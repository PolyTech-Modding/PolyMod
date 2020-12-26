use crate::error::ServiceResult;
use crate::model::Verification;
use actix_web::{web, HttpResponse};
use futures::StreamExt;
use semver::{Version, VersionReq};
use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use strsim::normalized_levenshtein;
use std::collections::BTreeMap;
use std::fmt::{self, Display};

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryModInfo {
    name: String,
    version: Option<String>,
    #[serde(default = "Verification::lowest")]
    verification: Verification,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
enum SortBy {
    Name,
    Downloads,
    Uploaded,
}

impl Display for SortBy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl Default for SortBy {
    fn default() -> SortBy {
        SortBy::Name
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SearchInfo {
    #[serde(default)]
    query: String,
    //#[serde(default)]
    //category: Categories,
    #[serde(default)]
    keywords_only: bool,
    #[serde(default)]
    names_only: bool,
    #[serde(default)]
    sort_by: SortBy,
    #[serde(default)]
    reverse: bool,
    #[serde(default = "therty")]
    per_page: u8,
    #[serde(default = "Verification::lowest")]
    verification: Verification,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<String>,
}

pub fn one() -> u8 { 1 }
pub fn therty() -> u8 { 30 }


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
    homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keywords: Option<Vec<String>>,
    //#[serde(skip_serializing_if = "Option::is_none")]
    //categories: Option<Vec<Categories>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    build_script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SearchModsResponse {
    checksum: String,
    name: String,
    version: String,
    description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    keywords: Vec<String>,
    verification: Verification,
    downloads: i64,
    uploaded: String,
}

pub async fn get_mod(
    data: web::Query<QueryModInfo>,
    db: web::Data<PgPool>,
) -> ServiceResult<HttpResponse> {
    let pool = db.as_ref();
    let mut mods = BTreeMap::new();

    let mut query = sqlx::query!(r#"
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
            homepage,
            license,
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
        "#, &data.name)
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

#[derive(Debug, Clone)]
struct QueryData {
    checksum: String,
    name: String,
    version: String,
    description: String,
    keywords: Option<Vec<String>>,
    verification: Option<Verification>,
    downloads: i64,
    uploaded: DateTime<Utc>
}

pub async fn search(
    data: web::Query<SearchInfo>,
    db: web::Data<PgPool>,
) -> ServiceResult<HttpResponse> {
    if data.names_only && data.keywords_only {
        return Ok(HttpResponse::NoContent().finish())
    }

    if data.query.len() > 64 {
        return Ok(HttpResponse::BadRequest().json("Max query length exceeded (64)"))
    }

    let pool = db.as_ref();
    let mut mods = vec![];

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
            ORDER BY
                CASE WHEN $2 = 'asc' THEN
                    CASE $1
                        WHEN 'uploaded' THEN uploaded::text
                        WHEN 'name' THEN name::text
                        WHEN 'downloads' THEN downloads::text
                        ELSE uploaded::text
                    END
                ELSE NULL
                END
                ASC,
                CASE WHEN $2 = 'desc' THEN
                    CASE $1
                        WHEN 'uploaded' THEN uploaded::text
                        WHEN 'name' THEN name::text
                        WHEN 'downloads' THEN downloads::text
                        ELSE uploaded::text
                    END
                ELSE NULL
                END
                DESC
        "#,
            &data.sort_by.to_string(),
            {
                if data.reverse {
                    "asc"
                } else {
                    "desc"
                }
            },
        )
            .fetch(pool)
            .boxed();

    let mut cont = if data.after.is_some() { false } else { true };

    while let Some(Ok(values)) = query.next().await {
        if let Some(before) = data.before.clone() {
            if values.checksum == before {
                break
            }
        }

        if let Some(after) = data.after.clone() {
            if values.checksum == after && !cont {
                cont = true;
            }
        }

        if cont {
            if values.verification.clone().unwrap_or_default() < data.verification {
                continue
            }

            let valid: bool = if data.query.is_empty() {
                true
            } else {
                let values_c = values.clone();
                let data_c = data.clone();

                web::block(move || {
                    if false { return Err(()) }

                    let query = data_c.query.split(" ");

                    for i in query {
                        if !data_c.names_only {
                            if !data_c.keywords_only {
                                for j in values_c.description.split(" ") {
                                    if normalized_levenshtein(&i, &j) > 0.9 || j.contains(&i) {
                                        return Ok(true)
                                    }
                                }
                            }

                            for j in values_c.keywords.clone().unwrap_or_default() {
                                if normalized_levenshtein(&i, &j) > 0.9 || j.contains(&i) {
                                    return Ok(true)
                                }
                            }
                        }

                        if !data_c.keywords_only {
                            if normalized_levenshtein(&i, &values_c.name) > 0.9 || values_c.name.contains(&i) {
                                return Ok(true)
                            }
                        }
                    }

                    Ok(false)
                }).await.unwrap_or_default()
            };

            if valid {
                mods.push(
                    SearchModsResponse {
                        checksum: values.checksum.to_string(),
                        name: values.name.to_string(),
                        version: values.version.to_string(),
                        description: values.description.to_string(),
                        keywords: values.keywords.clone().unwrap_or_default(),
                        verification: values.verification.clone().unwrap_or_default(),
                        downloads: values.downloads,
                        uploaded: values.uploaded.to_rfc3339(),
                    }
                );
            }
        }

        if mods.len() >= data.per_page as usize {
            break
        }
    }

    if mods.is_empty() {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::Ok().json(mods))
    }
}
