// routes
mod admission;
mod blob;
mod blob_upload;
mod catalog;
mod health;
mod manifest;
mod readiness;

// helpers
mod extracts;
mod macros;
mod response;

use std::str;
use std::sync::Arc;
use std::time::Duration;

use axum::body::{Body, HttpBody};
use axum::extract::State;
use axum::http::method::Method;
use axum::http::{header, HeaderName, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use hyper::http::HeaderValue;
use response::errors::Error;
use response::html::HTML;
use response::trow_token::{self, TrowToken, ValidBasicToken};
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::{cors, trace};
use tracing::{event, Level};

use crate::TrowServerState;

fn add_router_layers<S: Send + Sync + Clone + 'static>(
    mut app: Router<S>,
    cors_domains: &Option<Vec<String>>,
) -> Router<S> {
    // configure logging
    app = app.layer(
        trace::TraceLayer::new_for_http()
            .make_span_with(|req: &axum::http::Request<Body>| {
                tracing::info_span!(
                    "request",
                    method = req.method().as_str(),
                    path = req.uri().path(),
                )
            })
            .on_response(
                |body: &Response<Body>, duration: Duration, _span: &tracing::Span| {
                    let size = body.size_hint();
                    let size_str = humansize::format_size(
                        size.upper().unwrap_or(size.lower()),
                        humansize::BINARY.space_after_value(false),
                    );
                    tracing::info!(
                        status = body.status().as_str(),
                        duration_ms = duration.as_millis(),
                        size = size_str,
                        "response sent"
                    );
                },
            )
            .on_eos(
                |_trailers: Option<&hyper::HeaderMap>,
                 stream_duration: Duration,
                 _span: &tracing::Span| {
                    tracing::info!(duration_ms = stream_duration.as_millis(), "end of stream");
                },
            ),
    );
    // Set API Version Header
    app = app.layer(SetResponseHeaderLayer::if_not_present(
        HeaderName::try_from("Docker-Distribution-API-Version").unwrap(),
        HeaderValue::from_static("registry/2.0"),
    ));
    // Ugly hack to return a json body on HEAD requests
    app = app.layer(
        ServiceBuilder::new()
            .map_response(|mut r: Response| {
                r.headers_mut().insert(
                    "Docker-Distribution-API-Version",
                    HeaderValue::from_static("registry/2.0"),
                );
                r
            })
            .map_response(|mut r: Response| {
                if r.status() == StatusCode::NOT_FOUND {
                    let body = r.body_mut();
                    if let Some(0) = body.size_hint().upper() {
                        let err = Error::NotFound.to_string();

                        *body = Body::from(err.clone());
                        r.headers_mut()
                            .insert(header::CONTENT_LENGTH, err.len().into());
                    }
                }
                r
            }),
    );

    if let Some(domains) = &cors_domains {
        app = app.layer(
            cors::CorsLayer::new()
                .allow_credentials(true)
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_origin(
                    domains
                        .iter()
                        .map(|d| d.parse::<HeaderValue>().unwrap())
                        .collect::<Vec<_>>(),
                ),
        );
    }
    app
}

pub fn create_app(state: Arc<super::TrowServerState>) -> Router {
    let mut app = Router::new();

    app = app
        .route("/v2/", get(get_v2root))
        .route("/", get(get_homepage))
        .route("/login", get(login))
        .route("/healthz", get(health::healthz))
        .route("/readiness", get(readiness::readiness));

    app = blob::route(app);
    app = blob_upload::route(app);
    app = catalog::route(app);
    app = manifest::route(app);
    app = admission::route(app);

    app = add_router_layers(app, &state.config.cors);
    app.with_state(state)
}

/*
 * v2 - throw Empty
 */
async fn get_v2root(_auth_user: TrowToken) -> &'static str {
    "{}"
}
/*
 * Welcome message
 */
async fn get_homepage<'a>() -> HTML<'a> {
    const ROOT_RESPONSE: &str = "<!DOCTYPE html><html><body>
<h1>Welcome to Trow, the cluster registry</h1>
</body></html>";

    HTML(ROOT_RESPONSE)
}

// // Want non HTML return for 404 for docker client
// #[catch(404)]
// fn not_found(_: &Request) -> Json<String> {
//     Json("404 page not found".to_string())
// }

// #[catch(401)]
// fn no_auth(_req: &Request) -> Authenticate {
//     Authenticate {}
// }

/* login should it be /v2/login?
 * this is where client will attempt to login
 *
 * If login is called with a valid bearer token, return session token
 */
async fn login(
    auth_user: ValidBasicToken,
    State(state): State<Arc<TrowServerState>>,
) -> Result<TrowToken, Error> {
    let tok = trow_token::new(auth_user, &state.config);
    match tok {
        Ok(t) => Ok(t),
        Err(e) => {
            event!(Level::ERROR, "Failed to create token: {:#}", e);
            Err(Error::InternalError)
        }
    }
}
