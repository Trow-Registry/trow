//! Routes-layer plumbing for registry authentication: header extraction and
//! response rendering. Credential checks and token signing live in
//! `services::auth_service`.

use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{RequestPartsExt, body};
use axum_extra::headers;
use base64::Engine as _;
use base64::engine::general_purpose as base64_engine;
use headers::HeaderMapExt;
use serde::{Deserialize, Serialize};

use super::authenticate::Authenticate;
use crate::TrowServerState;
use crate::routes::extracts::AlwaysHost;
use crate::services::auth_service::AuthenticatedUser;

const AUTHORIZATION: &str = "authorization";

/// A `Basic` authorization header that has been checked against the configured
/// user. Only produced by [`ValidBasicToken`].
pub struct ValidBasicToken {
    user: String,
}

impl<S> FromRequestParts<S> for ValidBasicToken
where
    Arc<TrowServerState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, ());

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_service = &Arc::from_ref(state).services.auth;

        if !auth_service.requires_auth() {
            tracing::warn!("Attempted login, but no users are configured");
            return Err((StatusCode::UNAUTHORIZED, ()));
        }

        // The value of the header is the type of the auth (Basic or Bearer),
        // followed by an encoded string, separated by whitespace.
        let auth_val = match req.headers.get(AUTHORIZATION) {
            Some(a) => a.to_str().map_err(|_| (StatusCode::UNAUTHORIZED, ()))?,
            None => return Err((StatusCode::UNAUTHORIZED, ())),
        };

        let auth_strings: Vec<String> = auth_val.split_whitespace().map(String::from).collect();
        if auth_strings.len() != 2 {
            //TODO: Should this be BadRequest?
            return Err((StatusCode::UNAUTHORIZED, ()));
        }
        // We're looking for a Basic token
        if auth_strings[0] != "Basic" {
            //TODO: This probably isn't right, maybe check if bearer?
            return Err((StatusCode::UNAUTHORIZED, ()));
        }

        tracing::debug!("Attempting to decode auth string {}", auth_strings[1]);

        let user_pass = base64_engine::STANDARD
            .decode(&auth_strings[1])
            .map_err(|_| (StatusCode::UNAUTHORIZED, ()))?;

        match auth_service.verify_credentials(&user_pass) {
            Ok(identity) => Ok(ValidBasicToken {
                user: identity.user,
            }),
            Err(e) => {
                tracing::debug!("Rejected basic credentials: {e}");
                Err((StatusCode::UNAUTHORIZED, ()))
            }
        }
    }
}

impl ValidBasicToken {
    pub fn user(&self) -> &str {
        &self.user
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TrowToken {
    pub user: String,
    pub token: String,
}

impl TrowToken {
    pub fn new(user: String, token: String) -> Self {
        Self { user, token }
    }
}

impl From<AuthenticatedUser> for TrowToken {
    fn from(identity: AuthenticatedUser) -> Self {
        Self {
            user: identity.user,
            token: identity.token,
        }
    }
}

/*
 * Responder returns token as JSON body
 */
impl IntoResponse for TrowToken {
    fn into_response(self) -> Response {
        //TODO: would be better to use serde here
        let formatted_body = format!("{{\"token\": \"{}\"}}", self.token);
        Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CONTENT_LENGTH, formatted_body.len())
            .status(StatusCode::OK)
            .body(body::Body::from(formatted_body))
            .unwrap()
            .into_response()
    }
}

impl<S> FromRequestParts<S> for TrowToken
where
    Arc<TrowServerState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Authenticate;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_service = &Arc::from_ref(state).services.auth;
        let base_url = match parts
            .extract_with_state::<Result<AlwaysHost, _>, _>(state)
            .await
            .unwrap()
        {
            Ok(AlwaysHost(host)) => host,
            Err(_) => String::new(),
        };

        if !auth_service.requires_auth() {
            //Authentication is not configured
            let no_auth_token =
                TrowToken::from(crate::services::auth_service::AuthService::anonymous());
            return Ok(no_auth_token);
        }
        let authorization = match parts
            .headers
            .typed_get::<headers::Authorization<headers::authorization::Bearer>>()
        {
            Some(bt) => bt,
            None => return Err(Authenticate::new(base_url)),
        };

        match auth_service.verify_token(authorization.token()) {
            Ok(identity) => Ok(TrowToken::from(identity)),
            Err(e) => {
                tracing::warn!("Failed to verify user token: {e}");
                Err(Authenticate::new(base_url))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test() {}
}
