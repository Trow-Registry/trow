use std::collections::HashMap;

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use log::info;
use quoted_string::strip_dquotes;
use regex::Regex;
use reqwest::StatusCode;
use reqwest::{self, Method};

use crate::server::create_accept_header;
use crate::server::Image;
use crate::RegistryProxyConfig;

const AUTHN_HEADER: &str = "www-authenticate";

pub enum HttpAuth {
    Basic(String, Option<String>),
    Bearer(String),
    None,
}

/// Wrapper around `reqwest::Client` that automagically handles authentication
/// to other container registries
pub struct ProxyClient {
    pub cl: reqwest::Client,
    pub auth: HttpAuth,
}

impl ProxyClient {
    pub async fn try_new(proxy_cfg: &RegistryProxyConfig, proxy_image: &Image) -> Result<Self> {
        let base_client = reqwest::Client::new();

        let www_authenticate_header =
            Self::get_www_authenticate_header(&base_client, proxy_image).await?;
        let cl = match www_authenticate_header.as_str() {
            h if h.starts_with("Basic") => {
                if proxy_cfg.username.is_none() {
                    return Err(anyhow!(
                        "Registry `{}` requires Basic auth but no username was provided",
                        proxy_cfg.host
                    ));
                }
                ProxyClient {
                    cl: base_client,
                    auth: HttpAuth::Basic(
                        proxy_cfg.username.clone().unwrap(),
                        proxy_cfg.password.clone(),
                    ),
                }
            }
            h if h.starts_with("Bearer") => {
                let tok = Self::get_bearer_auth_token(&base_client, h, proxy_cfg)
                    .await
                    .map_err(|e| {
                        anyhow!(
                            "Failed to get bearer auth token for {}. Error: {}",
                            proxy_image,
                            e
                        )
                    })?;

                ProxyClient {
                    cl: base_client,
                    auth: HttpAuth::Bearer(tok),
                }
            }
            "" => ProxyClient {
                cl: base_client,
                auth: HttpAuth::None,
            },
            _ => {
                return Err(anyhow!(
                    "Registry `{}` requires authentication but no supported scheme was provided in WWW-Authenticate",
                    proxy_cfg.host
                ));
            }
        };

        Ok(cl)
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

    async fn get_bearer_auth_token(
        cl: &reqwest::Client,
        www_authenticate_header: &str,
        auth: &RegistryProxyConfig,
    ) -> Result<String> {
        let mut bearer_param_map = Self::get_bearer_param_map(www_authenticate_header);
        info!("bearer param map: {:?}", bearer_param_map);
        let realm = bearer_param_map
            .get("realm")
            .cloned()
            .ok_or_else(|| anyhow!("Expected realm key in authenticate header"))?;

        bearer_param_map.remove("realm");
        info!("Realm is {}", realm);
        let mut request = cl.get(realm.as_str()).query(&bearer_param_map);

        if let Some(u) = &auth.username {
            info!("Attempting proxy authentication with user {}", u);
            request = request.basic_auth(u, auth.password.as_ref());
        }

        let resp = request
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send authenticate to {} request: {}", realm, e))?;
        info!("resp: {:?}", resp);
        if !resp.status().is_success() {
            return Err(anyhow!("Failed to authenticate to {}", realm));
        }

        let resp_json = resp
            .json::<serde_json::Value>()
            .await
            .map_err(|e| anyhow!("Failed to deserialize auth response {}", e))?;

        resp_json
            .get("access_token")
            .or_else(|| resp_json.get("token"))
            .and_then(|s| s.as_str())
            .map(|s| strip_dquotes(s).unwrap_or(s).to_string())
            .ok_or_else(|| anyhow!("Failed to find auth token in auth repsonse"))
    }

    async fn get_www_authenticate_header(cl: &reqwest::Client, image: &Image) -> Result<String> {
        let resp = cl
            .head(&image.get_manifest_url())
            .headers(create_accept_header())
            .send()
            .await
            .map_err(|e| {
                anyhow!(
                    "Attempt to authenticate to {} failed with: {}",
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
                .and_then(|v| {
                    v.to_str()
                        .map_err(|e| anyhow!("Failed to read auth header: {:?}", e))
                })
                .map(|s| s.to_string()),
            StatusCode::OK => Ok(String::new()),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    const AUTHZ_HEADER: &str = "Authorization";

    fn get_basic_setup() -> (MockServer, RegistryProxyConfig, Image) {
        let server = MockServer::start();

        let proxy_cfg = RegistryProxyConfig {
            host: format!("http://{}", server.address()),
            alias: "toto".to_string(),
            username: None,
            password: None,
        };

        let proxy_image = Image {
            host: format!("{}/v2", &proxy_cfg.host),
            repo: "hello_world".to_string(),
            tag: "latest".to_string(),
        };

        (server, proxy_cfg, proxy_image)
    }

    #[tokio::test]
    async fn test_no_auth() {
        let (server, proxy_cfg, proxy_image) = get_basic_setup();

        let mock_server = server.mock(|when, then| {
            when.method("HEAD")
                .path("/v2/hello_world/manifests/latest")
                .header_exists("Accept");
            then.status(200);
        });

        ProxyClient::try_new(&proxy_cfg, &proxy_image)
            .await
            .unwrap();
        mock_server.assert();
    }

    #[tokio::test]
    async fn test_basic_auth() {
        let (server, mut cfg, image) = get_basic_setup();

        let mock_server = server.mock(|when, then| {
            when.method("HEAD")
                .path("/v2/hello_world/manifests/latest")
                .header_exists("Accept");
            then.status(401)
                .header(AUTHN_HEADER, "Basic realm=\"hell\", charset=\"UTF-8\"");
        });
        let username = "lucifer";
        cfg.username = Some(username.to_string());

        let clt = ProxyClient::try_new(&cfg, &image).await.unwrap();

        mock_server.assert();
        assert!(matches!(clt.auth, HttpAuth::Basic(u, None) if u == username));
    }

    #[tokio::test]
    async fn test_bearer_auth() {
        let (server, cfg, image) = get_basic_setup();

        let mock_head_req = server.mock(|when, then| {
            when.method("HEAD")
                .path("/v2/hello_world/manifests/latest")
                .header_exists("Accept");
            then.status(401).header(
                AUTHN_HEADER,
                format!(
                    "Bearer realm=\"{}/hell\", charset=\"UTF-8\",service=\"trow_registry\",scope=\"push/pull\"",
                    server.base_url()
                ),
            );
        });
        let token = "no-token-haha";
        let mock_auth_tok = server.mock(|when, then| {
            when.method("GET")
                .path("/hell")
                .query_param("charset", "UTF-8")
                .query_param("service", "trow_registry")
                .query_param("scope", "push/pull");
            then.status(200).json_body(json!({
                "access_token": token,
            }));
        });

        let cl = ProxyClient::try_new(&cfg, &image).await.unwrap();

        mock_head_req.assert();
        mock_auth_tok.assert();
        assert!(matches!(cl.auth, HttpAuth::Bearer(tok) if tok == token));
    }

    #[tokio::test]
    async fn test_bearer_auth_with_username_password() {
        let (server, mut cfg, image) = get_basic_setup();

        cfg.username = Some("like-this".to_string());
        cfg.password = Some("reign-of-the-septims".to_string());

        let mock_head_req = server.mock(|when, then| {
            when.method("HEAD")
                .path("/v2/hello_world/manifests/latest")
                .header_exists("Accept");
            then.status(401).header(
                AUTHN_HEADER,
                format!(
                    "Bearer realm=\"{}/hive/impish\",oscillating=\"YES\", born=\"too-slow\",scope=\"repository:nvidia/cuda:pull,push\"",
                    server.base_url()
                ),
            );
        });
        let token = "alleycat-token";
        let mock_auth_tok = server.mock(|when, then| {
            when.method("GET")
                .path("/hive/impish")
                .query_param("oscillating", "YES")
                .query_param("born", "too-slow")
                .query_param("scope", "repository:nvidia/cuda:pull,push")
                .header(
                    AUTHZ_HEADER,
                    format!("Basic {}", base64::encode("like-this:reign-of-the-septims")),
                );
            then.status(200).json_body(json!({
                "token": token,
            }));
        });

        let cl = ProxyClient::try_new(&cfg, &image).await.unwrap();

        mock_head_req.assert();
        mock_auth_tok.assert();
        assert!(matches!(cl.auth, HttpAuth::Bearer(tok) if tok == token));
    }
}
