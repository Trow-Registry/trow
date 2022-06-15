use crate::client_interface::ClientInterface;
use crate::registry_interface::admission::AdmissionValidation;
use crate::TrowConfig;

use anyhow::Result;
use k8s_openapi::api::core::v1::Pod;
use kube::core::admission::{AdmissionRequest, AdmissionResponse, AdmissionReview};
use kube::core::DynamicObject;
use rocket::post;
use rocket::serde::json::Json;

// Kubernetes webhooks for admitting images
#[post("/validate-image", data = "<image_data>")]
pub async fn validate_image(
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    image_data: Json<AdmissionReview<Pod>>,
) -> Json<AdmissionReview<DynamicObject>> {
    let req: Result<AdmissionRequest<_>, _> = image_data.into_inner().try_into();

    Json::from(match req {
        Err(e) => {
            AdmissionResponse::invalid(format!("Invalid admission request: {}", e)).into_review()
        }
        Ok(req) => ci
            .validate_admission(&req, &tc.service_name)
            .await
            .into_review(),
    })
}
