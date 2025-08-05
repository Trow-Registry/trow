use k8s_openapi::api::core::v1::Pod;
use kube::core::admission::{AdmissionRequest, AdmissionResponse};
use serde::{Deserialize, Serialize};

use super::TrowServer;
use crate::registry::proxy::RemoteImage;

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
            tracing::warn!(
                "Invalid default image validation config: `{}`. Should be `Allow` or `Deny`. Default to `Deny`.",
                config.default
            );
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

fn extract_images(pod: &Pod) -> (Vec<String>, Vec<jsonptr::PointerBuf>) {
    let mut images = vec![];
    let mut paths = vec![];

    let spec = pod.spec.clone().unwrap_or_default();
    for (i, container) in spec.containers.iter().enumerate() {
        if let Some(image) = &container.image {
            images.push(image.clone());
            paths.push(jsonptr::PointerBuf::parse(format!("/spec/containers/{i}/image")).unwrap());
        }
    }

    for (i, container) in spec.init_containers.unwrap_or_default().iter().enumerate() {
        if let Some(image) = &container.image {
            images.push(image.clone());
            paths.push(
                jsonptr::PointerBuf::parse(format!("/spec/initContainers/{i}/image")).unwrap(),
            );
        }
    }

    (images, paths)
}

// AdmissionController
impl TrowServer {
    pub async fn validate_admission(&self, ar: &AdmissionRequest<Pod>) -> AdmissionResponse {
        let resp = AdmissionResponse::from(ar);

        if self.config.image_validation.is_none() {
            return resp.deny("Image validation not configured");
        }
        let pod = match &ar.object {
            Some(pod) => pod,
            None => return resp.deny("No pod in pod admission request"),
        };
        let (images, _) = extract_images(pod);

        let mut valid = true;
        let mut reasons = Vec::new();

        for image_raw in images {
            let (v, r) =
                check_image_is_allowed(&image_raw, self.config.image_validation.as_ref().unwrap());
            if !v {
                valid = false;
                reasons.push(format!("{image_raw}: {r}"));
                break;
            }
        }

        if valid {
            resp
        } else {
            resp.deny(reasons.join("; "))
        }
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
            deny: vec!["docker.io".into(), "toto.land".into()],
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
