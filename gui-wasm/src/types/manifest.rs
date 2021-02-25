use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use std::default::Default;

#[derive( Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: u8,
    pub config: Descriptor,
    pub layers: Vec<Descriptor>,
    pub media_type: String,
    pub annotations: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Descriptor {
    pub media_type: String,
    pub digest: String,
    pub size: u32,
    pub urls: Vec<String>,
    pub annotations: HashMap<String, String>
}


