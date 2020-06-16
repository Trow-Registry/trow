/*
* Trow health endpoint
* GET /healthz
*/
use crate::client_interface::ClientInterface;
use crate::TrowConfig;
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};

use tokio::runtime::Runtime;


#[get("/healthz")]
pub fn healthz(
    ci: State<ClientInterface>,
    tc: State<TrowConfig>,
) -> JsonValue {
    let re = ci.is_healthy();
    let mut rt = Runtime::new().unwrap();
    rt.block_on(re).ok();
    json!({ "status": "ok" })
}
