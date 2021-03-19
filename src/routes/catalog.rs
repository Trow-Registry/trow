use rocket_contrib::json::{Json, JsonValue};

#[options("/v2/_catalog")]
pub fn options_catalog() -> Json<JsonValue> {
    Json(json!({}))
}
