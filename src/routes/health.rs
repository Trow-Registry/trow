/*
* Trow health endpoint
* GET /healthz
*/
use crate::client_interface::ClientInterface;
// use crate::TrowConfig;
use rocket::http::{Header, Status};
use crate::types::HealthResponse;
use rocket::State;
use rocket_contrib::json::{Json};

use tokio::runtime::Runtime;

#[get("/healthz")]
pub fn healthz(
    ci: State<ClientInterface>,
    // tc: State<TrowConfig>,
) -> Json<HealthResponse> {
    let request = ci.is_healthy();
    let mut rt = Runtime::new().unwrap();

    match rt.block_on(request) {
        Ok(res) => {
            println!("responding {:?}",res);
            Json(HealthResponse{
                status: "OK".to_string(),
                message: res.message,
                is_healthy: res.is_healthy
            })
        },
        Err(error) => {
            Json(HealthResponse{
                status: "Error".to_string(),
                message: error.to_string(),
                is_healthy: false
            })
        }
    } 
}
