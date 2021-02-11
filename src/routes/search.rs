use crate::error::ServiceResult;
use crate::model::Verification;
use actix_web::{web, HttpResponse};
use futures::StreamExt;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::fmt::{self, Display};
use strsim::normalized_levenshtein;

#[derive(Debug, Clone)]
pub struct QueryData {
    pub checksum: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub keywords: Option<Vec<String>>,
    pub verification: Option<Verification>,
    pub downloads: i64,
    pub uploaded: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SearchModsResponse {
    pub checksum: String,
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    pub verification: Verification,
    pub downloads: i64,
    pub uploaded: String,
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
    #[serde(default = "thirty")]
    per_page: u8,
    #[serde(default)]
    verification: Verification,

    #[serde(skip_serializing_if = "Option::is_none")]
    before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<String>,
}

pub fn one() -> u8 {
    1
}
pub fn thirty() -> u8 {
    30
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

pub async fn search(
    data: web::Query<SearchInfo>,
    db: web::Data<PgPool>,
) -> ServiceResult<HttpResponse> {
    if data.names_only && data.keywords_only {
        return Ok(HttpResponse::NoContent().finish());
    }

    if data.query.len() > 64 {
        return Ok(HttpResponse::BadRequest().json("Max query length exceeded (64)"));
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

    let mut cont = data.after.is_none();

    while let Some(Ok(values)) = query.next().await {
        if let Some(before) = data.before.clone() {
            if values.checksum == before {
                break;
            }
        }

        if let Some(after) = data.after.clone() {
            if values.checksum == after && !cont {
                cont = true;
            }
        }

        if cont {
            if values.verification.clone().unwrap_or_default() < data.verification {
                continue;
            }

            let valid: bool = if data.query.is_empty() {
                true
            } else {
                let values_c = values.clone();
                let data_c = data.clone();

                web::block(move || {
                    if false {
                        return Err(());
                    }

                    let query = data_c.query.split(' ');

                    for i in query {
                        if !data_c.names_only {
                            if !data_c.keywords_only {
                                for j in values_c.description.split(' ') {
                                    if normalized_levenshtein(&i, &j) > 0.9 || j.contains(&i) {
                                        return Ok(true);
                                    }
                                }
                            }

                            for j in values_c.keywords.clone().unwrap_or_default() {
                                if normalized_levenshtein(&i, &j) > 0.9 || j.contains(&i) {
                                    return Ok(true);
                                }
                            }
                        }

                        if !data_c.keywords_only
                            && (normalized_levenshtein(&i, &values_c.name) > 0.9
                                || values_c.name.contains(&i))
                        {
                            return Ok(true);
                        }
                    }

                    Ok(false)
                })
                .await
                .unwrap_or_default()
            };

            if valid {
                mods.push(SearchModsResponse {
                    checksum: values.checksum.to_string(),
                    name: values.name.to_string(),
                    version: values.version.to_string(),
                    description: values.description.to_string(),
                    keywords: values.keywords.clone().unwrap_or_default(),
                    verification: values.verification.clone().unwrap_or_default(),
                    downloads: values.downloads,
                    uploaded: values.uploaded.to_rfc3339(),
                });
            }
        }

        if mods.len() >= data.per_page as usize {
            break;
        }
    }

    if mods.is_empty() {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::Ok().json(mods))
    }
}
