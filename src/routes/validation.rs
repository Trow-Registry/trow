use crate::client_interface::ClientInterface;
use crate::registry_interface::validation::{self, Validation};

use crate::types::AdmissionReview;
use crate::TrowConfig;
use rocket::serde::json::Json;

//Kubernetes webhooks for admitting images
//Update to use rocket_contrib::Json
//Just using String for debugging
#[post("/validate-image", data = "<image_data>")]
pub async fn validate_image(
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    image_data: Json<AdmissionReview>,
) -> Json<AdmissionReview> {
    /*
     * The return type is a little complicated. Always return a 200 including for disallowed images. The JSON is an
     * AdmissionReview object with an AdmissionResponse entry. The object sent to this endpoint can be reused, or
     * a new created with the same UID.
     *
     * The docs on this stuff is a bit lacking, it's easiest to refer to the Go code in kubernetes/api.
     */
    let mut resp_data = image_data.clone();
    match image_data.0.request {
        Some(req) => match ci.validate_admission(&req, &tc.host_names).await {
            Ok(res) => {
                resp_data.response = Some(res);
                Json(resp_data)
            }
            Err(e) => {
                resp_data.response = Some(validation::AdmissionResponse {
                    uid: req.uid.clone(),
                    allowed: false,
                    status: Some(validation::Status {
                        status: "Failure".to_owned(),
                        message: Some(format!("Internal Error {:?}", e)),
                        code: None,
                    }),
                });
                Json(resp_data)
            }
        },

        None => {
            resp_data.response = Some(validation::AdmissionResponse {
                uid: "UNKNOWN".to_string(),
                allowed: false,
                status: Some(validation::Status {
                    status: "Failure".to_owned(),
                    message: Some("No request found in review object".to_owned()),
                    code: None,
                }),
            });

            Json(resp_data)
        }
    }
}
