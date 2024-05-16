use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use async_recursion::async_recursion;
use aws_config::BehaviorVersion;
use base64::engine::general_purpose;
use base64::Engine as _;
use futures::future::try_join_all;
use lazy_static::lazy_static;
use quoted_string::strip_dquotes;
use regex::Regex;
use reqwest::{self, Method, StatusCode};
use tracing::{event, Level};

use super::create_accept_header;
use super::proxy_config::SingleRegistryProxyConfig;
use super::remote_image::RemoteImage;
use crate::registry::manifest::{Manifest, OCIManifest};
use crate::registry::{Digest, TrowServer};

const AUTHN_HEADER: &str = "www-authenticate";
static DIGEST_HEADER: &str = "Docker-Content-Digest";

#[derive(Debug)]
pub enum HttpAuth {
    Basic(String, Option<String>),
    Bearer(String),
    None,
}

impl HttpAuth {
    pub async fn bearer(
        proxy_cfg: &SingleRegistryProxyConfig,
        cl: &reqwest::Client,
        authn_header: &str,
    ) -> Result<Self> {
        let tok = get_bearer_auth_token(cl, authn_header, proxy_cfg)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to get bearer auth token for {}: {}",
                    proxy_cfg.host,
                    e
                )
            })?;

        Ok(Self::Bearer(tok))
    }
}

/// Wrapper around `reqwest::Client` that automagically handles authentication
/// to other container registries
pub struct ProxyClient {
    cl: reqwest::Client,
    auth: HttpAuth,
    remote_image: RemoteImage,
}

impl ProxyClient {
    pub async fn try_new(
        mut proxy_cfg: SingleRegistryProxyConfig,
        remote_image: &RemoteImage,
    ) -> Result<Self> {
        let base_client = reqwest::ClientBuilder::new()
            // .connect_timeout(Duration::from_millis(1000))
            .build()?;

        let authn_header = get_www_authenticate_header(&base_client, remote_image).await?;

        if proxy_cfg.host.contains(".dkr.ecr.")
            && proxy_cfg.host.contains(".amazonaws.com")
            && matches!(&proxy_cfg.username, Some(u) if u == "AWS")
            && proxy_cfg.password.is_none()
        {
            let passwd = get_aws_ecr_password_from_env(&proxy_cfg.host)
                .await
                .context("Could not fetch password to ECR registry")?;
            proxy_cfg.password = Some(passwd);
        }

        let auth = match authn_header {
            Some(h) if h.starts_with("Basic") => {
                if proxy_cfg.username.is_none() {
                    return Err(anyhow!(
                        "Registry `{}` requires Basic auth but no username was provided",
                        proxy_cfg.host
                    ));
                }
                HttpAuth::Basic(
                    proxy_cfg.username.clone().unwrap(),
                    proxy_cfg.password.clone(),
                )
            }
            Some(h) if h.starts_with("Bearer") => {
                HttpAuth::bearer(&proxy_cfg, &base_client, &h).await?
            }
            None => HttpAuth::None,

            Some(invalid_header) => {
                return Err(anyhow!(
                    "Could not parse {AUTHN_HEADER} of registry `{}`: `{}`",
                    proxy_cfg.host,
                    invalid_header
                ))
            }
        };

        Ok(Self {
            cl: base_client,
            auth,
            remote_image: remote_image.clone(),
        })
    }

    /// Build a request with added authentication.
    /// The auth method will vary depending on the registry being queried.
    fn authenticated_request(&self, method: Method, url: &str) -> reqwest::RequestBuilder {
        let req = self.cl.request(method, url);
        match &self.auth {
            HttpAuth::Basic(username, password) => req.basic_auth(username, password.to_owned()),
            HttpAuth::Bearer(token) => req.bearer_auth(token),
            HttpAuth::None => req,
        }
    }

    /// Download a blob that is part of `remote_image`.
    async fn download_blob(&self, registry: &TrowServer, digest: &Digest) -> Result<()> {
        if registry
            .storage
            .get_blob_stream(self.remote_image.get_repo(), digest)
            .await
            .is_ok()
        {
            event!(Level::DEBUG, "Already have blob {}", digest);
            return Ok(());
        }
        let addr = format!("{}/blobs/{}", self.remote_image.get_base_uri(), digest);
        event!(Level::INFO, "Downloading blob {}", addr);
        let resp = self
            .authenticated_request(Method::GET, &addr)
            .send()
            .await
            .context("GET blob failed")?;
        registry
            .storage
            .write_blob_stream(digest, resp.bytes_stream(), true)
            .await
            .context("Failed to write blob")?;
        Ok(())
    }

    #[async_recursion]
    pub async fn download_manifest_and_layers(
        &self,
        registry: &TrowServer,
        remote_image: &RemoteImage,
        local_repo_name: &str,
    ) -> Result<()> {
        event!(
            Level::DEBUG,
            "Downloading manifest + layers for {}",
            remote_image
        );
        let resp = self
            .authenticated_request(Method::GET, &remote_image.get_manifest_url())
            .headers(create_accept_header())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!(
                "GET {} returned unexpected {}",
                &remote_image.get_manifest_url(),
                resp.status()
            ));
        }
        let bytes = resp.bytes().await?;
        let mani = Manifest::from_bytes(bytes)?;
        match mani.parsed() {
            OCIManifest::List(_) => {
                let images_to_dl = mani
                    .get_local_asset_digests()?
                    .into_iter()
                    .map(|digest| {
                        let mut image = remote_image.clone();
                        image.reference = digest.to_string();
                        image
                    })
                    .collect::<Vec<_>>();
                let futures = images_to_dl
                    .iter()
                    .map(|img| self.download_manifest_and_layers(registry, img, local_repo_name));
                try_join_all(futures).await?;
            }
            OCIManifest::V2(_) => {
                let digests: Vec<_> = mani.get_local_asset_digests()?;

                let futures = digests
                    .iter()
                    .map(|digest| self.download_blob(registry, digest));
                try_join_all(futures).await?;
            }
        }
        registry
            .storage
            .write_image_manifest(mani.raw(), local_repo_name, &remote_image.reference, false)
            .await?;

        Ok(())
    }

    pub async fn get_digest_from_remote(&self) -> Option<Digest> {
        let resp = self
            .authenticated_request(Method::HEAD, &self.remote_image.get_manifest_url())
            .headers(create_accept_header())
            .send()
            .await;
        match resp.as_ref().map(|r| r.headers().get(DIGEST_HEADER)) {
            Err(e) => {
                event!(
                    Level::ERROR,
                    "Remote registry didn't respond correctly to HEAD request {}",
                    e
                );
                None
            }
            Ok(None) => {
                event!(
                    Level::ERROR,
                    "Remote registry didn't send header {DIGEST_HEADER}",
                );
                None
            }
            Ok(Some(header)) => {
                let digest_str = header.to_str().unwrap();
                let digest = Digest::try_from_raw(digest_str).unwrap();
                Some(digest)
            }
        }
    }
}

/// Fetches AWS ECR credentials.
/// We use the [rusoto ChainProvider](https://docs.rs/rusoto_credential/0.48.0/rusoto_credential/struct.ChainProvider.html)
/// to fetch AWS credentials.
async fn get_aws_ecr_password_from_env(ecr_host: &str) -> Result<String> {
    let region = ecr_host
        .split('.')
        .nth(3)
        .ok_or_else(|| anyhow!("Could not parse region from ECR URL"))?
        .to_owned();
    let region = aws_types::region::Region::new(region);
    let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .region(region)
        .load()
        .await;
    let ecr_clt = aws_sdk_ecr::Client::new(&config);
    let token_response = ecr_clt.get_authorization_token().send().await?;
    let token = token_response
        .authorization_data
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .authorization_token
        .unwrap();

    // The token is base64(username:password). Here, username is "AWS".
    // To get the password, we trim "AWS:" from the decoded token.
    let mut auth_str = general_purpose::STANDARD.decode(token)?;
    auth_str.drain(0..4);

    String::from_utf8(auth_str).context("Could not convert ECR token to valid password")
}

/// Get the WWW-Authenticate header from a registry.
/// Ok(None) is returned if the registry does not require authentication.
async fn get_www_authenticate_header(
    cl: &reqwest::Client,
    image: &RemoteImage,
) -> Result<Option<String>> {
    let resp = cl
        .head(&image.get_manifest_url())
        .headers(create_accept_header())
        .send()
        .await
        .map_err(|e| {
            anyhow!(
                "Could not fetch www-authenticate header from {} (failed with: {:?})",
                &image.get_manifest_url(),
                e
            )
        })?;

    match resp.status() {
        StatusCode::UNAUTHORIZED => resp
            .headers()
            .get(AUTHN_HEADER)
            .ok_or_else(|| {
                anyhow!("Expected www-authenticate header to identify authentication server")
            })
            .and_then(|v| v.to_str().context("Failed to read auth header"))
            .map(|s| Some(s.to_string())),
        StatusCode::OK => Ok(None),
        _ => Err(anyhow!("Unexpected status code {}", resp.status())),
    }
}

fn get_bearer_param_map(www_authenticate_header: &str) -> HashMap<String, String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"(?P<key>[^=]+)="(?P<value>.*?)",? *"#).unwrap();
    }
    let base = www_authenticate_header
        .strip_prefix("Bearer ")
        .unwrap_or("");

    RE.captures_iter(base)
        .map(|m| {
            (
                m.name("key").unwrap().as_str().to_string(),
                m.name("value").unwrap().as_str().to_string(),
            )
        })
        .collect()
}

async fn get_bearer_auth_token(
    cl: &reqwest::Client,
    www_authenticate_header: &str,
    auth: &SingleRegistryProxyConfig,
) -> Result<String> {
    let mut bearer_param_map = get_bearer_param_map(www_authenticate_header);
    event!(Level::DEBUG, "bearer param map: {:?}", bearer_param_map);
    let realm = bearer_param_map
        .get("realm")
        .cloned()
        .ok_or_else(|| anyhow!("Expected realm key in authenticate header"))?;

    bearer_param_map.remove("realm");
    event!(Level::DEBUG, "Realm is {}", realm);
    let mut request = cl.get(realm.as_str()).query(&bearer_param_map);
    if let Some(u) = &auth.username {
        event!(Level::INFO, "Attempting proxy authentication with user {u}");
        request = request.basic_auth(u, auth.password.as_ref());
    }

    let resp = request.send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "Authentication {} failed (HTTP {})",
            realm,
            resp.status()
        ));
    }

    let resp_json = resp
        .json::<serde_json::Value>()
        .await
        .context("Failed to deserialize auth response")?;

    resp_json
        .get("access_token")
        .or_else(|| resp_json.get("token"))
        .and_then(|s| s.as_str())
        .map(|s| strip_dquotes(s).unwrap_or(s).to_string())
        .ok_or_else(|| anyhow!("Failed to find auth token in auth response"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::matchers::{header, header_exists, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    const AUTHZ_HEADER: &str = "Authorization";

    async fn get_basic_setup() -> (MockServer, SingleRegistryProxyConfig, RemoteImage) {
        let server = MockServer::start().await;

        let proxy_cfg = SingleRegistryProxyConfig {
            host: server.uri(),
            alias: "toto".to_string(),
            username: None,
            password: None,
            ignore_repos: vec![],
        };

        let proxy_image = RemoteImage::new(&proxy_cfg.host, "hello_world".into(), "latest".into());
        (server, proxy_cfg, proxy_image)
    }

    #[tokio::test]
    async fn test_no_auth() {
        let (server, proxy_cfg, proxy_image) = get_basic_setup().await;

        Mock::given(method("HEAD"))
            .and(path("/v2/hello_world/manifests/latest"))
            .and(header_exists("Accept"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        ProxyClient::try_new(proxy_cfg, &proxy_image).await.unwrap();
    }

    #[tokio::test]
    async fn test_basic_auth() {
        let (server, mut cfg, image) = get_basic_setup().await;

        Mock::given(method("HEAD"))
            .and(path("/v2/hello_world/manifests/latest"))
            .and(header_exists("Accept"))
            .respond_with(
                ResponseTemplate::new(401)
                    .insert_header(AUTHN_HEADER, "Basic realm=\"hell\", charset=\"UTF-8\""),
            )
            .expect(1)
            .mount(&server)
            .await;

        let username = "lucifer";
        cfg.username = Some(username.to_string());

        let clt = ProxyClient::try_new(cfg, &image).await.unwrap();

        assert!(matches!(clt.auth, HttpAuth::Basic(u, None) if u == username));
    }

    #[tokio::test]
    async fn test_bearer_auth() {
        let (server, cfg, image) = get_basic_setup().await;
        let response_auth_header = format!(
            "Bearer realm=\"{}/hell\", charset=\"UTF-8\",service=\"trow_registry\",scope=\"push/pull\"",
            server.uri()
        );

        Mock::given(method("HEAD"))
            .and(path("/v2/hello_world/manifests/latest"))
            .and(header_exists("Accept"))
            .respond_with(
                ResponseTemplate::new(401)
                    .insert_header(AUTHN_HEADER, response_auth_header.as_str()),
            )
            .expect(1)
            .mount(&server)
            .await;

        let token = "no-token-haha";
        Mock::given(method("GET"))
            .and(path("/hell"))
            .and(query_param("charset", "UTF-8"))
            .and(query_param("service", "trow_registry"))
            .and(query_param("scope", "push/pull"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": token,
            })))
            .expect(1)
            .mount(&server)
            .await;
        let cl = ProxyClient::try_new(cfg, &image).await.unwrap();

        assert!(matches!(cl.auth, HttpAuth::Bearer(tok) if tok == token));
    }

    #[tokio::test]
    async fn test_bearer_auth_with_username_password() {
        let (server, mut cfg, image) = get_basic_setup().await;

        cfg.username = Some("like-this".to_string());
        cfg.password = Some("reign-of-the-septims".to_string());

        let resp_authn_header = format!(
            "Bearer realm=\"{}/hive/impish\",oscillating=\"YES\", born=\"too-slow\",scope=\"repository:nvidia/cuda:pull,push\"",
            server.uri()
        );
        let mock_401 = Mock::given(method("HEAD"))
            .and(path("/v2/hello_world/manifests/latest"))
            .and(header_exists("Accept"))
            .respond_with(
                ResponseTemplate::new(401).insert_header(AUTHN_HEADER, resp_authn_header.as_str()),
            )
            .expect(1)
            .named("HEAD 401")
            .mount_as_scoped(&server)
            .await;

        let token = "alleycat-token";
        let expected_authz_header = format!(
            "Basic {}",
            general_purpose::STANDARD_NO_PAD.encode("like-this:reign-of-the-septims")
        );
        let mock_get_bearer = Mock::given(method("GET"))
            .and(path("/hive/impish"))
            .and(query_param("oscillating", "YES"))
            .and(query_param("born", "too-slow"))
            .and(query_param("scope", "repository:nvidia/cuda:pull,push"))
            .and(header(AUTHZ_HEADER, expected_authz_header.as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "token": token,
            })))
            .expect(1)
            .named("GET bearer")
            .mount_as_scoped(&server)
            .await;

        let cl = ProxyClient::try_new(cfg, &image).await;
        if let Err(e) = &cl {
            println!("{:#}", e);
        }

        assert!(matches!(cl.unwrap().auth, HttpAuth::Bearer(tok) if tok == token));
        drop(mock_401);
        drop(mock_get_bearer);
    }
}
