use rocket_contrib::json::{Json, JsonValue};

#[options("/v2/<name_repo>/blobs/<digest>")]
pub fn options_blob(
    name_repo: String,
    digest: String,
) -> Json<JsonValue> {
    let _ = name_repo;
    let _ = digest;
    Json(json!({}))
}

#[options("/v2/<name>/<repo>/blobs/<digest>")]
pub fn options_blob_2level(
    name: String,
    repo: String,
    digest: String,
) -> Json<JsonValue> {
    options_blob(format!("{}/{}", name, repo), digest)
}


#[options("/v2/<org>/<name>/<repo>/blobs/<digest>")]
pub fn options_blob_3level(
    org: String,
    name: String,
    repo: String,
    digest: String,
) -> Json<JsonValue> {
    options_blob(format!("{}/{}/{}", org, name, repo), digest)
}


#[options("/v2/<fourth>/<org>/<name>/<repo>/blobs/<digest>")]
pub fn options_blob_4level(
    fourth: String,
    org: String,
    name: String,
    repo: String,
    digest: String,
) -> Json<JsonValue> {
    
    options_blob(
        format!("{}/{}/{}/{}", fourth, org, name, repo),digest
    )
}