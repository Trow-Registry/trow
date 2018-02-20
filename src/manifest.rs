use failure::Error;
use std;
use serde_json::Value;

pub trait FromJson {
    fn from_json(raw: &Value) -> Result<Self, Error>
    where
        Self: std::marker::Sized;
}

#[derive(Debug, Default)]
pub struct Manifest {
    pub schema_version: u64,
    pub name: String,
    pub tag: String,
    pub architecture: String,
    pub fs_layers: Box<Vec<BlobSummary>>,
    pub history: Box<Vec<EmptyStruct>>,
    pub signatures: Box<Vec<Signature>>,
}

#[derive(Fail, Debug)]
#[fail(display = "Invalid Manifest: {}", err)]
pub struct InvalidManifest {
    err: String,
}

impl FromJson for Manifest {
    fn from_json(raw: &Value) -> Result<Self, Error> {
        let schema_version = raw["schemaVersion"].as_u64().ok_or(InvalidManifest {
            err: "schemaVersion is required".to_owned(),
        })?;
        let name = raw["name"].as_str().ok_or(InvalidManifest {
            err: "name is required".to_owned(),
        })?;
        let name = name.to_owned();
        let tag = raw["tag"].as_str().unwrap_or("latest").to_owned(); //Not sure this is correct
        let architecture = raw["architecture"].as_str().unwrap_or("amd64").to_owned();

        debug!("version {}", schema_version);
        if schema_version == 1 {
            Ok(Manifest {
                schema_version,
                name,
                tag,
                architecture,
                /*
                fsLayers: BlobSummary::from_json_map(&raw["fsLayers"]),
                signatures: Signature::from_json_map(&raw["signatures"]),
                history: EmptyStruct::from_json_map(&raw["history"]),
                */
                fs_layers: Box::new(Vec::new()),
                signatures: Box::new(Vec::new()),
                history: Box::new(Vec::new()),
            })
        } else {
            Ok(Manifest {
                schema_version: raw["schemaVersion"].as_u64().unwrap(),
                name: "UNSUPPORTED".to_owned(),
                tag: "UNSUPPORTED".to_owned(),
                architecture: "UNSUPPORTED".to_owned(),
                fs_layers: Box::new(Vec::new()),
                signatures: Box::new(Vec::new()),
                history: Box::new(Vec::new()),
            })
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
    fn from_json(raw: &Value) -> Result<Self, Error> {

        let crv = raw["crv"].as_str().ok_or(InvalidManifest {
            err: "crv is required".to_owned(),
        })?;
        let crv = crv.to_owned();
        let kty = raw["kty"].as_str().ok_or(InvalidManifest {
            err: "kty is required".to_owned(),
        })?;
        let kty = kty.to_owned();
        let x = raw["x"].as_str().ok_or(InvalidManifest {
            err: "x is required".to_owned(),
        })?;
        let x = x.to_owned();
        let y = raw["y"].as_str().ok_or(InvalidManifest {
            err: "y is required".to_owned(),
        })?;
        let y = y.to_owned();

        Ok(SignatureJWK {crv, kty, x, y})
    }
}

#[derive(Debug, Default)]
struct SignatureHeader {
    alg: String,
    jwk: SignatureJWK,
}

impl FromJson for SignatureHeader {
    fn from_json(raw: &Value) -> Result<Self, Error> {

        let alg = raw["alg"].as_str().ok_or(InvalidManifest{
            err: "alg is required".to_owned()
        })?;
        let alg = alg.to_owned();
        let jwk = SignatureJWK::from_json(&raw["jwk"])?;
     
        Ok(SignatureHeader { alg, jwk })
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
    fn from_json(raw: &Value) -> Result<Self, Error> {

        let header = SignatureHeader::from_json(&raw["header"])?; 

        let payload = raw["payload"].as_str().ok_or(InvalidManifest{
            err: "payload is required".to_owned()
        })?;
        let payload = payload.to_owned();
        let protected = raw["protected"].as_str().ok_or(InvalidManifest{
            err: "protected is required".to_owned()
        })?;
        let protected = protected.to_owned();
        let signature = raw["signature"].as_str().ok_or(InvalidManifest{
            err: "signature is required".to_owned()
        })?;
        let signature = signature.to_owned();

        Ok(Signature { header, payload, protected, signature })
    }
}

#[derive(Clone, Debug, Default, RustcDecodable, RustcEncodable)]
pub struct BlobSummary {
    pub blob_sum: String,
}

impl FromJson for BlobSummary {
    fn from_json(raw: &Value) -> Result<Self, Error> {

        let blob_sum = raw["blobSum"].as_str().ok_or(InvalidManifest{
            err: "blobSum is required".to_owned()
        })?;
        let blob_sum = blob_sum.to_owned();
        Ok(BlobSummary { blob_sum })
    }
}

#[derive(Clone, Debug, Default, RustcDecodable, RustcEncodable)]
pub struct EmptyStruct {}

impl FromJson for EmptyStruct {
    fn from_json(_: &Value) -> Result<Self, Error> {
        Ok(EmptyStruct {})
    }
}
