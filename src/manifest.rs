use std;
use serde_json::Value;

pub trait FromJson {
    fn from_json(raw: &Value) -> Self;

    fn from_json_map(raw: &Value) -> Vec<Self>
    where
        Self: std::marker::Sized,
    {
        match raw.as_array() {
            Some(vec) => vec.into_iter().map(Self::from_json).collect(),
            None => vec![],
        }
    }
}

#[derive(Debug, Default)]
#[allow(non_snake_case)]
pub struct Manifest {
    pub schemaVersion: u64,
    pub name: String,
    pub tag: String,
    pub architecture: String,
    pub fsLayers: Vec<BlobSummary>,
    pub history: Vec<EmptyStruct>,
    pub signatures: Vec<Signature>,
}

impl FromJson for Manifest {
    fn from_json(raw: &Value) -> Self {
        let version = raw["schemaVersion"].as_u64().unwrap();
        debug!("version {}", version);
        if version == 1 {
            Manifest {
                schemaVersion: raw["schemaVersion"].as_u64().unwrap(),
                name: raw["name"].as_str().unwrap().to_owned(),
                tag: raw["tag"].as_str().unwrap().to_owned(),
                architecture: raw["architecture"].as_str().unwrap().to_owned(),
                /*
                fsLayers: BlobSummary::from_json_map(&raw["fsLayers"]),
                signatures: Signature::from_json_map(&raw["signatures"]),
                history: EmptyStruct::from_json_map(&raw["history"]),
                */
                fsLayers: Vec::new(),
                signatures: Vec::new(),
                history: Vec::new(),

            }
        } else {
            Manifest {
                schemaVersion: raw["schemaVersion"].as_u64().unwrap(),
                name: "UNSUPPORTED".to_owned(),
                tag: "UNSUPPORTED".to_owned(),
                architecture: "UNSUPPORTED".to_owned(),
                fsLayers: Vec::new(),
                signatures: Vec::new(),
                history: Vec::new(),
            }
        }
    }
}

#[derive(Debug, Default)]
struct SignatureJWK {
    crv: String,
    kty: String,
    x: String,
    y: String,
}

impl FromJson for SignatureJWK {
    fn from_json(raw: &Value) -> Self {
        SignatureJWK {
            crv: raw["crv"].as_str().unwrap().to_owned(),
            kty: raw["kty"].as_str().unwrap().to_owned(),
            x: raw["x"].as_str().unwrap().to_owned(),
            y: raw["y"].as_str().unwrap().to_owned(),
        }
    }
}

#[derive(Debug, Default)]
struct SignatureHeader {
    alg: String,
    jwk: SignatureJWK,
}

impl FromJson for SignatureHeader {
    fn from_json(raw: &Value) -> Self {
        SignatureHeader {
            alg: raw["alg"].as_str().unwrap().to_owned(),
            jwk: SignatureJWK::from_json(&raw["jwk"]),
        }
    }
}

#[derive(Debug, Default)]
pub struct Signature {
    header: SignatureHeader,
    payload: String,
    protected: String,
    signature: String,
}

impl FromJson for Signature {
    fn from_json(raw: &Value) -> Self {
        Signature {
            header: SignatureHeader::from_json(&raw["header"]),
            payload: raw["payload"].as_str().unwrap().to_owned(),
            protected: raw["protected"].as_str().unwrap().to_owned(),
            signature: raw["signature"].as_str().unwrap().to_owned(),
        }
    }
}

#[derive(Clone, Debug, Default, RustcDecodable, RustcEncodable)]
#[allow(non_snake_case)]
pub struct BlobSummary {
    pub blobSum: String,
}

impl FromJson for BlobSummary {
    fn from_json(raw: &Value) -> Self {
        BlobSummary {
            blobSum: raw["blobSum"].as_str().unwrap().to_owned(),
        }
    }
}

#[derive(Clone, Debug, Default, RustcDecodable, RustcEncodable)]
pub struct EmptyStruct {}

impl FromJson for EmptyStruct {
    fn from_json(_: &Value) -> Self {
        EmptyStruct {}
    }
}
