use std::str;
use std::sync::Arc;
use std::time::Duration;

use axum::body::{boxed, Body};
use axum::extract::State;
use axum::http::method::Method;
use axum::http::{header, StatusCode};
use axum::response::Response;
use axum::routing::{get, post, put};
use axum::Router;
use hyper::body::HttpBody;
use hyper::http::HeaderValue;
use tower::ServiceBuilder;
use tower_http::{cors, trace};

use crate::response::errors::Error;
use crate::response::html::HTML;
use crate::response::trow_token::{self, TrowToken, ValidBasicToken};
use crate::TrowServerState;

mod admission;
mod blob;
mod catalog;
mod health;
mod manifest;
mod metrics;
mod readiness;

macro_rules! route_5_levels {
    ($app:ident, $prefix:literal $route:literal, $($method:ident($handler1:expr, $handler2:expr, $handler3:expr, $handler4:expr, $handler5:expr)),*) => {
        $app = $app
            .route(
                concat!($prefix, "/:one", $route),
                $($method($handler1)).*
            )
            .route(
                concat!($prefix, "/:one/:two", $route),
                $($method($handler2)).*
            )
            .route(
                concat!($prefix, "/:one/:two/:three", $route),
                $($method($handler3)).*,
            )
            .route(
                concat!($prefix, "/:one/:two/:three/:four", $route),
                $($method($handler4)).*,
            )
            .route(
                concat!($prefix, "/:one/:two/:three/:four/:five", $route),
                $($method($handler5)).*,
            )
            ;
    };
}

pub fn create_app(state: super::TrowServerState) -> Router {
    let mut app = Router::new()
        .route("/v2/", get(get_v2root))
        .route("/", get(get_homepage))
        .route("/login", get(login))
        .route("/validate-image", post(admission::validate_image))
        .route("/mutate-image", post(admission::mutate_image))
        .route("/healthz", get(health::healthz))
        .route("/metrics", get(metrics::metrics))
        .route("/readiness", get(readiness::readiness));

    // blob
    #[rustfmt::skip]
    route_5_levels!(
        app,
        "/v2" "/blobs/:digest",
        get(blob::get_blob, blob::get_blob_2level, blob::get_blob_3level, blob::get_blob_4level, blob::get_blob_5level),
        delete(blob::delete_blob, blob::delete_blob_2level, blob::delete_blob_3level, blob::delete_blob_4level, blob::delete_blob_5level)
    );
    #[rustfmt::skip]
    route_5_levels!(
        app,
        "/v2" "/blobs/uploads/",
        post(blob::post_blob_upload, blob::post_blob_upload_2level, blob::post_blob_upload_3level, blob::post_blob_upload_4level, blob::post_blob_upload_5level)
    );
    #[rustfmt::skip]
    route_5_levels!(
        app,
        "/v2" "/blobs/uploads/:uuid",
        put(blob::put_blob, blob::put_blob_2level, blob::put_blob_3level, blob::put_blob_4level, blob::put_blob_5level),
        patch(blob::patch_blob, blob::patch_blob_2level, blob::patch_blob_3level, blob::patch_blob_4level, blob::patch_blob_5level)
    );

    // catalog
    app = app.route("/v2/_catalog", get(catalog::get_catalog));
    #[rustfmt::skip]
    route_5_levels!(
        app,
        "/v2" "/tags/list",
        get(catalog::list_tags, catalog::list_tags_2level, catalog::list_tags_3level, catalog::list_tags_4level, catalog::list_tags_5level)
    );
    #[rustfmt::skip]
    route_5_levels!(
        app,
        "" "/manifest_history/:reference",
        get(catalog::get_manifest_history, catalog::get_manifest_history_2level, catalog::get_manifest_history_3level, catalog::get_manifest_history_4level, catalog::get_manifest_history_5level)
    );

    // manifest
    #[rustfmt::skip]
    route_5_levels!(
        app,
        "/v2" "/manifests/:reference",
        get(manifest::get_manifest, manifest::get_manifest_2level, manifest::get_manifest_3level, manifest::get_manifest_4level, manifest::get_manifest_5level),
        put(manifest::put_image_manifest, manifest::put_image_manifest_2level, manifest::put_image_manifest_3level, manifest::put_image_manifest_4level, manifest::put_image_manifest_5level),
        delete(manifest::delete_image_manifest, manifest::delete_image_manifest_2level, manifest::delete_image_manifest_3level, manifest::delete_image_manifest_4level, manifest::delete_image_manifest_5level)
    );

    app = app.layer(
        trace::TraceLayer::new_for_http()
            .make_span_with(|req: &axum::http::Request<Body>| {
                tracing::info_span!(
                    "request",
                    method = req.method().as_str(),
                    path = req.uri().path(),
                )
            })
            .on_response(|_: &_, duration: Duration, _span: &tracing::Span| {
                tracing::info!("done in {:?}", duration)
            }),
    );

    if let Some(domains) = &state.config.cors {
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

    app.with_state(Arc::new(state)).layer(
        // Set API Version Header
        ServiceBuilder::new().map_response(|mut r: Response| {
            r.headers_mut().insert(
                "Docker-Distribution-API-Version",
                HeaderValue::from_static("registry/2.0"),
            );
            // ugly hack to work around the fact that axum returns not body for HEAD
            if r.status() == StatusCode::NOT_FOUND {
                let body = r.body_mut();
                if let Some(0) = body.size_hint().upper() {
                    let err = Error::NotFound.to_string();

                    *body = boxed(Body::from(err.clone()));
                    r.headers_mut()
                        .insert(header::CONTENT_LENGTH, err.len().into());
                }
            }

            r
        }),
    )
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
    trow_token::new(auth_user, &state.config).map_err(|_| Error::InternalError)
}
