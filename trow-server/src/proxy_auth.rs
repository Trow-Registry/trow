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
                .get("www-authenticate")
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
            static ref RE: Regex = Regex::new(r#"(?P<key>[^=]+)="(?P<value>.*?)",?"#).unwrap();
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
