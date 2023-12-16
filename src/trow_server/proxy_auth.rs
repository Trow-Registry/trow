use std::collections::HashMap;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use aws_config::BehaviorVersion;
use base64::engine::general_purpose;
use base64::Engine as _;
use lazy_static::lazy_static;
use quoted_string::strip_dquotes;
use regex::Regex;
use reqwest::{self, Method, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{event, Level};

use super::image::RemoteImage;
use super::server::create_accept_header;

const AUTHN_HEADER: &str = "www-authenticate";

#[derive(Debug)]
pub enum HttpAuth {
    Basic(String, Option<String>),
    Bearer(String),
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegistryProxiesConfig {
    pub registries: Vec<SingleRegistryProxyConfig>,
    #[serde(default)]
    pub offline: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SingleRegistryProxyConfig {
    pub alias: String,
    /// This field is unvalidated and may contain a scheme or not.
    /// eg: `http://example.com` and `example.com`
    pub host: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// Wrapper around `reqwest::Client` that automagically handles authentication
/// to other container registries
pub struct ProxyClient {
    pub cl: reqwest::Client,
    pub auth: HttpAuth,
}

impl ProxyClient {
    pub async fn try_new(
        mut proxy_cfg: SingleRegistryProxyConfig,
        proxy_image: &RemoteImage,
    ) -> Result<Self> {
        let base_client = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_millis(1000))
            .build()?;

        let authn_header = get_www_authenticate_header(&base_client, proxy_image).await?;

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

        match authn_header {
            Some(h) if h.starts_with("Basic") => {
                Self::try_new_with_basic_auth(&proxy_cfg, base_client).await
            }
            Some(h) if h.starts_with("Bearer") => {
                Self::try_new_with_bearer_auth(&proxy_cfg, base_client, &h).await
            }
            None => Ok(ProxyClient {
                cl: base_client,
                auth: HttpAuth::None,
            }),
            Some(invalid_header) => Err(anyhow!(
                "Could not parse {AUTHN_HEADER} of registry `{}`: `{}`",
                proxy_cfg.host,
                invalid_header
            )),
        }
    }

    async fn try_new_with_basic_auth(
        proxy_cfg: &SingleRegistryProxyConfig,
        cl: reqwest::Client,
    ) -> Result<Self> {
        if proxy_cfg.username.is_none() {
            return Err(anyhow!(
                "Registry `{}` requires Basic auth but no username was provided",
                proxy_cfg.host
            ));
        }
        Ok(ProxyClient {
            cl,
            auth: HttpAuth::Basic(
                proxy_cfg.username.clone().unwrap(),
                proxy_cfg.password.clone(),
            ),
        })
    }

    async fn try_new_with_bearer_auth(
        proxy_cfg: &SingleRegistryProxyConfig,
        cl: reqwest::Client,
        authn_header: &str,
    ) -> Result<Self> {
        let tok = get_bearer_auth_token(&cl, authn_header, proxy_cfg)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to get bearer auth token for {}: {}",
                    proxy_cfg.host,
                    e
                )
            })?;

        Ok(ProxyClient {
            cl,
            auth: HttpAuth::Bearer(tok),
        })
    }

    /// Build a request with added authentication.
    /// The auth method will vary depending on the registry being queried.
    pub fn authenticated_request(&self, method: Method, url: &str) -> reqwest::RequestBuilder {
        let req = self.cl.request(method, url);
        match &self.auth {
            HttpAuth::Basic(username, password) => req.basic_auth(username, password.to_owned()),
            HttpAuth::Bearer(token) => req.bearer_auth(token),
            HttpAuth::None => req,
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
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
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
                "Could not fetch www-authenticate header from {} (failed with: {})",
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
