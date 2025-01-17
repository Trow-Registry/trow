use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::Router;
use oci_spec::distribution::{RepositoryList, RepositoryListBuilder, TagList, TagListBuilder};
use serde_derive::Deserialize;

use super::macros::endpoint_fn_7_levels;
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::routes::response::OciJson;
use crate::TrowServerState;

#[derive(Debug, Deserialize)]
pub struct CatalogListQuery {
    n: Option<u64>,
    last: Option<String>,
}

async fn get_catalog(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Query(query): Query<CatalogListQuery>,
) -> Result<OciJson<RepositoryList>, Error> {
    let mut conn = state.db.acquire().await?;
    let last_name = match &query.last {
        Some(l) => l,
        None => "",
    };
    let limit = query.n.unwrap_or(i64::MAX as u64) as i64;
    let repos = sqlx::query!(
        r#"
        SELECT DISTINCT rba.repo_name
        FROM repo_blob_association rba
        WHERE rba.repo_name > $1
        ORDER BY rba.repo_name ASC
        LIMIT $2
        "#,
        last_name,
        limit
    )
    .fetch_all(&mut *conn)
    .await?;
    let raw_repos = repos.into_iter().map(|r| r.repo_name).collect::<Vec<_>>();

    Ok(OciJson::new(
        &RepositoryListBuilder::default()
            .repositories(raw_repos)
            .build()
            .unwrap(),
        true,
    ))
}

async fn list_tags(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path(repo_name): Path<String>,
    Query(query): Query<CatalogListQuery>,
) -> Result<OciJson<TagList>, Error> {
    let mut conn = state.db.acquire().await?;
    let last_tag = match &query.last {
        Some(l) => l,
        None => "",
    };
    let limit = query.n.unwrap_or(i64::MAX as u64) as i64;
    let tags = sqlx::query!(
        r#"
        SELECT t.tag
        FROM tag t
        WHERE t.repo = $1
            AND t.tag > $2
        ORDER BY t.tag COLLATE NOCASE ASC
        LIMIT $3
        "#,
        repo_name,
        last_tag,
        limit
    )
    .fetch_all(&mut *conn)
    .await?;
    let raw_tags = tags.into_iter().map(|t| t.tag).collect::<Vec<_>>();

    Ok(OciJson::new(
        &TagListBuilder::default()
            .name(repo_name)
            .tags(raw_tags)
            .build()
            .unwrap(),
        true,
    ))
}
endpoint_fn_7_levels!(
    list_tags(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name],
        query: Query<CatalogListQuery>
    ) -> Result<OciJson<TagList>, Error>
);

pub fn route(mut app: Router<Arc<TrowServerState>>) -> Router<Arc<TrowServerState>> {
    app = app.route("/v2/_catalog", get(get_catalog));
    #[rustfmt::skip]
    route_7_levels!(
        app,
        "/v2" "/tags/list",
        get(list_tags, list_tags_2level, list_tags_3level, list_tags_4level, list_tags_5level, list_tags_6level, list_tags_7level)
    );
    app
}
