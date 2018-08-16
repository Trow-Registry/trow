use failure::Error;
use std::fs::OpenOptions;
use trow_protobuf::server::*;
use trow_protobuf::server_grpc::BackendClient;
use types::{self,*};
use std::io::prelude::*;

pub struct ClientInterface {
    backend: BackendClient,
}

impl ClientInterface {
    pub fn new(backend: BackendClient) -> Self {
        ClientInterface { backend }
    }

    /**
     * Ok so these functions are largely boilerplate to call the GRPC backend.
     * But doing it here lets us change things behind the scenes much cleaner.
     *
     * Frontend code becomes smaller and doesn't need to know about GRPC types.
     * In fact you could pull it out for a different implementation now by
     * just changing this file...
     **/

    pub fn request_upload(&self, repo_name: &str) -> Result<UploadInfo, Error> {
        let mut req = UploadRequest::new();
        req.set_repo_name(repo_name.to_owned());

        let response = self.backend.request_upload(&req)?;

        Ok(create_upload_info(
            response.get_uuid().to_owned(),
            repo_name.to_string(),
            (0, 0),
        ))
    }

    pub fn complete_upload(&self, repo_name: &str, uuid: &str, digest: &str) -> Result<AcceptedUpload, Error> {

        let mut req = CompleteRequest::new();
        req.set_repo_name(repo_name.to_string());
        req.set_uuid(uuid.to_string());
        req.set_user_digest(digest.to_string());
        let resp = self.backend.complete_upload(&req)?;

        Ok(create_accepted_upload(resp.digest.to_owned(), repo_name.to_owned()))
   
    }

    pub fn get_write_sink_for_upload (
        &self,
        repo_name: &str,
        uuid: &str,
    ) -> Result<impl Write, Error> {

        let mut br = BlobRef::new();
        br.set_uuid(uuid.to_owned());
        br.set_repo_name(repo_name.to_owned());

        let resp = self.backend.get_write_location_for_blob(&br)?;
        
        //For the moment we know it's a file location
        let file = OpenOptions::new().create(true).append(true).open(resp.path)?;
        Ok(file)
    }
    
    pub fn get_write_sink_for_manifest (
        &self,
        repo_name: &str,
        reference: &str,
    ) -> Result<impl Write, Error> {

        let mut mr = ManifestRef::new();
        mr.set_reference(reference.to_owned());
        mr.set_repo_name(repo_name.to_owned());

        let resp = self.backend.get_write_location_for_manifest(&mr)?;
        
        //For the moment we know it's a file location
        //Manifests don't append; just overwrite
        let file = OpenOptions::new().create(true).write(true).open(resp.path)?;
        Ok(file)
    }

    pub fn get_reader_for_manifest (
        &self,
        repo_name: &str,
        reference: &str,
    ) -> Result<ManifestReader, Error> {

        let mut mr = ManifestRef::new();
        mr.set_reference(reference.to_owned());
        mr.set_repo_name(repo_name.to_owned());

        let resp = self.backend.get_read_location_for_manifest(&mr)?;
        
        //For the moment we know it's a file location
        let file = OpenOptions::new().read(true).open(resp.location)?;
        let mr = create_manifest_reader(Box::new(file), resp.content_type, resp.digest);
        Ok(mr)

    }

    pub fn get_reader_for_blob (
        &self,
        repo_name: &str,
        digest: &str,
    ) -> Result<BlobReader, Error> {

        let mut dr = DownloadRef::new();
        dr.set_digest(digest.to_owned());
        dr.set_repo_name(repo_name.to_owned());

        let resp = self.backend.get_read_location_for_blob(&dr)?;
        
        //For the moment we know it's a file location
        let file = OpenOptions::new().read(true).open(resp.path)?;
        let br = create_blob_reader(Box::new(file), digest.to_string());
        Ok(br)
    }

    pub fn verify_manifest (
        &self,
        repo_name: &str,
        reference: &str,
    ) -> Result<types::VerifiedManifest, Error> {

        let mut mr = ManifestRef::new();
        mr.set_reference(reference.to_owned());
        mr.set_repo_name(repo_name.to_owned());

        let resp = self.backend.verify_manifest(&mr)?;
        
        let vm = create_verified_manifest(resp.get_location().to_string(), resp.get_digest().to_string(), resp.get_content_type().to_string());
        Ok(vm)
    }
}
