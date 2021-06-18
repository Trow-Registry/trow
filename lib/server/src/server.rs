use core::fmt::Display;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs::{self, DirEntry, File};
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::str;
use std::sync::{Arc, RwLock};

use chrono::prelude::*;
use failure::{self, Error, Fail};
use futures::TryFutureExt;
use prost_types::Timestamp;
use quoted_string::strip_dquotes;
use reqwest::header::ToStrError;
use reqwest::{
    self,
    header::{HeaderMap, HeaderValue},
    StatusCode,
};
use rustc_serialize::hex::ToHex;
use rustc_serialize::json::ToJson;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::digest::sha256_tag_digest;
use crate::manifest::{manifest_media_type, FromJson, Manifest};
use crate::metrics;
use crate::server::trow_server::registry_server::Registry;

use self::trow_server::*;

pub mod trow_server {
    include!("../../protobuf/out/trow.rs");
}

static SUPPORTED_DIGESTS: [&str; 1] = ["sha256"];
static MANIFESTS_DIR: &str = "manifests";
static BLOBS_DIR: &str = "blobs";
static UPLOADS_DIR: &str = "scratch";

static PROXY_DIR: &str = "f/"; //Repositories starting with this are considered proxies
static HUB_PROXY_DIR: &str = "docker/"; //Repositories starting with this are considered proxies
static HUB_ADDRESS: &str = "https://registry-1.docker.io/v2";
static DIGEST_HEADER: &str = "Docker-Content-Digest";

/* Struct implementing callbacks for the Frontend
 *
 * _active_uploads_: a HashSet of all uuids that are currently being tracked
 * _manifests_path_: path to where the manifests are
 * _layers_path_: path to where blobs are stored
 * _scratch_path_: path to temporary storage for uploads
 *
 * Each "route" gets a clone of this struct.
 * The Arc makes sure they all point to the same data.
 */
#[derive(Clone)]
pub struct TrowServer {
    active_uploads: Arc<RwLock<HashSet<Upload>>>,
    manifests_path: PathBuf,
    blobs_path: PathBuf,
    scratch_path: PathBuf,
    proxy_hub: bool,
    hub_user: Option<String>,
    hub_pass: Option<String>,
    allow_prefixes: Vec<String>,
    allow_images: Vec<String>,
    deny_local_prefixes: Vec<String>,
    deny_local_images: Vec<String>,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct Upload {
    repo_name: String,
    uuid: String,
}

#[derive(Fail, Debug)]
#[fail(display = "Error getting proxied repo {}", msg)]
pub struct ProxyError {
    msg: String,
}

#[derive(Fail, Debug)]
#[fail(display = "Expected digest {} but got {}", user_digest, actual_digest)]
pub struct DigestValidationError {
    user_digest: String,
    actual_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub host: String, //Including port, docker.io by default
    pub repo: String, //Between host and : including any /s
    pub tag: String,  //Bit after the :, latest by default
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}:{}", self.host, self.repo, self.tag)
    }
}

impl Image {
    fn get_manifest_url(&self) -> String {
        format!("{}/{}/manifests/{}", self.host, self.repo, self.tag)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Auth {
    pub user: Option<String>, //DockerHub has anon auth
    pub pass: Option<String>,
}

fn create_accept_header() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::ACCEPT,
        HeaderValue::from_str(&format!(
            "{}, {}",
            manifest_media_type::OCI_V1,
            manifest_media_type::DOCKER_V2
        ))
        .unwrap(),
    );
    headers
}

fn create_path(data_path: &str, dir: &str) -> Result<PathBuf, std::io::Error> {
    let data_path = Path::new(data_path);
    let dir_path = data_path.join(dir);
    if !dir_path.exists() {
        return match fs::create_dir_all(&dir_path) {
            Ok(_) => Ok(dir_path),
            Err(e) => {
                error!(
                    r#"
                Failed to create directory required by trow {:?}
                Please check the parent directory is writable by the trow user.
                {:?}"#,
                    dir_path, e
                );
                Err(e)
            }
        };
    };
    Ok(dir_path)
}

fn does_manifest_match_digest(manifest: &DirEntry, digest: &str) -> bool {
    digest
        == match get_digest_from_manifest_path(manifest.path()) {
            Ok(test_digest) => test_digest,
            Err(e) => {
                warn!("Failure reading repo {:?}", e);
                "NO_MATCH".to_string()
            }
        }
}

struct RepoIterator {
    paths: Vec<Result<DirEntry, std::io::Error>>,
}

impl RepoIterator {
    fn new(base_dir: &Path) -> Result<RepoIterator, Error> {
        let paths = fs::read_dir(base_dir)?.collect();
        Ok(RepoIterator { paths })
    }
}

impl Iterator for RepoIterator {
    type Item = DirEntry;
    fn next(&mut self) -> Option<Self::Item> {
        match self.paths.pop() {
            None => None,
            Some(res_path) => match res_path {
                Err(e) => {
                    warn!("Error iterating over repos {:?}", e);
                    self.next()
                }
                Ok(path) => {
                    if path.file_type().unwrap().is_dir() {
                        let new_paths = fs::read_dir(path.path()).unwrap();
                        self.paths.extend(new_paths);
                        self.next()
                    } else {
                        Some(path)
                    }
                }
            },
        }
    }
}

/**
 * Checks a file matches the given digest.
 *
 * TODO: should be able to use range of hashes.
 * TODO: check if using a static for the hasher speeds things up.
 */
fn validate_digest(file: &PathBuf, digest: &str) -> Result<(), Error> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);

    let calculated_digest = sha256_tag_digest(reader)?;

    if calculated_digest != digest {
        error!(
            "Upload did not match given digest. Was given {} but got {}",
            digest, calculated_digest
        );
        Err(DigestValidationError {
            user_digest: digest.to_string(),
            actual_digest: calculated_digest,
        })?;
    }

    Ok(())
}

fn is_digest(maybe_digest: &str) -> bool {
    for alg in &SUPPORTED_DIGESTS {
        if maybe_digest.starts_with(&format!("{}:", alg)) {
            return true;
        }
    }

    false
}

fn is_path_writable(path: &PathBuf) -> io::Result<bool> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let permissions = metadata.permissions();
    Ok(!permissions.readonly())
}

fn get_digest_from_manifest_path<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let digest_date = fs::read_to_string(path)?;
    //Should be digest followed by date, but allow for digest only
    Ok(digest_date
        .split(' ')
        .next()
        .unwrap_or(&digest_date)
        .to_string())
}

impl TrowServer {
    pub fn new(
        data_path: &str,
        proxy_hub: bool,
        hub_user: Option<String>,
        hub_pass: Option<String>,
        allow_prefixes: Vec<String>,
        allow_images: Vec<String>,
        deny_local_prefixes: Vec<String>,
        deny_local_images: Vec<String>,
    ) -> Result<Self, Error> {
        let manifests_path = create_path(data_path, MANIFESTS_DIR)?;
        let scratch_path = create_path(data_path, UPLOADS_DIR)?;
        let blobs_path = create_path(data_path, BLOBS_DIR)?;
        let svc = TrowServer {
            active_uploads: Arc::new(RwLock::new(HashSet::new())),
            manifests_path,
            blobs_path,
            scratch_path,
            proxy_hub,
            hub_user,
            hub_pass,
            allow_prefixes,
            allow_images,
            deny_local_prefixes,
            deny_local_images,
        };
        Ok(svc)
    }

    fn get_upload_path_for_blob(&self, uuid: &str) -> PathBuf {
        self.scratch_path.join(uuid)
    }

    fn get_catalog_path_for_blob(&self, digest: &str) -> Result<PathBuf, Error> {
        let mut iter = digest.split(':');
        let alg = iter
            .next()
            .ok_or_else(|| format_err!("Digest {} did not contain alg component", digest))?;
        if !SUPPORTED_DIGESTS.contains(&alg) {
            return Err(format_err!("Hash algorithm {} not supported", alg));
        }
        let val = iter
            .next()
            .ok_or_else(|| format_err!("Digest {} did not contain value component", digest))?;
        assert_eq!(None, iter.next());
        Ok(self.blobs_path.join(alg).join(val))
    }

    // Given a manifest digest, check if it is referenced by any tag in the repo
    fn verify_manifest_digest_in_repo(&self, repo_name: &str, digest: &str) -> Result<bool, Error> {
        let mut ri = RepoIterator::new(&self.manifests_path.join(repo_name))?;
        let res = ri.find(|de| does_manifest_match_digest(de, &digest));
        Ok(res.is_some())
    }

    fn get_digest_from_manifest(&self, repo_name: &str, tag: &str) -> Result<String, Error> {
        get_digest_from_manifest_path(self.manifests_path.join(repo_name).join(tag))
    }

    fn save_tag(&self, digest: &str, repo_name: &str, tag: &str) -> Result<(), Error> {
        // Tag files should contain list of digests with timestamp
        // First line should always be the current digest

        let repo_dir = self.manifests_path.join(repo_name);
        let repo_path = repo_dir.join(tag);
        fs::create_dir_all(&repo_dir)?;

        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
        let contents = format!("{} {}\n", digest, ts);

        if let Ok(mut f) = fs::File::open(&repo_path) {
            let mut buf = Vec::new();
            buf.extend(contents.as_bytes().iter());
            f.read_to_end(&mut buf)?;
            // TODO: Probably best to write to temporary file and then copy over.
            // Will be closer to atomic
            fs::write(&repo_path, buf)?;
        } else {
            fs::File::create(&repo_path).and_then(|mut f| f.write_all(contents.as_bytes()))?;
        }
        Ok(())
    }

    fn get_path_for_manifest(&self, repo_name: &str, reference: &str) -> Result<PathBuf, Error> {
        let digest = if is_digest(reference) {
            if !self.verify_manifest_digest_in_repo(repo_name, reference)? {
                error!("Digest {} not in repository {}", reference, repo_name);
                return Err(failure::err_msg(format!(
                    "Digest {} not in repository {}",
                    reference, repo_name
                )));
            }
            reference.to_string()
        } else {
            //Content of tag is the digest
            self.get_digest_from_manifest(repo_name, reference)?
        };

        self.get_catalog_path_for_blob(&digest)
    }

    fn create_verified_manifest(
        &self,
        manifest_path: &PathBuf,
        verify_assets_exist: bool,
    ) -> Result<VerifiedManifest, Error> {
        let manifest_bytes = std::fs::read(&manifest_path)?;
        let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
        let manifest = Manifest::from_json(&manifest_json)?;

        if verify_assets_exist {
            for digest in manifest.get_local_asset_digests() {
                let path = self.get_catalog_path_for_blob(digest)?;

                if !path.exists() {
                    return Err(format_err!(
                        "Failed to find artifact with digest {}",
                        digest
                    ));
                }
            }
        }

        // Calculate the digest: sha256:...
        let reader = BufReader::new(manifest_bytes.as_slice());
        let digest = sha256_tag_digest(reader)?;

        // For performance, could generate only if verification is on, otherwise copy from somewhere
        Ok(VerifiedManifest {
            digest,
            content_type: manifest.get_media_type(),
        })
    }

    /**
    If repo is proxied to another registry, this will return the details of the remote image.
    If the repo isn't proxied None is returned
    **/
    fn get_proxy_address_and_auth(
        &self,
        repo_name: &str,
        reference: &str,
    ) -> Option<(Image, Option<Auth>)> {
        //All proxies are under "f_"
        if repo_name.starts_with(PROXY_DIR) {
            let proxy_name = repo_name.strip_prefix(PROXY_DIR).unwrap();

            if self.proxy_hub && proxy_name.starts_with(HUB_PROXY_DIR) {
                let mut repo = proxy_name.strip_prefix(HUB_PROXY_DIR).unwrap().to_string();

                //Official images have to use the library/ repository
                if !repo.contains('/') {
                    repo = format!("library/{}", repo).to_string();
                }

                return Some((
                    Image {
                        host: HUB_ADDRESS.to_string(),
                        repo,
                        tag: reference.to_string(),
                    },
                    Some(Auth {
                        user: self.hub_user.clone(),
                        pass: self.hub_pass.clone(),
                    }),
                ));
            }
        }

        None
    }

    async fn download_manifest_and_layers<T: Display>(
        &self,
        cl: &reqwest::Client,
        token: &Option<T>,
        remote_image: &Image,
        local_repo_name: &str,
    ) -> Result<(), Error> {
        let mut resp = cl.get(&remote_image.get_manifest_url());
        if let Some(auth) = token {
            resp = resp.bearer_auth(auth);
        }

        let resp = resp.headers(create_accept_header()).send().await?;

        if !resp.status().is_success() {
            return Err(failure::err_msg(format!(
                "GET {} returned unexpected {}",
                &remote_image.get_manifest_url(),
                resp.status()
            )));
        }

        //First save as bytes
        let mani_id = Uuid::new_v4().to_string();
        let temp_mani_path = self.scratch_path.join(mani_id);
        let mut buf = File::create(&temp_mani_path)?;
        let bytes = resp.bytes().await?;
        buf.write_all(&bytes)?;

        let mani: Manifest = serde_json::from_slice(&bytes)?;

        let mut paths = vec![];
        //TODO: change to perform dloads async
        for digest in mani.get_local_asset_digests() {
            //skip only blob if it already exists in local storage
            //we need to continue as docker images may share blobs
            if self.get_catalog_path_for_blob(digest)?.exists() {
                info!("Already have blob {}", digest);
                continue;
            }
            let addr = format!(
                "{}/{}/blobs/{}",
                remote_image.host, remote_image.repo, digest
            );
            info!("Downloading blob {}", addr);

            let resp = if let Some(auth) = token {
                cl.get(&addr).bearer_auth(auth).send().await?
            } else {
                cl.get(&addr).send().await?
            };
            let path = self.scratch_path.join(digest);

            let mut buf = File::create(&path)?;
            buf.write_all(&resp.bytes().await?)?; //Is this going to be buffered?
            paths.push((path, digest));
        }

        for (path, digest) in &paths {
            self.save_blob(&path, digest)?;
        }

        //Save out manifest
        let f = File::open(&temp_mani_path)?;
        let reader = BufReader::new(f);
        let calculated_digest = sha256_tag_digest(reader)?;

        self.save_blob(&temp_mani_path, &calculated_digest)?;
        self.save_tag(&calculated_digest, local_repo_name, &remote_image.tag)?;

        //Delete all the temp stuff
        fs::remove_file(&temp_mani_path)
            .unwrap_or_else(|e| error!("Failure deleting downloaded manifest {:?}", e));

        for (path, _digest) in paths {
            fs::remove_file(path)
                .unwrap_or_else(|e| error!("Failure deleting downloaded blob {:?}", e));
        }

        Ok(())
    }

    async fn get_auth_token(
        &self,
        cl: &reqwest::Client,
        image: &Image,
        auth: &Option<Auth>,
    ) -> Result<String, Error> {
        let www_authenticate_header = self.get_www_authenticate_header(cl, image).await?;
        debug!("www_authenticate_header: {}", www_authenticate_header);
        self.get_auth_token_from_header(cl, auth, www_authenticate_header)
            .await
    }

    async fn get_auth_token_from_header(
        &self,
        cl: &reqwest::Client,
        auth: &Option<Auth>,
        www_authenticate_header: String,
    ) -> Result<String, Error> {
        let mut bearer_param_map = TrowServer::get_bearer_param_map(www_authenticate_header);

        let realm = bearer_param_map
            .get("realm")
            .cloned()
            .ok_or(format_err!("Realm should exists in Bearer header"))?;

        bearer_param_map.remove("realm");

        let reqBuilder = cl.get(realm.as_str()).query(&bearer_param_map);

        let reqBuilder = match auth {
            Some(a) => {
                if a.user.is_some() {
                    debug!("Auth is used with user: {:?}", a.user.as_ref().unwrap());
                    reqBuilder.basic_auth(&a.user.as_ref().unwrap(), a.pass.as_ref())
                } else {
                    reqBuilder
                }
            }
            None => reqBuilder,
        };

        let resp = reqBuilder.send().await.or_else(|e| {
            Err(format_err!(
                "Failed to send authenticate to {} request: {}",
                realm,
                e
            ))
        })?;

        if !resp.status().is_success() {
            return Err(format_err!("Failed to authenticate to {}", realm));
        }

        let auth_json_body = resp
            .json::<serde_json::Value>()
            .await
            .or_else(|e| Err(format_err!("Failed to deserialize auth response {}", e)))?;

        let access_token = auth_json_body
            .get("access_token")
            .ok_or(format_err!("Failed to find auth token in auth repsonse"))?;

        debug!("access_token: {}", access_token);

        access_token
            .as_str()
            .map(|s| s.to_string())
            .ok_or(format_err!("Access token string conversion failed"))
    }

    async fn get_www_authenticate_header(
        &self,
        cl: &reqwest::Client,
        image: &Image,
    ) -> Result<String, Error> {
        let resp = cl
            .head(&image.get_manifest_url())
            .headers(create_accept_header())
            .send()
            .await
            .or_else(|e| Err(format_err!("Request for authenticate failed: {}", e)))?;

        if resp.status() != 401 {
            return Err(format_err!(
                "Request '{}' should fail with status unauthorized",
                &image.get_manifest_url()
            ));
        }

        let value = resp.headers().get("www-authenticate").ok_or(format_err!(
            "Request header www-authenticate for authenticate should exists"
        ))?;

        value
            .to_str()
            .map(|s| String::from(s))
            .map_err(|e| format_err!("Access token string conversion failed: {}", e))
    }

    fn get_bearer_param_map(www_authenticate_header: String) -> HashMap<String, String> {
        let base = www_authenticate_header.strip_prefix("Bearer ");

        base.unwrap_or("")
            .split(',')
            .map(|kv| kv.split('=').collect::<Vec<&str>>())
            .map(|vec| {
                (
                    vec[0].to_string(),
                    strip_dquotes(vec[1]).unwrap().to_string(),
                )
            })
            .collect()
    }

    async fn get_digest_from_header(
        &self,
        cl: &reqwest::Client,
        image: &Image,
        auth_token: &Option<String>,
    ) -> Option<String> {
        let resp = if let Some(auth) = auth_token {
            cl.head(&image.get_manifest_url())
                .bearer_auth(&auth)
                .headers(create_accept_header())
                .send()
                .await
        } else {
            cl.head(&image.get_manifest_url())
                .headers(create_accept_header())
                .send()
                .await
        };

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                error!("Remote registry didn't respond to HEAD request {}", e);
                return None;
            }
        };

        if let Some(digest) = resp.headers().get(DIGEST_HEADER) {
            let digest = format!("{:?}", digest);
            Some(digest.trim_matches('"').to_string())
        } else {
            None
        }
    }

    async fn create_manifest_read_location(
        &self,
        repo_name: String,
        reference: String,
        do_verification: bool,
    ) -> Result<ManifestReadLocation, Error> {
        if let Some((proxy_image, proxy_auth)) =
            self.get_proxy_address_and_auth(&repo_name, &reference)
        {
            //TODO: May want to consider download tracking in case of simultaneous requests
            //In short term this isn't a big problem as should just copy over itself in worst case
            info!(
                "Request for proxied repo {}:{} maps to {}",
                repo_name, reference, proxy_image
            );

            let cl = reqwest::Client::new();

            let mut have_manifest = false;
            //Get auth token

            let auth_token = match self.get_auth_token(&cl, &proxy_image, &proxy_auth).await {
                Ok(a) => Some(a),
                Err(e) => {
                    error!("Can't get auth_token: {}", e);
                    None
                }
            };

            let digest = self
                .get_digest_from_header(&cl, &proxy_image, &auth_token)
                .await;

            if let Some(digest) = digest {
                if self.get_catalog_path_for_blob(&digest)?.exists() {
                    info!(
                        "Have up to date manifest for {} digest {}",
                        repo_name, digest
                    );
                    have_manifest = true;

                    //Make sure our tag exists and is up-to-date
                    if !is_digest(&reference) {
                        let our_digest = self.get_digest_from_manifest(&repo_name, &reference);
                        if our_digest.is_err() || (our_digest.unwrap() != digest) {
                            let res = self.save_tag(&digest, &repo_name, &reference);
                            if res.is_err() {
                                error!(
                                    "Internal error updating tag for proxied image {:?}",
                                    res.unwrap()
                                );
                            }
                        }
                    }
                }
            }

            if !have_manifest {
                if let Err(e) = self
                    .download_manifest_and_layers(&cl, &auth_token, &proxy_image, &repo_name)
                    .await
                {
                    //Note that we may still have an out-of-date version that will be returned
                    error!("Failed to download proxied image {}", e);
                }
            }
        }

        //TODO: This isn't optimal
        let path = self.get_path_for_manifest(&repo_name, &reference)?;
        let vm = self.create_verified_manifest(&path, do_verification)?;
        Ok(ManifestReadLocation {
            content_type: vm.content_type.to_owned(),
            digest: vm.digest,
            path: path.to_string_lossy().to_string(),
        })
    }

    fn save_blob(&self, scratch_path: &PathBuf, digest: &str) -> Result<(), Error> {
        let digest_path = self.get_catalog_path_for_blob(digest)?;
        let repo_path = digest_path
            .parent()
            .ok_or_else(|| failure::err_msg("Error finding repository path"))?;

        if !repo_path.exists() {
            fs::create_dir_all(repo_path)?;
        }

        fs::copy(&scratch_path, &digest_path)?;
        Ok(())
    }

    fn validate_and_save_blob(&self, user_digest: &str, uuid: &str) -> Result<(), Error> {
        debug!("Saving blob {}", user_digest);

        let scratch_path = self.get_upload_path_for_blob(uuid);
        let res = match validate_digest(&scratch_path, user_digest) {
            Ok(_) => self.save_blob(&scratch_path, user_digest),
            Err(e) => Err(e),
        };

        //Not an error, even if it's not great
        fs::remove_file(&scratch_path).unwrap_or_else(|e| {
            error!(
                "Error deleting file {} {:?}",
                &scratch_path.to_string_lossy(),
                e
            )
        });

        res?;
        Ok(())
    }

    //Support functions for validate, would like to move these
    pub fn image_exists(&self, image: &Image) -> bool {
        match self.get_path_for_manifest(&image.repo, &image.tag) {
            Ok(f) => f.exists(),
            Err(_) => false,
        }
    }

    pub fn is_local_denied(&self, image: &Image) -> bool {
        //Try matching both with and without host name
        //Deny images are expected without host as always local
        let full_name = format!("{}", image);
        let name_without_host = format!("{}:{}", image.repo, image.tag);

        for prefix in &self.deny_local_prefixes {
            if full_name.starts_with(prefix) || name_without_host.starts_with(prefix) {
                info!("Image {} matches prefix {} on deny list", image, prefix);
                return true;
            }
        }

        for name in &self.deny_local_images {
            if &full_name == name || &name_without_host == name {
                info!("Image {} matches image {} on deny list", image, name);
                return true;
            }
        }

        false
    }

    pub fn is_allowed(&self, image: &Image) -> bool {
        //Have full names with host here
        let name = format!("{}", image);

        for prefix in &self.allow_prefixes {
            if name.starts_with(prefix) {
                info!("Image {} matches prefix {} on allow list", name, prefix);
                return true;
            }
        }

        for a_name in &self.allow_images {
            if &name == a_name {
                info!("Image {} matches image {} on allow list", name, a_name);
                return true;
            }
        }

        false
    }

    fn is_writable_repo(&self, repo_name: &str) -> bool {
        if repo_name.starts_with(PROXY_DIR) {
            return false;
        }

        true
    }
}

#[tonic::async_trait]
impl Registry for TrowServer {
    async fn request_upload(
        &self,
        request: Request<UploadRequest>,
    ) -> Result<Response<UploadDetails>, Status> {
        let repo_name = request.into_inner().repo_name;
        if self.is_writable_repo(&repo_name) {
            let uuid = Uuid::new_v4().to_string();
            let reply = UploadDetails { uuid: uuid.clone() };
            let upload = Upload { repo_name, uuid };
            {
                self.active_uploads.write().unwrap().insert(upload);
                debug!("Upload Table: {:?}", self.active_uploads);
            }
            Ok(Response::new(reply))
        } else {
            Err(Status::invalid_argument(format!(
                "Repository {} is not writable",
                repo_name
            )))
        }
    }

    async fn get_write_location_for_blob(
        &self,
        req: Request<UploadRef>,
    ) -> Result<Response<WriteLocation>, Status> {
        let br = req.into_inner();
        let upload = Upload {
            repo_name: br.repo_name.clone(),
            uuid: br.uuid.clone(),
        };

        // Apparently unwrap() is correct here. From the docs:
        // "We unwrap() the return value to assert that we are not expecting
        // threads to ever fail while holding the lock."

        let set = self.active_uploads.read().unwrap();
        if set.contains(&upload) {
            let path = self.get_upload_path_for_blob(&br.uuid);
            Ok(Response::new(WriteLocation {
                path: path.to_string_lossy().to_string(),
            }))
        } else {
            Err(Status::failed_precondition(format!(
                "No current upload matching {:?}",
                br
            )))
        }
    }

    async fn get_read_location_for_blob(
        &self,
        req: Request<BlobRef>,
    ) -> Result<Response<BlobReadLocation>, Status> {
        metrics::TOTAL_BLOB_REQUESTS.inc();
        let br = req.into_inner();
        let path = self
            .get_catalog_path_for_blob(&br.digest)
            .map_err(|e| Status::invalid_argument(format!("Error parsing digest {:?}", e)))?;

        if !path.exists() {
            warn!("Request for unknown blob: {:?}", path);
            Err(Status::not_found(format!(
                "No blob found matching {:?}",
                br
            )))
        } else {
            Ok(Response::new(BlobReadLocation {
                path: path.to_string_lossy().to_string(),
            }))
        }
    }

    /**
     * TODO: check if blob referenced by manifests. If so, refuse to delete.
     */
    async fn delete_blob(&self, req: Request<BlobRef>) -> Result<Response<BlobDeleted>, Status> {
        let br = req.into_inner();
        let path = self
            .get_catalog_path_for_blob(&br.digest)
            .map_err(|e| Status::invalid_argument(format!("Error parsing digest {:?}", e)))?;
        if !path.exists() {
            warn!("Request for unknown blob: {:?}", path);
            Err(Status::not_found(format!(
                "No blob found matching {:?}",
                br
            )))
        } else {
            fs::remove_file(&path)
                .map_err(|e| {
                    error!("Failed to delete blob {:?} {:?}", br, e);
                    Status::internal("Internal error deleting blob")
                })
                .and(Ok(Response::new(BlobDeleted {})))
        }
    }

    async fn delete_manifest(
        &self,
        req: Request<ManifestRef>,
    ) -> Result<Response<ManifestDeleted>, Status> {
        let mr = req.into_inner();
        if !is_digest(&mr.reference) {
            return Err(Status::invalid_argument(format!(
                "Manifests can only be deleted by digest. Got {}",
                mr.reference
            )));
        }
        let digest = mr.reference;
        //For the repo, go through all tags and see if they reference the digest. Delete them.
        //Can only delete manifest if no other tags in any repo reference it

        let ri = RepoIterator::new(&self.manifests_path.join(&mr.repo_name)).map_err(|e| {
            error!("Problem reading manifest catalog {:?}", e);
            Status::failed_precondition("Repository not found")
        })?;

        //TODO: error if no manifest matches?
        ri.filter(|de| does_manifest_match_digest(de, &digest))
            .for_each(|man| match fs::remove_file(man.path()) {
                Ok(_) => (),
                Err(e) => error!("Failed to delete manifest {:?} {:?}", &man, e),
            });

        Ok(Response::new(ManifestDeleted {}))
    }

    async fn get_write_details_for_manifest(
        &self,
        req: Request<ManifestRef>,
    ) -> Result<Response<ManifestWriteDetails>, Status> {
        let repo_name = req.into_inner().repo_name;
        if self.is_writable_repo(&repo_name) {
            //Give the manifest a UUID and save it to the uploads dir
            let uuid = Uuid::new_v4().to_string();

            let manifest_path = self.get_upload_path_for_blob(&uuid);
            Ok(Response::new(ManifestWriteDetails {
                path: manifest_path.to_string_lossy().to_string(),
                uuid,
            }))
        } else {
            Err(Status::invalid_argument(format!(
                "Repository {} is not writable",
                repo_name
            )))
        }
    }

    async fn get_read_location_for_manifest(
        &self,
        req: Request<ManifestRef>,
    ) -> Result<Response<ManifestReadLocation>, Status> {
        //Don't actually need to verify here; could set to false

        let mr = req.into_inner();
        metrics::TOTAL_MANIFEST_REQUESTS.inc();
        // TODO refactor to return directly
        match self
            .create_manifest_read_location(mr.repo_name, mr.reference, true)
            .await
        {
            Ok(vm) => Ok(Response::new(vm)),
            Err(e) => {
                warn!("Internal error with manifest {:?}", e);
                Err(Status::internal("Internal error finding manifest"))
            }
        }
    }

    /**
     * Take uploaded manifest (which should be uuid in uploads), check it, put in catalog and
     * by blob digest
     */
    async fn verify_manifest(
        &self,
        req: Request<VerifyManifestRequest>,
    ) -> Result<Response<VerifiedManifest>, Status> {
        let req = req.into_inner();
        let mr = req.manifest.unwrap(); // Pissed off that the manifest is optional!
        let uploaded_manifest = self.get_upload_path_for_blob(&req.uuid);

        match self.create_verified_manifest(&uploaded_manifest, true) {
            Ok(vm) => {
                // copy manifest to blobs and add tag
                let digest = vm.digest.clone();
                let ret = self
                    .save_blob(&uploaded_manifest, &digest)
                    .and(self.save_tag(&digest, &mr.repo_name, &mr.reference))
                    .map(|_| Response::new(vm))
                    .map_err(|e| {
                        error!(
                            "Failure cataloguing manifest {}/{} {:?}",
                            &mr.repo_name, &mr.reference, e
                        );
                        Status::internal("Internal error copying manifest")
                    });

                fs::remove_file(&uploaded_manifest)
                    .unwrap_or_else(|e| error!("Failure deleting uploaded manifest {:?}", e));

                ret
            }
            Err(e) => {
                error!("Error verifying manifest {:?}", e);
                Err(Status::invalid_argument("Failed to verify manifest"))
            }
        }
    }

    async fn complete_upload(
        &self,
        req: Request<CompleteRequest>,
    ) -> Result<Response<CompletedUpload>, Status> {
        let cr = req.into_inner();
        let ret = match self.validate_and_save_blob(&cr.user_digest, &cr.uuid) {
            Ok(_) => Ok(Response::new(CompletedUpload {
                digest: cr.user_digest.clone(),
            })),
            Err(e) => match e.downcast::<DigestValidationError>() {
                Ok(v_e) => Err(Status::invalid_argument(v_e.to_string())),
                Err(e) => {
                    warn!("Failure when saving layer: {:?}", e);
                    Err(Status::internal("Internal error saving layer"))
                }
            },
        };

        //delete uuid from uploads tracking
        let upload = Upload {
            repo_name: cr.repo_name.clone(),
            uuid: cr.uuid,
        };

        let mut set = self.active_uploads.write().unwrap();
        if !set.remove(&upload) {
            warn!("Upload {:?} not found when deleting", upload);
        }
        ret
    }

    type GetCatalogStream = ReceiverStream<Result<CatalogEntry, Status>>;

    async fn get_catalog(
        &self,
        request: Request<CatalogRequest>,
    ) -> Result<Response<Self::GetCatalogStream>, Status> {
        let cr = request.into_inner();
        let limit = cr.limit as usize;

        let (tx, rx) = mpsc::channel(4);
        let catalog: HashSet<String> = RepoIterator::new(&self.manifests_path)
            .map_err(|e| {
                error!("Error accessing catalog {:?}", e);
                Status::internal("Internal error streaming catalog")
            })?
            .map(|de| de.path())
            .filter_map(|p| p.parent().map(|p| p.to_path_buf()))
            .filter_map(|r| {
                r.strip_prefix(&self.manifests_path)
                    .ok()
                    .map(|p| p.to_path_buf())
            })
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        let partial_catalog: Vec<String> = if cr.last_repo.is_empty() {
            catalog.into_iter().take(limit).collect()
        } else {
            catalog
                .into_iter()
                .skip_while(|t| t != &cr.last_repo)
                .skip(1)
                .take(limit)
                .collect()
        };

        tokio::spawn(async move {
            for repo_name in partial_catalog {
                let ce = CatalogEntry { repo_name };

                tx.send(Ok(ce)).await.expect("Error streaming catalog");
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type ListTagsStream = ReceiverStream<Result<Tag, Status>>;

    async fn list_tags(
        &self,
        request: Request<ListTagsRequest>,
    ) -> Result<Response<Self::ListTagsStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
        let mut path = PathBuf::from(&self.manifests_path);

        let ltr = request.into_inner();

        let limit = ltr.limit as usize;
        path.push(&ltr.repo_name);

        let mut catalog: Vec<String> = RepoIterator::new(&path)
            .map_err(|e| {
                error!("Error accessing catalog {:?}", e);
                Status::internal("Internal error streaming catalog")
            })?
            .map(|de| de.path().file_name().unwrap().to_string_lossy().to_string())
            .collect();
        catalog.sort();
        let partial_catalog: Vec<String> = if ltr.last_tag.is_empty() {
            catalog.into_iter().take(limit).collect()
        } else {
            catalog
                .into_iter()
                .skip_while(|t| t != &ltr.last_tag)
                .skip(1)
                .take(limit)
                .collect()
        };

        tokio::spawn(async move {
            for tag in partial_catalog {
                tx.send(Ok(Tag {
                    tag: tag.to_string(),
                }))
                .await
                .expect("Error streaming tags");
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type GetManifestHistoryStream = ReceiverStream<Result<ManifestHistoryEntry, Status>>;

    async fn get_manifest_history(
        &self,
        request: Request<ManifestHistoryRequest>,
    ) -> Result<Response<Self::GetManifestHistoryStream>, Status> {
        let mr = request.into_inner();
        if is_digest(&mr.tag) {
            return Err(Status::invalid_argument(
                "Require valid tag (not digest) to search for history",
            ));
        }

        let manifest_path = self.manifests_path.join(&mr.repo_name).join(&mr.tag);

        let file = File::open(&manifest_path);

        if file.is_err() {
            return Err(Status::not_found(format!(
                "Could not find the requested manifest at: {}",
                &manifest_path.to_str().unwrap()
            )));
        }

        // It's safe to unwrap here
        let reader = BufReader::new(file.unwrap());

        let (tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            let mut searching_for_digest = mr.last_digest != ""; //Looking for a digest iff it's not empty

            let mut sent = 0;
            for line in reader.lines() {
                if let Ok(line) = line {
                    let (digest, date) = match line.find(' ') {
                        Some(ind) => {
                            let (digest_str, date_str) = line.split_at(ind);

                            if searching_for_digest {
                                if digest_str == mr.last_digest {
                                    searching_for_digest = false;
                                }
                                //Remember we want digest following matched digest
                                continue;
                            }

                            let dt_r = DateTime::parse_from_rfc3339(date_str.trim());

                            let ts = if let Ok(dt) = dt_r {
                                Some(Timestamp {
                                    seconds: dt.timestamp(),
                                    nanos: dt.timestamp_subsec_nanos() as i32,
                                })
                            } else {
                                warn!("Failed to parse timestamp {}", date_str);
                                None
                            };
                            (digest_str, ts)
                        }
                        None => {
                            warn!("No timestamp found in manifest");
                            (line.as_ref(), None)
                        }
                    };

                    let entry = ManifestHistoryEntry {
                        digest: digest.to_string(),
                        date,
                    };
                    tx.send(Ok(entry))
                        .await
                        .expect("Error streaming manifest history");

                    sent += 1;
                    if sent >= mr.limit {
                        break;
                    }
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    // Readiness check
    async fn is_ready(
        &self,
        _request: Request<ReadinessRequest>,
    ) -> Result<Response<ReadyStatus>, Status> {
        for path in &[&self.scratch_path, &self.manifests_path, &self.blobs_path] {
            match is_path_writable(path) {
                Ok(true) => {}
                Ok(false) => {
                    return Err(Status::unavailable(format!(
                        "{} is not writable",
                        path.to_string_lossy()
                    )));
                }
                Err(error) => {
                    return Err(Status::unavailable(error.to_string()));
                }
            }
        }

        //All paths writable
        let reply = trow_server::ReadyStatus {
            message: String::from("Ready"),
        };

        Ok(Response::new(reply))
    }

    async fn is_healthy(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthStatus>, Status> {
        let reply = trow_server::HealthStatus {
            message: String::from("Healthy"),
        };
        Ok(Response::new(reply))
    }

    async fn get_metrics(
        &self,
        _request: Request<MetricsRequest>,
    ) -> Result<Response<MetricsResponse>, Status> {
        match metrics::gather_metrics(&self.blobs_path) {
            Ok(metrics) => {
                let reply = trow_server::MetricsResponse { metrics };
                Ok(Response::new(reply))
            }

            Err(error) => Err(Status::unavailable(error.to_string())),
        }
    }
}
