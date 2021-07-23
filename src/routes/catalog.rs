use crate::client_interface::ClientInterface;
use crate::registry_interface::{CatalogOperations, ManifestHistory};
use crate::response::errors::Error;
use crate::response::trow_token::TrowToken;
use crate::types::{RepoCatalog, TagList};

#[get("/v2/_catalog?<n>&<last>")]
pub fn get_catalog(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    n: Option<u32>,
    last: Option<String>,
) -> Result<RepoCatalog, Error> {
    let limit = n.unwrap_or(std::u32::MAX);
    let last_repo = last.unwrap_or_default();

    let cat = ci
        .get_catalog(Some(&last_repo), Some(limit))
        .map_err(|_| Error::InternalError)?;

    Ok(RepoCatalog::from(cat))
}

#[get("/v2/<repo_name>/tags/list?<last>&<n>")]
pub fn list_tags(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    repo_name: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<TagList, Error> {
    let limit = n.unwrap_or(std::u32::MAX);
    let last_tag = last.unwrap_or_default();

    let tags = ci
        .get_tags(&repo_name, Some(&last_tag), Some(limit))
        .map_err(|_| Error::InternalError)?;
    Ok(TagList::new_filled(repo_name, tags))
}

#[get("/v2/<user>/<repo>/tags/list?<last>&<n>")]
pub fn list_tags_2level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    user: String,
    repo: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<TagList, Error> {
    list_tags(auth_user, ci, format!("{}/{}", user, repo), last, n)
}

#[get("/v2/<org>/<user>/<repo>/tags/list?<last>&<n>")]
pub fn list_tags_3level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<TagList, Error> {
    list_tags(auth_user, ci, format!("{}/{}/{}", org, user, repo), last, n)
}

#[get("/v2/<fourth>/<org>/<user>/<repo>/tags/list?<last>&<n>")]
pub fn list_tags_4level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<TagList, Error> {
    list_tags(
        auth_user,
        ci,
        format!("{}/{}/{}/{}", fourth, org, user, repo),
        last,
        n,
    )
}

// TODO add support for pagination
#[get("/<onename>/manifest_history/<reference>?<last>&<n>")]
pub fn get_manifest_history(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    onename: String,
    reference: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<ManifestHistory, Error> {
    let limit = n.unwrap_or(std::u32::MAX);
    let last_digest = last.unwrap_or_default();

    let mh = ci
        .get_history(&onename, &reference, Some(&last_digest), Some(limit))
        .map_err(|_| Error::InternalError)?;
    Ok(mh)
}

#[get("/<user>/<repo>/manifest_history/<reference>?<last>&<n>")]
pub fn get_manifest_history_2level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    user: String,
    repo: String,
    reference: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<ManifestHistory, Error> {
    get_manifest_history(
        auth_user,
        ci,
        format!("{}/{}", user, repo),
        reference,
        last,
        n,
    )
}

#[get("/<org>/<user>/<repo>/manifest_history/<reference>?<last>&<n>")]
pub fn get_manifest_history_3level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    reference: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<ManifestHistory, Error> {
    get_manifest_history(
        auth_user,
        ci,
        format!("{}/{}/{}", org, user, repo),
        reference,
        last,
        n,
    )
}

#[get("/<fourth>/<org>/<user>/<repo>/manifest_history/<reference>?<last>&<n>")]
pub fn get_manifest_history_4level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    reference: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<ManifestHistory, Error> {
    get_manifest_history(
        auth_user,
        ci,
        format!("{}/{}/{}/{}", fourth, org, user, repo),
        reference,
        last,
        n,
    )
}
