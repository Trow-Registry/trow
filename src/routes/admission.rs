use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Json, State};
use k8s_openapi::api::core::v1::Pod;
use kube::core::admission::{AdmissionRequest, AdmissionResponse, AdmissionReview};
use kube::core::DynamicObject;

use crate::TrowServerState;

pub async fn validate_image(
    State(state): State<Arc<TrowServerState>>,
    Json(image_data): Json<AdmissionReview<Pod>>,
) -> Json<AdmissionReview<DynamicObject>> {
    let req: Result<AdmissionRequest<_>, _> = image_data.try_into();

    Json::from(match req {
        Err(e) => {
            AdmissionResponse::invalid(format!("Invalid admission request: {:#}", e)).into_review()
        }
        Ok(req) => state.registry.validate_admission(&req).await.into_review(),
    })
}

pub async fn mutate_image(
    State(state): State<Arc<TrowServerState>>,
    Json(image_data): Json<AdmissionReview<Pod>>,
) -> Json<AdmissionReview<DynamicObject>> {
    let req: Result<AdmissionRequest<_>, _> = image_data.try_into();

    let res = match req {
        Err(e) => {
            AdmissionResponse::invalid(format!("Invalid admission request: {:#}", e)).into_review()
        }
        Ok(req) => state
            .registry
            .mutate_admission(&req, &state.config.service_name)
            .await
            .into_review(),
    };

    Json::from(res)
}
