use rocket::{get};
use rocket::State;
use rocket_contrib::json::{Json};
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
) ->  Json<ReadinessResponse> {
    let request = ci.is_ready();
    let mut rt = Runtime::new().unwrap();
    match rt.block_on(request) {
        Ok(response) => {
            Json(ReadinessResponse{
                message: response.message,
                status: response.status,
                is_ready: response.is_ready
            })
        },
        Err(error) => {
            Json(ReadinessResponse{
                message: "Error".to_string(),
                status: error.to_string(),
                is_ready: false
            })
        }
    }    
}
