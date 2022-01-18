use rand::Rng;
use reqwest::StatusCode;
use std::io::BufReader;
use std::process::Child;
use trow_server::digest;
use trow_server::manifest;

/* None of these are dead code, they are called from tests */

#[allow(dead_code)]
pub const TROW_ADDRESS: &str = "https://trow.test:8443";
#[allow(dead_code)]
pub const DIST_API_HEADER: &str = "Docker-Distribution-API-Version";
#[allow(dead_code)]
pub const UPLOAD_HEADER: &str = "Docker-Upload-Uuid";
#[allow(dead_code)]
pub const LOCATION_HEADER: &str = "Location";

#[cfg(test)]
#[allow(dead_code)]
pub fn gen_rand_blob(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut blob = Vec::with_capacity(size);
    for _ in 0..size {
        blob.push(rng.gen::<u8>());
    }
    blob
}

// https://stackoverflow.com/questions/49210815/how-do-i-send-a-signal-to-a-child-subprocess
#[cfg(test)]
#[allow(dead_code)]
pub fn kill_gracefully(child: &Child) {
    unsafe {
        libc::kill(child.id() as i32, libc::SIGTERM);
    }
}

#[cfg(test)]
#[allow(dead_code)]
pub async fn upload_layer(cl: &reqwest::Client, name: &str, tag: &str) {
    let resp = cl
        .post(&format!("{}/v2/{}/blobs/uploads/", TROW_ADDRESS, name))
        .send()
        .await
        .expect("Error uploading layer");
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    let uuid = resp.headers().get(UPLOAD_HEADER).unwrap().to_str().unwrap();
    let location = resp
        .headers()
        .get(LOCATION_HEADER)
        .unwrap()
        .to_str()
        .unwrap();

    //Upload file. Start uploading blob with patch then digest with put
    let blob = gen_rand_blob(100);
    let resp = cl
        .patch(location)
        .body(blob.clone())
        .send()
        .await
        .expect("Failed to send patch request");
    assert_eq!(resp.status(), StatusCode::ACCEPTED);

    let digest = digest::sha256_tag_digest(BufReader::new(blob.as_slice())).unwrap();
    let resp = cl
        .put(&format!(
            "{}/v2/{}/blobs/uploads/{}?digest={}",
            TROW_ADDRESS, name, uuid, digest
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    //Finally get it back again
    let resp = cl
        .get(&format!("{}/v2/{}/blobs/{}", TROW_ADDRESS, name, digest))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let digest_header = resp
        .headers()
        .get("Docker-Content-Digest")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(digest, digest_header);
    assert_eq!(blob, resp.bytes().await.unwrap());

    //Upload manifest
    //For time being use same blob for config and layer
    let config = manifest::Object {
        media_type: "application/vnd.docker.container.image.v1+json".to_owned(),
        size: Some(blob.len() as u64),
        digest: digest.clone(),
    };
    let layer = manifest::Object {
        media_type: "application/vnd.docker.image.rootfs.diff.tar.gzip".to_owned(),
        size: Some(blob.len() as u64),
        digest: digest.clone(),
    };
    let layers = vec![layer];
    let mani = manifest::ManifestV2 {
        schema_version: 2,
        media_type: Some("application/vnd.docker.distribution.manifest.v2+json".to_owned()),
        config,
        layers,
    };
    let manifest_addr = format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, tag);
    let resp = cl.put(&manifest_addr).json(&mani).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let location = resp.headers().get("Location").unwrap().to_str().unwrap();
    assert_eq!(&location, &manifest_addr);
}
