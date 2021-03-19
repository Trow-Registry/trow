use rocket_contrib::json::{Json, JsonValue};

#[options("/v2/<repo_name>/tags/list")]
pub fn options_tags(repo_name: String) -> Json<JsonValue> {
    let _ = repo_name;
    Json(json!({}))
}

#[options("/v2/<user>/<repo>/tags/list")]
pub fn options_tags_2level(user: String, repo: String) -> Json<JsonValue> {
    options_tags(format!("{}/{}", user, repo))
}

#[options("/v2/<org>/<user>/<repo>/tags/list")]
pub fn options_tags_3level(org: String, user: String, repo: String) -> Json<JsonValue> {
    options_tags(format!("{}/{}/{}", org, user, repo))
}

#[options("/v2/<fourth>/<org>/<user>/<repo>/tags/list")]
pub fn options_tags_4level(
    fourth: String,
    org: String,
    user: String,
    repo: String,
) -> Json<JsonValue> {
    options_tags(format!("{}/{}/{}/{}", fourth, org, user, repo))
}
