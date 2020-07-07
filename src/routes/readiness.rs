use rocket::{get};
use rocket::State;
use crate::client_interface::ClientInterface;
use tokio::runtime::Runtime;
use crate::types::ReadinessResponse;

/*
* Trow readiness endpoint
* GET /readiness
*/

#[get("/readiness")]
pub fn readiness(
    ci: State<ClientInterface>,
) -> ReadinessResponse {
    let request = ci.is_ready();
    let mut rt = Runtime::new().unwrap();
    match rt.block_on(request) {
        Ok(response) => {
            ReadinessResponse{
                message: response.message,
                status: response.status,
                is_ready: response.is_ready
            }
        },
        Err(error) => {
            ReadinessResponse{
                message: String::from("Error"),
                status: error.to_string(),
                is_ready: false
            }
        }
    }    
}
