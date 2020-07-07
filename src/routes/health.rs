use crate::client_interface::ClientInterface;
use crate::types::HealthResponse;

use rocket::State;
use tokio::runtime::Runtime;

/*
* Trow health endpoint
* GET /healthz
*/

#[get("/healthz")]
pub fn healthz(
    ci: State<ClientInterface>,
) -> HealthResponse {
    let request = ci.is_healthy();
    let mut rt = Runtime::new().unwrap();
    
    match rt.block_on(request) {
        Ok(res) => {
            HealthResponse{
                status: String::from("OK"),
                message: res.message,
                is_healthy: res.is_healthy
            }
        },
        Err(error) => {
            HealthResponse{
                status: String::from("Error"),
                message: error.to_string(),
                is_healthy: false
            }
        }
    } 
}
