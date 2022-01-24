use crate::response::authenticate::Authenticate;
use crate::response::errors::Error;
use crate::response::html::HTML;
use crate::response::trow_token::ValidBasicToken;
use crate::response::trow_token::{self, TrowToken};
use crate::TrowConfig;
use rocket::request::Request;
use rocket::serde::json::{json, Json, Value};
use rocket::State;
use rocket::{catch, catchers, get, routes};
use std::str;

mod blob;
mod catalog;
mod health;
mod manifest;
mod metrics;
mod readiness;
mod validation;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_v2root,
        get_homepage,
        login,
        manifest::get_manifest,
        manifest::get_manifest_2level,
        manifest::get_manifest_3level,
        manifest::get_manifest_4level,
        manifest::get_manifest_5level,
        manifest::put_image_manifest,
        manifest::put_image_manifest_2level,
        manifest::put_image_manifest_3level,
        manifest::put_image_manifest_4level,
        manifest::put_image_manifest_5level,
        manifest::delete_image_manifest,
        manifest::delete_image_manifest_2level,
        manifest::delete_image_manifest_3level,
        manifest::delete_image_manifest_4level,
        manifest::delete_image_manifest_5level,
        blob::get_blob,
        blob::get_blob_2level,
        blob::get_blob_3level,
        blob::get_blob_4level,
        blob::get_blob_5level,
        blob::put_blob,
        blob::put_blob_2level,
        blob::put_blob_3level,
        blob::put_blob_4level,
        blob::put_blob_5level,
        blob::patch_blob,
        blob::patch_blob_2level,
        blob::patch_blob_3level,
        blob::patch_blob_4level,
        blob::patch_blob_5level,
        blob::post_blob_upload,
        blob::post_blob_upload_2level,
        blob::post_blob_upload_3level,
        blob::post_blob_upload_4level,
        blob::post_blob_upload_5level,
        blob::post_blob_upload_6level,
        blob::delete_blob,
        blob::delete_blob_2level,
        blob::delete_blob_3level,
        blob::delete_blob_4level,
        blob::delete_blob_5level,
        catalog::list_tags,
        catalog::list_tags_2level,
        catalog::list_tags_3level,
        catalog::list_tags_4level,
        catalog::list_tags_5level,
        catalog::get_catalog,
        catalog::get_manifest_history,
        catalog::get_manifest_history_2level,
        catalog::get_manifest_history_3level,
        catalog::get_manifest_history_4level,
        catalog::get_manifest_history_5level,
        validation::validate_image,
        health::healthz,
        readiness::readiness,
        metrics::metrics
    ]
}

pub fn catchers() -> Vec<rocket::Catcher> {
    catchers![not_found, no_auth]
}

/*
 * v2 - throw Empty
 */
#[get("/v2")]
fn get_v2root(_auth_user: TrowToken) -> Json<Value> {
    Json(json!({}))
}
/*
 * Welcome message
 */
#[get("/")]
fn get_homepage<'a>() -> HTML<'a> {
    const ROOT_RESPONSE: &str = "<!DOCTYPE html><html><body>
<h1>Welcome to Trow, the cluster registry</h1>
</body></html>";

    HTML(ROOT_RESPONSE)
}

// Want non HTML return for 404 for docker client
#[catch(404)]
fn not_found(_: &Request) -> Json<String> {
    Json("404 page not found".to_string())
}

#[catch(401)]
fn no_auth(_req: &Request) -> Authenticate {
    Authenticate {}
}

/* login should it be /v2/login?
 * this is where client will attempt to login
 *
 * If login is called with a valid bearer token, return session token
 */
#[get("/login")]
fn login(auth_user: ValidBasicToken, tc: &State<TrowConfig>) -> Result<TrowToken, Error> {
    trow_token::new(auth_user, tc).map_err(|_| Error::InternalError)
}
