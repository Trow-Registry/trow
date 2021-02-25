use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Blob {
    pub architecture: String,
    pub author: String,
    pub os: String,
    pub created: String,
    pub config: Config,
    pub container: String,
    pub container_config: Config,
    pub history: Vec<BlobHistory>,
    pub rootfs: BlobRootFS,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct BlobHistory {
    pub created: String,
    pub author: String,
    pub created_by: String,
    pub comment: String,
    pub empty_layer: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub host_name: String,
    pub domain_name: String,
    pub user: String,
    pub attach_stdin: bool,
    pub attach_stdout: bool,
    pub attach_stderr: bool,
    pub tty: bool,
    pub open_stdin: bool,
    pub stdin_once: bool,
    pub env: Vec<String>,
    pub cmd: Vec<String>,
    pub image: String,
    pub volumes: HashMap<String, Empty>,
    pub working_dir: String,
    pub entrypoint: Vec<String>,
    pub exposed_ports: HashMap<String, Empty>,
    pub stop_signal: String,
    pub labels: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Empty {}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct BlobRootFS {
    pub diff_ids: Vec<String>,
    pub blob_type: String,
}
