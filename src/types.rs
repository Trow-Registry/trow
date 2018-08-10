pub type Digest = String;

#[derive(Debug, Serialize)]
pub struct UploadInfo {
    uuid: String,
    repo_name: String,
    range: (u32, u32),
}

impl UploadInfo {
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn repo_name(&self) -> &str {
        &self.repo_name
    }

    pub fn range(&self) -> (u32, u32) {
        self.range
    }
}

pub fn create_upload_info(uuid: String, repo_name: String, range: (u32, u32)) -> UploadInfo {
    UploadInfo {
        uuid,
        repo_name,
        range,
    }
}

#[derive(Debug, Serialize)]
pub struct AcceptedUpload {
    uuid: String,
    digest: String,
    repo_name: String,
}

pub fn create_accepted_upload(uuid: String, digest: String, repo_name: String) -> AcceptedUpload {
    AcceptedUpload {
        uuid,
        digest,
        repo_name,
    }
}
impl AcceptedUpload {
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn repo_name(&self) -> &str {
        &self.repo_name
    }

}

//DIE MOFO
#[derive(Debug, Clone)]
pub struct Layer {
    pub digest: Digest,
    pub repo_name: String,
}
