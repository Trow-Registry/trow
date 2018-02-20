use failure::Error;
use std;
use serde_json::Value;

pub trait FromJson {
    fn from_json(raw: &Value) -> Result<Self, Error>
    where
        Self: std::marker::Sized;
}

/*
    Manifests should be stored in internal format, but mirroring schema 2.2 which is the way forward.

    On input/output can be converted to appropriate format.
    Keep the code simple and in keeping with 2.2. Any hacks should be in the conversion to 2.1 code.

    I expect this code to change.
*/
#[derive(Debug, Default)]
pub struct Manifest {
    pub schema_version: u8,
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

fn schema_1(raw: &Value) -> Result<Manifest, Error> {
    let name = raw["name"].as_str().ok_or(InvalidManifest {
        err: "name is required".to_owned(),
    })?;
    let name = name.to_owned();
    let tag = raw["tag"].as_str().unwrap_or("latest").to_owned(); //Not sure this is correct
    let architecture = raw["architecture"].as_str().unwrap_or("amd64").to_owned();

    Ok(Manifest {
        schema_version: 1,
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
}

fn schema_2(raw: &Value) -> Result<Manifest, Error> {
    Ok(Manifest {
        schema_version: 2,
        name: "UNSUPPORTED".to_owned(),
        tag: "UNSUPPORTED".to_owned(),
        architecture: "UNSUPPORTED".to_owned(),
        fs_layers: Box::new(Vec::new()),
        signatures: Box::new(Vec::new()),
        history: Box::new(Vec::new()),
    })
}

impl FromJson for Manifest {
    fn from_json(raw: &Value) -> Result<Self, Error> {
        let schema_version = raw["schemaVersion"].as_u64().ok_or(InvalidManifest {
            err: "schemaVersion is required".to_owned(),
        })?;
        debug!("version {}", schema_version);
        match schema_version {
            1 => schema_1(raw),
            2 => schema_2(raw),
            n => Err(InvalidManifest{ err: format!("Unsupported version: {}", n).to_owned()})? //Something screwy here
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

        Ok(SignatureJWK { crv, kty, x, y })
    }
}

#[derive(Debug, Default)]
struct SignatureHeader {
    alg: String,
    jwk: SignatureJWK,
}

impl FromJson for SignatureHeader {
    fn from_json(raw: &Value) -> Result<Self, Error> {
        let alg = raw["alg"].as_str().ok_or(InvalidManifest {
            err: "alg is required".to_owned(),
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

        let payload = raw["payload"].as_str().ok_or(InvalidManifest {
            err: "payload is required".to_owned(),
        })?;
        let payload = payload.to_owned();
        let protected = raw["protected"].as_str().ok_or(InvalidManifest {
            err: "protected is required".to_owned(),
        })?;
        let protected = protected.to_owned();
        let signature = raw["signature"].as_str().ok_or(InvalidManifest {
            err: "signature is required".to_owned(),
        })?;
        let signature = signature.to_owned();

        Ok(Signature {
            header,
            payload,
            protected,
            signature,
        })
    }
}

#[derive(Clone, Debug, Default, RustcDecodable, RustcEncodable)]
pub struct BlobSummary {
    pub blob_sum: String,
}

impl FromJson for BlobSummary {
    fn from_json(raw: &Value) -> Result<Self, Error> {
        let blob_sum = raw["blobSum"].as_str().ok_or(InvalidManifest {
            err: "blobSum is required".to_owned(),
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

#[cfg(test)]
mod test {
    use serde_json::{self, Value};
    use super::Manifest;
    use super::FromJson;

    #[test]
    fn valid_v2() {
        let data = r#"{
     "schemaVersion": 2,
     "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
     "config": {
        "mediaType": "application/vnd.docker.container.image.v1+json",
        "size": 1278,
        "digest": "sha256:4a415e3663882fbc554ee830889c68a33b3585503892cc718a4698e91ef2a526"
     },
     "layers": [
        {
           "mediaType": "application/vnd.docker.image.rootfs.diff.tar.gzip",
           "size": 1967949,
           "digest": "sha256:1e76f742da490c8d7c921e811e5233def206e76683ee28d735397ec2231f131d"
        }
     ]
   }"#;
        let v: Value = serde_json::from_str(data).unwrap();
        assert!(Manifest::from_json(&v).is_ok());
    }

    #[test]
    fn valid_v1() {
        let data = r#"{
   "schemaVersion": 1,
   "name": "alpine",
   "tag": "latest",
   "architecture": "amd64",
   "fsLayers": [
      {
         "blobSum": "sha256:a3ed95caeb02ffe68cdd9fd84406680ae93d633cb16422d00e8a7c22955b46d4"
      },
      {
         "blobSum": "sha256:ff3a5c916c92643ff77519ffa742d3ec61b7f591b6b7504599d95a4a41134e28"
      }
   ],
   "history": [
   ],
   "signatures": [
      {
         "header": {
            "jwk": {
               "crv": "P-256",
               "kid": "65BS:JV6I:NAUR:VTRX:OIS4:MXBE:AILA:DM7L:E5AP:SF7J:373V:CNVZ",
               "kty": "EC",
               "x": "3iVq930taLCWgAbK8x4yYUDp_RBxpJnq3U0F-pQFLhU",
               "y": "yXBvhvMOk_ASBIt2btyRXZfpEgsjPyiWhA6whhJYOqo"
            },
            "alg": "ES256"
         },
         "signature": "pWC_DjgmtE5tO25WxDDQFFng1oHu3cg-8W8mKMk5ptZqS2AmoBOcD4441YhZQ02tuRae6vEInAuDguq0sKFrgg",
         "protected": "eyJmb3JtYXRMZW5ndGgiOjIxMzMsImZvcm1hdFRhaWwiOiJDbjAiLCJ0aW1lIjoiMjAxOC0wMi0yMFQxNDoxOTowNloifQ"
      }
   ]
    }"#;

        let v: Value = serde_json::from_str(data).unwrap();
        assert!(Manifest::from_json(&v).is_ok());
    }

}
