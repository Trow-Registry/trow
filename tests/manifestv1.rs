
extern crate jwt;
extern crate rustc_serialize;
extern crate crypto;
    

#[cfg(test)]
mod manifestv1_tests {

    use crypto::sha2::Sha256;
    // fn generate_manifest(digest: &str) -> Result<Value, ()> {
    //     use std::fs::File;
    //     use std::io::Read;
    //     let mut template = File::open(MANIFEST_TEMPLATE).unwrap();
    //     let mut contents = String::new();
    //     template.read_to_string(&mut contents).unwrap();
    //     let mut v: Value = serde_json::from_str(&contents).unwrap();
    //     v["fsLayers"][0]["blobSum"] = Value::String(digest.to_string());
    //     Ok(v)
    // }

    #[derive(Clone, Debug, Default, RustcDecodable, RustcEncodable)]
    struct EmptyStruct {}

    #[derive(Clone, Debug, Default, RustcDecodable, RustcEncodable)]
    #[allow(non_snake_case)]
    struct BlobSummary {
        blobSum: String
    }

    #[derive(Debug, RustcDecodable, RustcEncodable)]
    #[allow(non_snake_case)]
    struct SigningManifest {
        schemaVersion: u8,
        name: String,
        tag: String,
        architecture: String,
        fsLayers: Vec<BlobSummary>,
        history: Vec<EmptyStruct>
    }

    impl Default for SigningManifest {
        fn default() -> Self {
            SigningManifest {
                schemaVersion: 1,
                name: "test/test".to_owned(),
                tag: "latest".to_owned(),
                architecture: "amd64".to_owned(),
                fsLayers: Default::default(),
                history: Default::default(),
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

    #[derive(Debug, Default)]
    struct SignatureHeader {
        alg: String,
        jwk: SignatureJWK
    }

    #[derive(Debug, Default)]
    struct Signature {
        header: SignatureHeader,
        payload: String,
        protected: String,
        signature: String,
    }

    #[derive(Debug, Default)]
    #[allow(non_snake_case)]
    struct Manifest {
        schemaVersion: u8,
        name: String,
        tag: String,
        architecture: String,
        fsLayers: Vec<BlobSummary>,
        history: Vec<EmptyStruct>,
        signatures: Vec<Signature>,

    }

    #[test]
    fn upload_manifest() {
        // manifest with invalid layer
        sign_manifest("invalid_digest");
        // manifest with test/test/test_digest layer
    }


    fn sign_manifest(digest: &str) {
        // copying from https://github.com/ContainerSolutions/manifest-sample-python/blob/master/construct-manifest.py
        use jwt::{Header, Token};
        // let format_length = manifest_str.len() - 1;
        // let format_tail = '}';
        let header: Header = Default::default();
        let claims = SigningManifest {
            fsLayers: vec!(BlobSummary { blobSum: digest.to_owned() }),
            ..Default::default()
        };

        let manifest = Manifest::from_signing_manifest(&claims);
        let token = Token::new(header, claims);


        println!("{:?}", manifest);
        let signed = token.signed(b"secret_key", Sha256::new()).ok();
        println!("{:?}", signed);
    }

    impl Manifest {
        /* transformation function from SigningManifest -> Manifest
         * Possibly would be worth it to think about being able to
         * statically catch incorrect transformations when structs
         * change shape. ie. missed values in the transformation...
         */
        fn from_signing_manifest(sig: &SigningManifest) -> Self {
            Manifest {
                schemaVersion: sig.schemaVersion.to_owned(),
                name: sig.name.to_owned(),
                tag: sig.tag.to_owned(),
                architecture: sig.architecture.to_owned(),
                fsLayers: sig.fsLayers.clone(),
                history: sig.history.clone(),

                signatures: Default::default(),
            }
        }

    }
}