use anyhow::Result;
use log::warn;
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Status};
use json_patch::{Patch, AddOperation, PatchOperation};

use crate::image::Image;
use crate::RegistryProxyConfig;
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
    let image = match Image::try_from_str(raw_image_ref) {
        Ok(image) => image,
        Err(_) => return (false, "Invalid image reference"),
    };
    let image_ref = image.get_name_with_host();
    let mut is_allowed = match config.default.as_str() {
        "Allow" => true,
        "Deny" => false,
        _ => {
            warn!("Invalid default image validation config: `{}`. Should be `Allow` or `Deny`. Default to `Deny`.", config.default);
            false
        }
    };
    let mut match_len = 0;
    let mut match_reson = "Image did not match, using default config";

    for m in config.deny.iter() {
        if m.len() > match_len && image_ref.starts_with(m) {
            is_allowed = false;
            match_len = m.len();
            match_reson = "Image explicitely denied";
        }
    }

    for m in config.allow.iter() {
        if m.len() > match_len && image_ref.starts_with(m) {
            is_allowed = true;
            match_len = m.len();
            match_reson = "Image explicitely allowed";
        }
    }

    (is_allowed, match_reson)
}

fn get_proxy_image(image_ref: &str, proxy_cfg: &[RegistryProxyConfig]) -> Result<Image> {
    let image = Image::try_from_str(image_ref)?;

    for cfg in proxy_cfg.iter() {
        if image.get_host() == cfg.host {
            return Ok(Image::new(
                "trow.io",
                format!("/f/{}/{}", cfg.alias, image.get_name()),
                image.tag,
            ));
        }
    }

    unimplemented!()
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
        let mut reason = String::new();

        for image_raw in ar.images {
            let (v, r) =
                check_image_is_allowed(&image_raw, self.image_validation_config.as_ref().unwrap());
            if !v {
                valid = false;
                reason = format!("{reason}; {image_raw}: {r}");
                break;
            }
        }
        reason.drain(0..2);

        let ar = AdmissionResponse {
            patch: None,
            is_allowed: valid,
            reason,
        };
        Ok(Response::new(ar))
    }

    async fn mutate_admission(
        &self,
        ar: Request<AdmissionRequest>,
    ) -> Result<Response<AdmissionResponse>, Status> {
        let ar = ar.into_inner();

        // let repo_ip =

        for raw_image in ar.images {
            let image = match Image::try_from_str(raw_image) {
                Ok(image) => image,
                Err(_) => continue,
            };

            for cfg in proxy_cfg.iter() {
                if image.get_host() == cfg.host {
                    return Ok(Image::new(
                        "trow.io",
                        format!("/f/{}/{}", cfg.alias, image.get_name()),
                        image.tag,
                    ));
                }
            }

            let patch = Patch::new(vec![
                PatchOperation::new(
                    "replace",
                    "/spec/containers/0/image",
                    image.get_name_with_host(),
                ),
            ]);


        let image = match Image::try_from_str(&ar.images[0]) {
            Ok(image) => image,
            Err(_) => return Ok(Response::new(AdmissionResponse {
                patch: None,
                is_allowed: false,
                reason: "Invalid image reference".to_string(),
            })),
        };

        let mut patch_operations = Vec::<PatchOperation>::new();



        unimplemented!()
        // let mut response = AdmissionResponse::from(ar.into_inner());
        // .with_patch(Patch(vec![PatchOperation::Add(AddOperation {
        //     path: "/metadata/labels/my-label".to_owned(),
        //     value: serde_json::Value::String("my-value".to_owned()),
        // })]));

        // for image_raw in ar.images {



        //     let (v, r) =
        //         check_image_is_allowed(&image_raw, self.image_validation_config.as_ref().unwrap());
        //     if !v {
        //         valid = false;
        //         reason = format!("{reason}; {image_raw}: {r}");
        //         break;
        //     }
        // }
        // reason.drain(0..2);

        // let ar = AdmissionResponse {
        //     patch: None,
        //     is_allowed: valid,
        //     reason,
        // };
        // Ok(Response::new(ar))


    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_check() {
        let cfg = ImageValidationConfig {
            default: "Deny".to_string(),
            allow: vec!["localhost:8080".into(), "quay.io".into()],
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
