use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use oci_spec::distribution::{RepositoryList, TagList};
use serde_derive::Deserialize;

use super::macros::endpoint_fn_7_levels;
use crate::TrowServerState;
use crate::routes::macros::route_7_levels;
use crate::routes::response::OciJson;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;

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
    let result = state
        .services
        .catalog
        .list_repositories(query.last.as_deref(), query.n)
        .await?;
    Ok(OciJson::new(&result))
}

async fn list_tags(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path(repo_name): Path<String>,
    Query(query): Query<CatalogListQuery>,
) -> Result<OciJson<TagList>, Error> {
    let result = state
        .services
        .catalog
        .list_tags(&repo_name, query.last.as_deref(), query.n)
        .await?;
    Ok(OciJson::new(&result))
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
