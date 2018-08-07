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

#[derive(Debug, Clone)]
pub struct Layer {
    pub digest: Digest,
    pub repo_name: String,
}
