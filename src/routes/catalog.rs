use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::Router;
use serde_derive::Deserialize;

use super::macros::endpoint_fn_7_levels;
use crate::registry::ManifestHistory;
use crate::response::errors::Error;
use crate::response::trow_token::TrowToken;
use crate::routes::macros::route_7_levels;
use crate::types::{RepoCatalog, TagList};
use crate::TrowServerState;

#[derive(Debug, Deserialize)]
pub struct CatalogListQuery {
    n: Option<u32>,
    last: Option<String>,
}

async fn get_catalog(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Query(query): Query<CatalogListQuery>,
) -> Result<RepoCatalog, Error> {
    let limit = query.n.unwrap_or(std::u32::MAX);
    let last_repo = query.last.clone().unwrap_or_default();

    let cat = state
        .registry
        .get_catalog(Some(&last_repo), Some(limit))
        .await
        .map_err(|_| Error::InternalError)?;

    Ok(RepoCatalog::from(cat))
}

async fn list_tags(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path(repo_name): Path<String>,
    Query(query): Query<CatalogListQuery>,
) -> Result<TagList, Error> {
    let limit = query.n.unwrap_or(std::u32::MAX);
    let last_tag = query.last.clone().unwrap_or_default();

    let tags = state
        .registry
        .get_tags(&repo_name, Some(&last_tag), Some(limit))
        .await
        .map_err(|_| Error::InternalError)?;
    Ok(TagList::new_filled(repo_name, tags))
}
endpoint_fn_7_levels!(
    list_tags(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name],
        query: Query<CatalogListQuery>
    ) -> Result<TagList, Error>
);

async fn get_manifest_history(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((name, reference)): Path<(String, String)>,
    Query(query): Query<CatalogListQuery>,
) -> Result<ManifestHistory, Error> {
    let limit = query.n.unwrap_or(std::u32::MAX);
    let last_digest = query.last.clone().unwrap_or_default();

    let mh = state
        .registry
        .get_history(&name, &reference, Some(&last_digest), Some(limit))
        .await
        .map_err(|_| Error::InternalError)?;

    Ok(ManifestHistory::new(format!("{name}:{reference}"), mh))
}

endpoint_fn_7_levels!(
    get_manifest_history(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name, reference],
        query: Query<CatalogListQuery>
    ) -> Result<ManifestHistory, Error>
);

pub fn route(mut app: Router<Arc<TrowServerState>>) -> Router<Arc<TrowServerState>> {
    app = app.route("/v2/_catalog", get(get_catalog));
    #[rustfmt::skip]
    route_7_levels!(
        app,
        "/v2" "/tags/list",
        get(list_tags, list_tags_2level, list_tags_3level, list_tags_4level, list_tags_5level, list_tags_6level, list_tags_7level)
    );
    #[rustfmt::skip]
    route_7_levels!(
        app,
        "" "/manifest_history/:reference",
        get(get_manifest_history, get_manifest_history_2level, get_manifest_history_3level, get_manifest_history_4level, get_manifest_history_5level, get_manifest_history_6level, get_manifest_history_7level)
    );
    app
}
