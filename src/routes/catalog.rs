use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path, Query, State};
use serde_derive::Deserialize;

use super::macros::endpoint_fn_7_levels;
use crate::registry_interface::ManifestHistory;
use crate::response::errors::Error;
use crate::response::trow_token::TrowToken;
use crate::types::{RepoCatalog, TagList};
use crate::TrowServerState;

#[derive(Debug, Deserialize)]
pub struct CatalogListQuery {
    n: Option<u32>,
    last: Option<String>,
}

pub async fn get_catalog(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Query(query): Query<CatalogListQuery>,
) -> Result<RepoCatalog, Error> {
    let limit = query.n.unwrap_or(std::u32::MAX);
    let last_repo = query.last.clone().unwrap_or_default();

    let cat = state
        .client
        .get_catalog(Some(&last_repo), Some(limit))
        .await
        .map_err(|_| Error::InternalError)?;

    Ok(RepoCatalog::from(cat))
}

pub async fn list_tags(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path(repo_name): Path<String>,
    Query(query): Query<CatalogListQuery>,
) -> Result<TagList, Error> {
    let limit = query.n.unwrap_or(std::u32::MAX);
    let last_tag = query.last.clone().unwrap_or_default();

    let tags = state
        .client
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

pub async fn get_manifest_history(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((name, reference)): Path<(String, String)>,
    Query(query): Query<CatalogListQuery>,
) -> Result<ManifestHistory, Error> {
    let limit = query.n.unwrap_or(std::u32::MAX);
    let last_digest = query.last.clone().unwrap_or_default();

    let mh = state
        .client
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
