use rocket_contrib::json::{Json, JsonValue};

#[options("/login")]
pub fn options_login() -> Json<JsonValue> {
    Json(json!({}))
}
