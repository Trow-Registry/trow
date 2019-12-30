pub mod trow_proto {
    include!("../lib/protobuf/out/trow.rs");
}

use trow_proto::{
    registry_client::RegistryClient, BlobRef, CatalogEntry, CatalogRequest,
    CompleteRequest, DownloadRef, ManifestRef, UploadRequest,
};
use tonic::Request;
use crate::types::{self, *};
use failure::{format_err, Error};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::prelude::*;
use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio;

pub struct ClientInterface {
    server: String,
    runtime: Runtime
    //rc: RegistryClient<tonic::transport::Channel>,
    //ac: AdmissionControllerClient,
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
        
        //delete me
        let runtime = Runtime::new()?;

        Ok(ClientInterface { server, runtime })


        //Create tokio runtime here. 
        // Should be able to call spawn (but not block_on which is &mut)
    }

    async fn connect_registry(&self) -> 
    Result<RegistryClient<tonic::transport::Channel>, tonic::transport::Error> {

        warn!("Connecting to {}", self.server);
        let x = RegistryClient::connect(self.server.to_string()).await;
        warn!("Connected to {}", self.server);
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
        
        let req = UploadRequest{
            repo_name: repo_name.0.clone()
        };

        let response = self.connect_registry().await?.request_upload(Request::new(req)).await?.into_inner();

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
    ) -> Result<AcceptedUpload, Error> {
        let req = CompleteRequest {
            repo_name: repo_name.0.clone(),
            uuid: uuid.0.clone(),
            user_digest: digest.0.clone()
        };
        let resp = self.connect_registry().await?.complete_upload(Request::new(req)).await?.into_inner();

        Ok(create_accepted_upload(
            Digest(resp.digest.to_owned()),
            repo_name.clone(),
        ))
    }

    pub async fn get_write_sink_for_upload(
        &self,
        repo_name: &RepoName,
        uuid: &Uuid,
    ) -> Result<impl Write, Error> {
        let br = BlobRef {
            uuid: uuid.0.clone(),
            repo_name: repo_name.0.clone()
        };

        let resp = self.connect_registry().await?.get_write_location_for_blob(
            Request::new(br)).await?.into_inner();

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
    ) -> Result<impl Write, Error> {
        let mr = ManifestRef {
            reference: reference.to_owned(),
            repo_name: repo_name.0.clone()
        };
    
        let resp = self.connect_registry().await?.get_write_location_for_manifest(
            Request::new(mr)).await?.into_inner();

        //For the moment we know it's a file location
        //Manifests don't append; just overwrite
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(resp.path)?;
        Ok(file)
    }

    pub async fn get_reader_for_manifest(
        &self,
        repo_name: &RepoName,
        reference: &str,
    ) -> Result<ManifestReader, Error> {
        
        let mr = ManifestRef {
            reference: reference.to_owned(),
            repo_name: repo_name.0.clone()
        };
        let resp = self.connect_registry().await?.get_read_location_for_manifest(
            Request::new(mr)).await?.into_inner();

        //For the moment we know it's a file location
        let file = OpenOptions::new().read(true).open(resp.path)?;
        let mr = create_manifest_reader(
            Box::new(file),
            resp.content_type,
            Digest(resp.digest.to_owned()),
        );
        Ok(mr)
    }

    pub async fn get_reader_for_blob(
        &self,
        repo_name: &RepoName,
        digest: &Digest,
    ) -> Result<BlobReader, Error> {
        let dr = DownloadRef {
            digest: digest.0.clone(),
            repo_name: repo_name.0.clone()
        };

        let resp = self.connect_registry().await?.get_read_location_for_blob(
            Request::new(dr)).await?.into_inner();

        //For the moment we know it's a file location
        let file = OpenOptions::new().read(true).open(resp.path)?;
        let br = create_blob_reader(Box::new(file), digest.clone());
        Ok(br)
    }

    pub async fn verify_manifest(
        &self,
        repo_name: &RepoName,
        reference: &str,
    ) -> Result<types::VerifiedManifest, Error> {
        let mr = ManifestRef {
            reference: reference.to_owned(),
            repo_name: repo_name.0.clone()
        };

        let resp = self.connect_registry().await?.verify_manifest(
            Request::new(mr)).await?.into_inner();
        

        let vm = create_verified_manifest(
            repo_name.clone(),
            Digest(resp.digest.to_owned()),
            reference.to_string(),
            resp.content_type.to_owned(),
        );
        Ok(vm)
    }

    pub async fn get_catalog(&self) -> Result<RepoCatalog, Error> {

        let cr = CatalogRequest {};
        let mut stream = self.connect_registry().await?
            .get_catalog(
                Request::new(cr))
                .await?
                .into_inner();
        let mut catalog = RepoCatalog::new();

        while let Some(ce) = stream.message().await? {
            catalog.insert(RepoName(ce.repo_name.to_owned()));
        }    

        Ok(catalog)
    }

    pub async fn list_tags(&self, repo_name: &RepoName) -> Result<TagList, Error> {
        let ce = CatalogEntry {
            repo_name: repo_name.0.clone()
        };

        let mut stream = self.connect_registry().await?
        .list_tags(
            Request::new(ce))
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
    pub fn validate_admission(
        &self,
        in_req: &types::AdmissionRequest,
        host_names: &[String],
    ) -> Result<types::AdmissionResponse, Error> {

        return Ok(
            types::AdmissionResponse {
                uid: in_req.uid.clone(),
                allowed: true,
                status: None,
            })

        /*
        //TODO: write something to convert automatically (into())
        let mut a_req = AdmissionRequest::new();

        // TODO: we should really be sending the full object to the backend.
        // Revisit this when we have proper rust bindings
        let mut images = Vec::new();
        extract_images(&in_req.object, &mut images);

        //The conversion here will be easier when we can upgrade the protobuf stuff
        a_req.set_images(RepeatedField::from_vec(images.clone()));

        a_req.set_namespace(in_req.namespace.clone());
        a_req.set_operation(in_req.operation.clone());
        a_req.set_host_names(RepeatedField::from_vec(host_names.to_vec()));

        let resp = self.ac.validate_admission(&a_req)?;

        //TODO: again, this should be an automatic conversion
        let st = if resp.get_is_allowed() {
            types::Status {
                status: "Success".to_owned(),
                message: None,
                code: None,
            }
        } else {
            //Not sure "Failure is correct"
            types::Status {
                status: "Failure".to_owned(),
                message: Some(resp.get_reason().to_string()),
                code: None,
            }
        };
        Ok(types::AdmissionResponse {
            uid: in_req.uid.clone(),
            allowed: resp.get_is_allowed(),
            status: Some(st),
        })
        */
    }
}
