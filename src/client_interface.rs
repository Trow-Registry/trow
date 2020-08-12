pub mod trow_proto {
    include!("../lib/protobuf/out/trow.rs");
}

use trow_proto::{
    admission_controller_client::AdmissionControllerClient, registry_client::RegistryClient,
    BlobRef, CatalogRequest, CompleteRequest, HealthRequest, ListTagsRequest,
    ManifestHistoryRequest, ManifestRef, MetricsRequest, ReadinessRequest, UploadRef,
    UploadRequest, VerifyManifestRequest,
};

use tonic::Request;

use crate::chrono::TimeZone;
use crate::types::{self, *};
use failure::Error;
use serde_json::Value;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::prelude::*;

pub struct ClientInterface {
    server: String,
}

/**
 * This is really bad way to do things on several levels, but it works for the moment.
 *
 * The major problem is Rust doesn't have TCO so we could be DOS'd by a malicious request.
 */
fn extract_images<'a>(blob: &Value, images: &'a mut Vec<String>) -> &'a Vec<String> {
    match blob {
        Value::Array(vals) => {
            for v in vals {
                extract_images(v, images);
            }
        }
        Value::Object(m) => {
            for (k, v) in m {
                if k == "image" {
                    if let Value::String(image) = v {
                        images.push(image.to_owned())
                    }
                } else {
                    extract_images(v, images);
                }
            }
        }
        _ => (),
    }
    images
}

impl ClientInterface {
    pub fn new(server: String) -> Result<Self, Error> {
        Ok(ClientInterface { server })
    }

    async fn connect_registry(
        &self,
    ) -> Result<RegistryClient<tonic::transport::Channel>, tonic::transport::Error> {
        debug!("Connecting to {}", self.server);
        let x = RegistryClient::connect(self.server.to_string()).await;
        debug!("Connected to {}", self.server);
        x
    }

    async fn connect_admission_controller(
        &self,
    ) -> Result<AdmissionControllerClient<tonic::transport::Channel>, tonic::transport::Error> {
        debug!("Connecting to {}", self.server);
        let x = AdmissionControllerClient::connect(self.server.to_string()).await;
        debug!("Connected to {}", self.server);
        x
    }

    /**
     * Ok so these functions are largely boilerplate to call the GRPC backend.
     * But doing it here lets us change things behind the scenes much cleaner.
     *
     * Frontend code becomes smaller and doesn't need to know about GRPC types.
     * In fact you could pull it out for a different implementation now by
     * just changing this file...
     **/

    pub async fn request_upload(&self, repo_name: &RepoName) -> Result<UploadInfo, Error> {
        let req = UploadRequest {
            repo_name: repo_name.0.clone(),
        };

        let response = self
            .connect_registry()
            .await?
            .request_upload(Request::new(req))
            .await?
            .into_inner();

        Ok(create_upload_info(
            types::Uuid(response.uuid.to_owned()),
            repo_name.clone(),
            (0, 0),
        ))
    }

    pub async fn complete_upload(
        &self,
        repo_name: &RepoName,
        uuid: &Uuid,
        digest: &Digest,
        len: u64,
    ) -> Result<AcceptedUpload, Error> {
        let req = CompleteRequest {
            repo_name: repo_name.0.clone(),
            uuid: uuid.0.clone(),
            user_digest: digest.0.clone(),
        };
        let resp = self
            .connect_registry()
            .await?
            .complete_upload(Request::new(req))
            .await?
            .into_inner();

        Ok(create_accepted_upload(
            Digest(resp.digest.to_owned()),
            repo_name.clone(),
            uuid.clone(),
            (0, (len as u32)),
        ))
    }

    pub async fn get_write_sink_for_upload(
        &self,
        repo_name: &RepoName,
        uuid: &Uuid,
    ) -> Result<impl Write + Seek, Error> {
        let br = UploadRef {
            uuid: uuid.0.clone(),
            repo_name: repo_name.0.clone(),
        };

        let resp = self
            .connect_registry()
            .await?
            .get_write_location_for_blob(Request::new(br))
            .await?
            .into_inner();

        //For the moment we know it's a file location
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(resp.path)?;
        Ok(file)
    }

    pub async fn get_write_sink_for_manifest(
        &self,
        repo_name: &RepoName,
        reference: &str,
    ) -> Result<(impl Write, String), Error> {
        let mr = ManifestRef {
            reference: reference.to_owned(),
            repo_name: repo_name.0.clone(),
        };

        let resp = self
            .connect_registry()
            .await?
            .get_write_details_for_manifest(Request::new(mr))
            .await?
            .into_inner();

        //For the moment we know it's a file location
        //Manifests don't append; just overwrite
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(resp.path)?;
        Ok((file, resp.uuid))
    }

    pub async fn get_reader_for_manifest(
        &self,
        repo_name: &RepoName,
        reference: &str,
    ) -> Result<ManifestReader, Error> {
        let mr = ManifestRef {
            reference: reference.to_owned(),
            repo_name: repo_name.0.clone(),
        };
        let resp = self
            .connect_registry()
            .await?
            .get_read_location_for_manifest(Request::new(mr))
            .await?
            .into_inner();

        //For the moment we know it's a file location
        let file = OpenOptions::new().read(true).open(resp.path)?;
        let mr = create_manifest_reader(
            Box::new(file),
            resp.content_type,
            Digest(resp.digest.to_owned()),
        );
        Ok(mr)
    }

    pub async fn get_manifest_history(
        &self,
        repo_name: &RepoName,
        reference: &str,
        limit: u32,
        last_digest: &str,
    ) -> Result<ManifestHistory, Error> {
        let mr = ManifestHistoryRequest {
            tag: reference.to_owned(),
            repo_name: repo_name.0.clone(),
            limit,
            last_digest: last_digest.to_owned(),
        };
        let mut stream = self
            .connect_registry()
            .await?
            .get_manifest_history(Request::new(mr))
            .await?
            .into_inner();
        let mut history = ManifestHistory::new(format!("{}:{}", repo_name, reference));

        while let Some(entry) = stream.message().await? {
            let ts = if let Some(date) = entry.date {
                chrono::Utc.timestamp(date.seconds, date.nanos.try_into().unwrap())
            } else {
                warn!("Manifest digest stored without timestamp. Using Epoch.");
                chrono::Utc.timestamp(0, 0)
            };
            history.insert(entry.digest, ts);
        }

        Ok(history)
    }

    pub async fn get_reader_for_blob(
        &self,
        repo_name: &RepoName,
        digest: &Digest,
    ) -> Result<BlobReader, Error> {
        let br = BlobRef {
            digest: digest.0.clone(),
            repo_name: repo_name.0.clone(),
        };

        let resp = self
            .connect_registry()
            .await?
            .get_read_location_for_blob(Request::new(br))
            .await?
            .into_inner();

        //For the moment we know it's a file location
        let file = OpenOptions::new().read(true).open(resp.path)?;
        let reader = create_blob_reader(Box::new(file), digest.clone());
        Ok(reader)
    }

    pub async fn delete_blob(
        &self,
        repo_name: &RepoName,
        digest: &Digest,
    ) -> Result<BlobDeleted, Error> {
        let br = BlobRef {
            digest: digest.0.clone(),
            repo_name: repo_name.0.clone(),
        };

        self.connect_registry()
            .await?
            .delete_blob(Request::new(br))
            .await?
            .into_inner();
        Ok(BlobDeleted {})
    }

    pub async fn verify_manifest(
        &self,
        repo_name: &RepoName,
        reference: &str,
        uuid: &str,
    ) -> Result<types::VerifiedManifest, Error> {
        let vmr = VerifyManifestRequest {
            manifest: Some(ManifestRef {
                reference: reference.to_owned(),
                repo_name: repo_name.0.clone(),
            }),
            uuid: uuid.to_string(),
        };

        let resp = self
            .connect_registry()
            .await?
            .verify_manifest(Request::new(vmr))
            .await?
            .into_inner();

        let vm = create_verified_manifest(
            repo_name.clone(),
            Digest(resp.digest.to_owned()),
            reference.to_string(),
            resp.content_type.to_owned(),
        );
        Ok(vm)
    }

    pub async fn delete_manifest(
        &self,
        repo_name: &RepoName,
        digest: &Digest,
    ) -> Result<ManifestDeleted, Error> {
        let mr = ManifestRef {
            reference: digest.0.clone(),
            repo_name: repo_name.0.clone(),
        };

        self.connect_registry()
            .await?
            .delete_manifest(Request::new(mr))
            .await?
            .into_inner();
        Ok(ManifestDeleted {})
    }

    pub async fn get_catalog(&self, limit: u32, last_repo: &str) -> Result<RepoCatalog, Error> {
        let cr = CatalogRequest {
            limit,
            last_repo: last_repo.to_string(),
        };
        let mut stream = self
            .connect_registry()
            .await?
            .get_catalog(Request::new(cr))
            .await?
            .into_inner();
        let mut catalog = RepoCatalog::new();

        while let Some(ce) = stream.message().await? {
            catalog.insert(RepoName(ce.repo_name.to_owned()));
        }

        Ok(catalog)
    }

    pub async fn list_tags(
        &self,
        repo_name: &RepoName,
        limit: u32,
        last_tag: &str,
    ) -> Result<TagList, Error> {
        let ltr = ListTagsRequest {
            repo_name: repo_name.0.clone(),
            limit,
            last_tag: last_tag.to_string(),
        };

        let mut stream = self
            .connect_registry()
            .await?
            .list_tags(Request::new(ltr))
            .await?
            .into_inner();
        let mut list = TagList::new(repo_name.clone());

        while let Some(tag) = stream.message().await? {
            list.insert(tag.tag.to_owned());
        }

        Ok(list)
    }

    /**
     * Returns an AdmissionReview object with the AdmissionResponse completed with details of vaildation.
     */
    pub async fn validate_admission(
        &self,
        req: &types::AdmissionRequest,
        host_names: &[String],
    ) -> Result<types::AdmissionResponse, Error> {
        //TODO: write something to convert automatically (into()) between AdmissionRequest types
        // TODO: we should really be sending the full object to the backend.
        let mut images = Vec::new();
        extract_images(&req.object, &mut images);
        let ar = trow_proto::AdmissionRequest {
            images,
            namespace: req.namespace.clone(),
            operation: req.operation.clone(),
            host_names: host_names.to_vec(),
        };

        let resp = self
            .connect_admission_controller()
            .await?
            .validate_admission(Request::new(ar))
            .await?
            .into_inner();

        //TODO: again, this should be an automatic conversion
        let st = if resp.is_allowed {
            types::Status {
                status: "Success".to_owned(),
                message: None,
                code: None,
            }
        } else {
            //Not sure "Failure" is correct
            types::Status {
                status: "Failure".to_owned(),
                message: Some(resp.reason.to_string()),
                code: None,
            }
        };
        Ok(types::AdmissionResponse {
            uid: req.uid.clone(),
            allowed: resp.is_allowed,
            status: Some(st),
        })
    }

    /**
     Health check.

    Note that the server will indicate unhealthy by returning an error.
    */
    pub async fn is_healthy(&self) -> types::HealthResponse {
        let mut client = match self.connect_registry().await {
            Ok(cl) => cl,
            Err(_) => {
                return types::HealthResponse {
                    is_healthy: false,
                    message: "Failed to connect to registry".to_string(),
                }
            }
        };

        let req = Request::new(HealthRequest {});
        let resp = match client.is_healthy(req).await {
            Ok(r) => r,
            Err(e) => {
                return types::HealthResponse {
                    is_healthy: false,
                    message: e.to_string(),
                }
            }
        };
        let response_value = resp.into_inner();

        types::HealthResponse {
            is_healthy: true,
            message: response_value.message,
        }
    }

    /**
     Readiness check.

     Note that the server will indicate not ready by returning an error.
    */
    pub async fn is_ready(&self) -> types::ReadinessResponse {
        let mut client = match self.connect_registry().await {
            Ok(cl) => cl,
            Err(_) => {
                return types::ReadinessResponse {
                    is_ready: false,
                    message: "Failed to connect to registry".to_string(),
                }
            }
        };

        let req = Request::new(ReadinessRequest {});
        let resp = match client.is_ready(req).await {
            Ok(r) => r,
            Err(e) => {
                return types::ReadinessResponse {
                    is_ready: false,
                    message: e.to_string(),
                }
            }
        };
        let response_value = resp.into_inner();
        types::ReadinessResponse {
            is_ready: true,
            message: response_value.message,
        }
    }

    /**
     Metrics call.

     Returns disk and total request metrics(blobs, manifests).
    */
    pub async fn get_metrics(&self) -> Result<types::MetricsResponse, Error> {
        let req = Request::new(MetricsRequest {});
        let resp = self
            .connect_registry()
            .await?
            .get_metrics(req)
            .await?
            .into_inner();

        Ok(types::MetricsResponse {
            metrics: resp.metrics,
        })
    }
}
