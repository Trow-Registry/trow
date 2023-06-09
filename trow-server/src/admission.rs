use anyhow::Result;
use json_patch::{Patch, PatchOperation, ReplaceOperation};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Status};

use crate::image::RemoteImage;
use crate::server::trow_server::admission_controller_server::AdmissionController;
use crate::server::trow_server::{AdmissionRequest, AdmissionResponse};
use crate::server::TrowServer;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageValidationConfig {
    pub default: String,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

fn check_image_is_allowed(
    raw_image_ref: &str,
    config: &ImageValidationConfig,
) -> (bool, &'static str) {
    let image = match RemoteImage::try_from_str(raw_image_ref) {
        Ok(image) => image,
        Err(_) => return (false, "Invalid image reference"),
    };
    let image_ref = image.get_ref();
    let mut is_allowed = match config.default.as_str() {
        "Allow" => true,
        "Deny" => false,
        _ => {
            warn!("Invalid default image validation config: `{}`. Should be `Allow` or `Deny`. Default to `Deny`.", config.default);
            false
        }
    };
    let mut match_len = 0;
    let mut match_reason =
        "Image is neither explicitly allowed nor denied (using default behavior)";

    for m in config.deny.iter() {
        if m.len() > match_len && image_ref.starts_with(m) {
            is_allowed = false;
            match_len = m.len();
            match_reason = "Image explicitly denied";
        }
    }

    for m in config.allow.iter() {
        if m.len() > match_len && image_ref.starts_with(m) {
            is_allowed = true;
            match_len = m.len();
            match_reason = "Image explicitly allowed";
        }
    }

    (is_allowed, match_reason)
}

#[tonic::async_trait]
impl AdmissionController for TrowServer {
    async fn validate_admission(
        &self,
        ar: Request<AdmissionRequest>,
    ) -> Result<Response<AdmissionResponse>, Status> {
        if self.image_validation_config.is_none() {
            return Ok(Response::new(AdmissionResponse {
                patch: None,
                is_allowed: false,
                reason: "Missing image validation config !".to_string(),
            }));
        }
        let ar = ar.into_inner();
        let mut valid = true;
        let mut reasons = Vec::new();

        for image_raw in ar.images {
            let (v, r) =
                check_image_is_allowed(&image_raw, self.image_validation_config.as_ref().unwrap());
            if !v {
                valid = false;
                reasons.push(format!("{image_raw}: {r}"));
                break;
            }
        }

        let ar = AdmissionResponse {
            patch: None,
            is_allowed: valid,
            reason: reasons.join("; "),
        };
        Ok(Response::new(ar))
    }

    async fn mutate_admission(
        &self,
        ar: Request<AdmissionRequest>,
    ) -> Result<Response<AdmissionResponse>, Status> {
        let ar = ar.into_inner();
        let mut patch_operations = Vec::<PatchOperation>::new();

        for (raw_image, image_path) in ar.images.iter().zip(ar.image_paths.iter()) {
            let image = match RemoteImage::try_from_str(raw_image) {
                Ok(image) => image,
                Err(_) => continue,
            };

            for cfg in self.proxy_registry_config.iter() {
                if image.get_host() == cfg.host {
                    info!(
                        "mutate_admission: proxying image {} to {}",
                        raw_image, cfg.alias
                    );
                    let im = RemoteImage::new(
                        &ar.host_name,
                        format!("f/{}/{}", cfg.alias, image.get_repo()),
                        image.reference.clone(),
                    );
                    patch_operations.push(PatchOperation::Replace(ReplaceOperation {
                        path: image_path.clone(),
                        value: serde_json::Value::String(im.get_ref()),
                    }));
                    break;
                }
                info!("mutate_admission: could not proxy image {}", raw_image);
            }
        }
        let patch = Patch(patch_operations);
        let patch_vec = Some(
            serde_json::to_vec(&patch)
                .map_err(|e| Status::internal(format!("Could not serialize patch: {}", e)))?,
        );

        return Ok(Response::new(AdmissionResponse {
            patch: patch_vec,
            is_allowed: true,
            reason: "".to_string(),
        }));
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_check() {
        let cfg = ImageValidationConfig {
            default: "Deny".to_string(),
            allow: vec!["localhost:8080/".into(), "quay.io/".into()],
            deny: vec![],
        };

        let (v, _) = check_image_is_allowed("localhost:8080/mydir/myimage:test", &cfg);
        assert!(v);

        let (v, _) = check_image_is_allowed("quay.io:8080/mydir/myimage:test", &cfg);
        assert!(!v);

        let (v, _) = check_image_is_allowed("quay.io/mydir/myimage:test", &cfg);
        assert!(v);

        let cfg = ImageValidationConfig {
            default: "Allow".to_string(),
            allow: vec![],
            deny: vec!["registry-1.docker.io".into(), "toto.land".into()],
        };

        let (v, _) = check_image_is_allowed("ubuntu", &cfg);
        assert!(!v);

        let (v, _) = check_image_is_allowed("toto.land/myimage:test", &cfg);
        assert!(!v);

        let (v, _) = check_image_is_allowed("quay.io/myimage:test", &cfg);
        assert!(v);

        let (v, _) = check_image_is_allowed("quay.io/myimage@invalid", &cfg);
        assert!(!v);
    }
}
