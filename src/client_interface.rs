use std::convert::TryInto;
use std::io::SeekFrom;

use crate::trow_server;
use crate::trow_server::api_types::{HealthStatus, ReadyStatus, Status};
use anyhow::{anyhow, Result};
use axum::extract::BodyStream;
use chrono::TimeZone;
use futures::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::core::admission::{AdmissionRequest, AdmissionResponse};
use thiserror::Error;
use tokio::io::{AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt};
use tracing::{event, Level};
use trow_server::api_types::{
    BlobRef, CatalogRequest, CompleteRequest, ListTagsRequest, ManifestHistoryRequest, ManifestRef,
    MetricsRequest, UploadRef, UploadRequest, VerifyManifestRequest,
};

use crate::registry_interface::blob_storage::Stored;
use crate::registry_interface::digest::{self, Digest, DigestAlgorithm};
use crate::registry_interface::{
    AdmissionValidation, BlobReader, BlobStorage, CatalogOperations, ContentInfo, ManifestHistory,
    ManifestReader, ManifestStorage, Metrics, MetricsError, StorageDriverError,
};
use crate::types::{self, *};
use trow_server::api_types::MetricsResponse;
use trow_server::TrowServer;

#[derive(Debug)]
pub struct ClientInterface {
    trow_server: TrowServer,
}

fn extract_images(pod: &Pod) -> (Vec<String>, Vec<String>) {
    let mut images = vec![];
    let mut paths = vec![];

    let spec = pod.spec.clone().unwrap_or_default();
    for (i, container) in spec.containers.iter().enumerate() {
        if let Some(image) = &container.image {
            images.push(image.clone());
            paths.push(format!("/spec/containers/{i}/image"));
        }
    }

    for (i, container) in spec.init_containers.unwrap_or_default().iter().enumerate() {
        if let Some(image) = &container.image {
            images.push(image.clone());
            paths.push(format!("/spec/initContainers/{i}/image"));
        }
    }

    (images, paths)
}

// TODO: Each function should have it's own enum of the errors it can return
// There must be a standard pattern for this somewhere...
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Invalid repository or tag")]
    InvalidName,
    #[error("Invalid manifest")]
    InvalidManifest,
    #[error("Internal registry error")]
    Internal,
}

#[axum::async_trait]
impl ManifestStorage for ClientInterface {
    async fn get_manifest(
        &self,
        name: &str,
        tag: &str,
    ) -> Result<ManifestReader, StorageDriverError> {
        let rn = RepoName(name.to_string());
        let mr = self.get_reader_for_manifest(&rn, tag).await.map_err(|e| {
            event!(Level::WARN, "Error getting manifest: {}", e);
            StorageDriverError::Internal
        })?;

        Ok(mr)
    }

    async fn store_manifest<'a>(
        &self,
        name: &str,
        tag: &str,
        data: BodyStream,
    ) -> Result<Digest, StorageDriverError> {
        let repo = RepoName(name.to_string());

        match self.upload_manifest(&repo, tag, data).await {
            Ok(vm) => Ok(vm.digest().clone()),
            Err(RegistryError::InvalidName) => {
                Err(StorageDriverError::InvalidName(format!("{}:{}", name, tag)))
            }
            Err(RegistryError::InvalidManifest) => Err(StorageDriverError::InvalidManifest),
            Err(_) => Err(StorageDriverError::Internal),
        }
    }

    async fn delete_manifest(&self, name: &str, digest: &Digest) -> Result<(), StorageDriverError> {
        let repo = RepoName(name.to_string());
        self
            .delete_by_manifest(&repo, digest)
            .await
            .map_err(|e| match e {
                Status::InvalidArgument(_) => StorageDriverError::Unsupported,
                Status::NotFound(_) => StorageDriverError::InvalidManifest,
                _ => StorageDriverError::Internal,
            })?;

        Ok(())
    }

    async fn has_manifest(&self, _name: &str, _algo: &DigestAlgorithm, _reference: &str) -> bool {
        todo!()
    }
}

#[axum::async_trait]
impl BlobStorage for ClientInterface {
    async fn get_blob(
        &self,
        name: &str,
        digest: &Digest,
    ) -> Result<BlobReader, StorageDriverError> {
        let rn = RepoName(name.to_string());
        let br = self.get_reader_for_blob(&rn, digest).await.map_err(|e| {
            event!(Level::WARN, "Error getting blob: {}", e);
            StorageDriverError::Internal
        })?;

        Ok(br)
    }

    async fn store_blob_chunk<'a>(
        &self,
        name: &str,
        session_id: &str,
        data_info: Option<ContentInfo>,
        mut data: BodyStream,
    ) -> Result<Stored, StorageDriverError> {
        let rn = RepoName(name.to_string());
        let uuid = Uuid(session_id.to_string());
        let mut sink = self
            .get_write_sink_for_upload(&rn, &uuid)
            .await
            .map_err(|e| {
                event!(Level::WARN, "Error finding write sink for blob {:?}", e);
                StorageDriverError::InvalidName(format!("{} {}", name, session_id))
            })?;

        let have_range = data_info.is_some();
        let info = data_info.unwrap_or(ContentInfo {
            length: 0,
            range: (0, 0),
        });

        let start_index = sink.seek(SeekFrom::End(0)).await.unwrap_or(0);
        if have_range && (start_index != info.range.0) {
            event!(
                Level::WARN,
                "Asked to store blob with invalid start index. Expected {} got {}",
                start_index,
                info.range.0
            );
            return Err(StorageDriverError::InvalidContentRange);
        }

        let mut chunk_size = 0;
        while let Some(v) = data.next().await {
            match v {
                Ok(v) => {
                    chunk_size += v.len() as u64;
                    if let Err(e) = sink.write_all(&v).await {
                        event!(Level::ERROR, "Error writing out manifest {:?}", e);
                        return Err(StorageDriverError::Internal);
                    }
                }
                Err(e) => {
                    event!(Level::ERROR, "Error reading manifest {e}",);
                    return Err(StorageDriverError::Internal);
                }
            }
        }

        let total = sink.seek(SeekFrom::End(0)).await.unwrap();
        if have_range {
            if (info.range.1 + 1) != total {
                event!(Level::WARN, "total {} r + 1 {}", total, info.range.1 + 1);
                return Err(StorageDriverError::InvalidContentRange);
            }
            //Check length if chunked upload
            if info.length != chunk_size {
                event!(
                    Level::WARN,
                    "info.length {} len {}",
                    info.length,
                    chunk_size
                );
                return Err(StorageDriverError::InvalidContentRange);
            }
        }

        Ok(Stored {
            total_stored: total,
            chunk: chunk_size,
        })
    }

    async fn complete_and_verify_blob_upload(
        &self,
        name: &str,
        session_id: &str,
        digest: &Digest,
    ) -> Result<(), StorageDriverError> {
        self.complete_upload(name, session_id, digest).await.map_err(|e| {
            match e {
                Status::InvalidArgument(_) => StorageDriverError::InvalidDigest,
                _ => StorageDriverError::Internal,
            }
        })?;
        Ok(())
    }

    async fn start_blob_upload(&self, name: &str) -> Result<String, StorageDriverError> {
        self.request_upload(name).await.map_err(|e| match e {
            Status::InvalidArgument(_) => StorageDriverError::InvalidName(name.to_string()),
            _ => StorageDriverError::Internal,
        })
    }

    async fn delete_blob(&self, name: &str, digest: &Digest) -> Result<(), StorageDriverError> {
        event!(
            Level::INFO,
            "Attempting to delete blob {} in {}",
            digest,
            name
        );
        let rn = RepoName(name.to_string());

        self.delete_blob_local(&rn, digest)
            .await
            .map_err(|_| StorageDriverError::InvalidDigest)?;
        Ok(())
    }

    async fn status_blob_upload(
        &self,
        _name: &str,
        _session_id: &str,
    ) -> crate::registry_interface::UploadInfo {
        todo!()
    }

    async fn cancel_blob_upload(
        &self,
        _name: &str,
        _session_id: &str,
    ) -> Result<(), StorageDriverError> {
        todo!()
    }

    async fn has_blob(&self, _name: &str, _digest: &Digest) -> bool {
        todo!()
    }
}

#[axum::async_trait]
impl CatalogOperations for ClientInterface {
    async fn get_catalog(
        &self,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<Vec<String>, StorageDriverError> {
        let num_results = num_results.unwrap_or(u32::MAX);
        let start_value = start_value.unwrap_or_default();

        self.get_catalog_part(num_results, start_value)
            .await
            .map_err(|_| StorageDriverError::Internal)
            .map(|rc| rc.raw())
    }

    async fn get_tags(
        &self,
        repo: &str,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<Vec<String>, StorageDriverError> {
        let num_results = num_results.unwrap_or(u32::MAX);
        let start_value = start_value.unwrap_or_default();

        self.list_tags(repo, num_results, start_value)
            .await
            .map_err(|_| StorageDriverError::Internal)
            .map(|rc| rc.raw())
    }

    async fn get_history(
        &self,
        repo: &str,
        name: &str,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<ManifestHistory, StorageDriverError> {
        let num_results = num_results.unwrap_or(u32::MAX);
        let start_value = start_value.unwrap_or_default();

        self.get_manifest_history(repo, name, num_results, start_value)
            .await
            .map_err(|_| StorageDriverError::Internal)
    }
}

#[axum::async_trait]
impl AdmissionValidation for ClientInterface {
    async fn validate_admission(
        &self,
        admission_req: &AdmissionRequest<Pod>,
        host_name: &str,
    ) -> AdmissionResponse {
        self.validate_admission_internal(admission_req, host_name)
            .await
            .unwrap_or_else(|e| {
                AdmissionResponse::from(admission_req).deny(format!("Internal error: {}", e))
            })
    }

    async fn mutate_admission(
        &self,
        admission_req: &AdmissionRequest<Pod>,
        host_name: &str,
    ) -> AdmissionResponse {
        self.mutate_admission_internal(admission_req, host_name)
            .await
            .unwrap_or_else(|e| {
                AdmissionResponse::from(admission_req).deny(format!("Internal error: {}", e))
            })
    }
}

#[axum::async_trait]
impl Metrics for ClientInterface {
    async fn is_healthy(&self) -> bool {
        self.is_healthy().await.is_healthy
    }

    async fn is_ready(&self) -> bool {
        self.is_ready().await.is_ready
    }

    async fn get_metrics(
        &self,
    ) -> Result<MetricsResponse, crate::registry_interface::MetricsError> {
        self.get_metrics().await.map_err(|_| MetricsError::Internal)
    }
}

impl ClientInterface {
    pub fn new(ts: TrowServer) -> Result<Self> {
        Ok(ClientInterface { trow_server: ts })
    }

    async fn request_upload(&self, repo_name: &str) -> Result<String, Status> {
        event!(Level::INFO, "Request Upload called for {}", repo_name);
        let req = UploadRequest {
            repo_name: repo_name.to_string(),
        };

        let response = self.trow_server.request_upload(req).await?;

        Ok(response.uuid)
    }

    async fn complete_upload(&self, repo_name: &str, uuid: &str, digest: &Digest) -> Result<(), Status> {
        event!(
            Level::INFO,
            "Complete Upload called for repository {} with upload id {} digest {}",
            repo_name,
            uuid,
            digest
        );

        let req = CompleteRequest {
            repo_name: repo_name.to_string(),
            uuid: uuid.to_string(),
            user_digest: digest.to_string(),
        };

        self.trow_server.complete_upload(req).await?;

        Ok(())
    }

    async fn get_write_sink_for_upload(
        &self,
        repo_name: &RepoName,
        uuid: &Uuid,
    ) -> Result<impl AsyncWrite + AsyncSeek> {
        event!(
            Level::INFO,
            "Getting write location for blob in repo {} with upload id {}",
            repo_name,
            uuid
        );
        let br = UploadRef {
            uuid: uuid.0.clone(),
            repo_name: repo_name.0.clone(),
        };

        let resp = self.trow_server.get_write_location_for_blob(br).await?;

        //For the moment we know it's a file location
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(resp.path)
            .await?;
        Ok(file)
    }

    async fn upload_manifest(
        &self,
        repo_name: &RepoName,
        reference: &str,
        mut manifest: BodyStream,
    ) -> Result<types::VerifiedManifest, RegistryError> {
        let (mut sink_loc, uuid) = self
            .get_write_sink_for_manifest(repo_name, reference)
            .await
            ?;

        while let Some(v) = manifest.next().await {
            match v {
                Err(e) => {
                    event!(Level::ERROR, "Could not read manifest: {e}");
                    return Err(RegistryError::Internal);
                }
                Ok(bytes) => match sink_loc.write_all(&bytes).await {
                    Ok(_) => {}
                    Err(e) => {
                        event!(Level::ERROR, "Could not write manifest: {e}");
                        return Err(RegistryError::Internal);
                    }
                },
            }
        }

        self.verify_manifest(repo_name, reference, &uuid)
            .await
    }

    async fn get_write_sink_for_manifest(
        &self,
        repo_name: &RepoName,
        reference: &str,
    ) -> Result<(impl AsyncWrite, String), RegistryError> {
        event!(
            Level::INFO,
            "Getting write location for manifest in repo {} with ref {}",
            repo_name,
            reference
        );
        let mr = ManifestRef {
            reference: reference.to_owned(),
            repo_name: repo_name.0.clone(),
        };

        let resp = self.trow_server.get_write_details_for_manifest(mr).await.map_err(|_| {
            RegistryError::InvalidName
        })?;

        //For the moment we know it's a file location
        //Manifests don't append; just overwrite
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(resp.path)
            .await.map_err(|_| RegistryError::Internal)?;
        Ok((file, resp.uuid))
    }

    async fn get_reader_for_manifest(
        &self,
        repo_name: &RepoName,
        reference: &str,
    ) -> Result<ManifestReader> {
        event!(
            Level::DEBUG,
            "Getting read location for {} with ref {}",
            repo_name,
            reference
        );
        let mr = ManifestRef {
            reference: reference.to_owned(),
            repo_name: repo_name.0.clone(),
        };
        let resp = self.trow_server.get_read_location_for_manifest(mr).await?;

        //For the moment we know it's a file location
        let file = tokio::fs::File::open(resp.path).await?;
        let digest = digest::parse(&resp.digest)?;
        let mr = ManifestReader::new(resp.content_type, digest, file).await?;
        Ok(mr)
    }

    async fn get_manifest_history(
        &self,
        repo_name: &str,
        reference: &str,
        limit: u32,
        last_digest: &str,
    ) -> Result<ManifestHistory> {
        event!(
            Level::INFO,
            "Getting manifest history for repo {} ref {} limit {} last_digest {}",
            repo_name,
            reference,
            limit,
            last_digest
        );
        let mr = ManifestHistoryRequest {
            tag: reference.to_owned(),
            repo_name: repo_name.to_string(),
            limit,
            last_digest: last_digest.to_owned(),
        };
        let stream = self.trow_server.get_manifest_history(mr).await?;
        let mut history = ManifestHistory::new(format!("{}:{}", repo_name, reference));

        for entry in stream {
            let ts = if let Some(date) = entry.date {
                chrono::Utc
                    .timestamp_opt(date.seconds, date.nanos.try_into().unwrap())
                    .earliest()
                    .unwrap()
            } else {
                event!(
                    Level::WARN,
                    "Manifest digest stored without timestamp. Using Epoch."
                );
                chrono::Utc.timestamp_opt(0, 0).earliest().unwrap()
            };
            history.insert(entry.digest, ts);
        }

        Ok(history)
    }

    async fn get_reader_for_blob(
        &self,
        repo_name: &RepoName,
        digest: &Digest,
    ) -> Result<BlobReader> {
        event!(
            Level::DEBUG,
            "Getting read location for blob {} in {}",
            digest,
            repo_name
        );
        let br = BlobRef {
            digest: digest.to_string(),
            repo_name: repo_name.0.clone(),
        };

        let resp = self.trow_server.get_read_location_for_blob(br).await?;

        //For the moment we know it's a file location
        let file = tokio::fs::File::open(resp.path).await?;
        let reader = BlobReader::new(digest.clone(), file).await;
        Ok(reader)
    }

    async fn delete_blob_local(
        &self,
        repo_name: &RepoName,
        digest: &Digest,
    ) -> Result<BlobDeleted> {
        event!(
            Level::INFO,
            "Attempting to delete blob {} in {}",
            digest,
            repo_name
        );
        let br = BlobRef {
            digest: digest.to_string(),
            repo_name: repo_name.0.clone(),
        };

        self.trow_server.delete_blob(br).await?;
        Ok(BlobDeleted {})
    }

    async fn verify_manifest(
        &self,
        repo_name: &RepoName,
        reference: &str,
        uuid: &str,
    ) -> Result<types::VerifiedManifest, RegistryError> {
        event!(
            Level::INFO,
            "Verifying manifest {} in {} uuid {}",
            reference,
            repo_name,
            uuid
        );
        let vmr = VerifyManifestRequest {
            manifest: Some(ManifestRef {
                reference: reference.to_owned(),
                repo_name: repo_name.0.clone(),
            }),
            uuid: uuid.to_string(),
        };

        let resp = self
            .trow_server
            .verify_manifest(vmr)
            .await
            .map_err(|e| match e {
                Status::InvalidArgument(_) => RegistryError::InvalidManifest,
                _ => RegistryError::Internal,
            })?;

        let digest = digest::parse(&resp.digest).map_err(|_| RegistryError::InvalidManifest)?;
        let vm = VerifiedManifest::new(None, repo_name.clone(), digest, reference.to_string());
        Ok(vm)
    }

    async fn delete_by_manifest(
        &self,
        repo_name: &RepoName,
        digest: &Digest,
    ) -> Result<ManifestDeleted, Status> {
        event!(
            Level::INFO,
            "Attempting to delete manifest {} in {}",
            digest,
            repo_name
        );
        let mr = ManifestRef {
            reference: digest.to_string(),
            repo_name: repo_name.0.clone(),
        };

        self.trow_server.delete_manifest(mr).await?;
        Ok(ManifestDeleted {})
    }

    async fn get_catalog_part(&self, limit: u32, last_repo: &str) -> Result<RepoCatalog> {
        event!(
            Level::INFO,
            "Getting image catalog limit {} last_repo {}",
            limit,
            last_repo
        );

        let cr = CatalogRequest {
            limit,
            last_repo: last_repo.to_string(),
        };
        let stream = self.trow_server.get_catalog(cr).await?;
        let mut catalog = RepoCatalog::new();

        for ce in stream {
            catalog.insert(ce.repo_name.to_owned());
        }

        Ok(catalog)
    }

    async fn list_tags(&self, repo_name: &str, limit: u32, last_tag: &str) -> Result<TagList> {
        event!(
            Level::INFO,
            "Getting tag list for {} limit {} last_tag {}",
            repo_name,
            limit,
            last_tag
        );
        let ltr = ListTagsRequest {
            repo_name: repo_name.to_string(),
            limit,
            last_tag: last_tag.to_string(),
        };

        let stream = self.trow_server.list_tags(ltr).await?;
        let mut list = TagList::new(repo_name.to_string());
        for tag in stream {
            list.insert(tag.tag);
        }

        Ok(list)
    }

    /**
     * Returns an AdmissionReview object with the AdmissionResponse completed with details of vaildation.
     */
    async fn validate_admission_internal(
        &self,
        req: &AdmissionRequest<Pod>,
        host_name: &str,
    ) -> Result<AdmissionResponse> {
        event!(
            Level::INFO,
            "Validating admission request {} host_name {:?}",
            req.uid,
            host_name
        );
        // TODO: we should really be sending the full object to the backend.
        let obj = req
            .object
            .as_ref()
            .ok_or_else(|| anyhow!("No pod in pod admission request"))?;
        let (images, _) = extract_images(obj);
        let ar = trow_server::api_types::AdmissionRequest {
            host_name: host_name.to_string(),
            image_paths: vec![], // unused in validation
            images,
            namespace: req
                .namespace
                .clone()
                .ok_or_else(|| anyhow!("Object has no namespace"))?,
        };

        let internal_resp = self.trow_server.validate_admission(ar).await?;

        let mut resp = AdmissionResponse::from(req);
        if !internal_resp.is_allowed {
            resp = resp.deny(internal_resp.reason);
        }

        Ok(resp)
    }

    async fn mutate_admission_internal(
        &self,
        req: &AdmissionRequest<Pod>,
        host_name: &str,
    ) -> Result<AdmissionResponse> {
        event!(
            Level::INFO,
            "Mutating admission request {} host_name {:?}",
            req.uid,
            host_name
        );
        // TODO: we should really be sending the full object to the backend.
        let obj = req
            .object
            .as_ref()
            .ok_or_else(|| anyhow!("No pod in pod admission request"))?;
        let (images, image_paths) = extract_images(obj);
        let ar = trow_server::api_types::AdmissionRequest {
            host_name: host_name.to_string(),
            image_paths,
            images,
            namespace: req
                .namespace
                .clone()
                .ok_or_else(|| anyhow!("Object has no namespace"))?,
        };

        let internal_resp = self.trow_server.mutate_admission(ar).await?;

        let mut resp = AdmissionResponse::from(req);
        if let Some(raw_patch) = internal_resp.patch {
            let patch: json_patch::Patch = serde_json::from_slice(raw_patch.as_slice())?;
            resp = resp.with_patch(patch)?;
        }

        if !internal_resp.is_allowed {
            resp = resp.deny("Failure");
        }

        Ok(resp)
    }

    /**
    Health check.

    Note that the server will indicate unhealthy by returning an error.
    */
    async fn is_healthy(&self) -> HealthStatus {
        event!(Level::DEBUG, "Calling health check");
        self.trow_server.is_healthy().await
    }

    /**
     Readiness check.

     Note that the server will indicate not ready by returning an error.
    */
    async fn is_ready(&self) -> ReadyStatus {
        event!(Level::DEBUG, "Calling readiness check");
        self.trow_server.is_ready().await
    }

    /**
     Metrics call.

     Returns disk and total request metrics(blobs, manifests).
    */
    async fn get_metrics(&self) -> Result<MetricsResponse, MetricsError> {
        event!(Level::DEBUG, "Getting metrics");
        let req = MetricsRequest {};
        self.trow_server
            .get_metrics(req)
            .await
            .map_err(|_| MetricsError::Internal)
    }
}
