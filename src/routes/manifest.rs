use rocket_contrib::json::{Json, JsonValue};

#[options("/v2/<onename>/manifests/<reference>")]
pub fn options_manifest(
    onename: String,
    reference: String,
) -> Json<JsonValue> {
    let _ =onename;
    let _ = reference;
    Json(json!({}))
}

#[options("/v2/<user>/<repo>/manifests/<reference>")]
pub fn options_manifest_2level(
    user: String,
    repo: String,
    reference: String,
) -> Json<JsonValue> {
    let _ = repo;
    let _ = user;
    let _ = reference;
    Json(json!({}))
}


#[options("/v2/<org>/<user>/<repo>/manifests/<reference>")]
pub fn options_manifest_3level(
    org: String,
    user: String,
    repo: String,
    reference: String,
) -> Json<JsonValue> {
    let _  = org; 
    let _  = user; 
    let _  = repo;
    let _ = reference;
    Json(json!({}))
}


#[options("/v2/<fourth>/<org>/<user>/<repo>/manifests/<reference>")]
pub fn options_manifest_4level(
    fourth: String,
    org: String,
    user: String,
    repo: String,
    reference: String,
) -> Json<JsonValue> {
    let _ = fourth;
    let _ = org; 
    let _ = user;
    let _ = repo;
    let _ = reference;

    Json(json!({}))
}