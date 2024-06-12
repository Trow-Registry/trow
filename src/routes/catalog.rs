use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::Router;
use oci_spec::distribution::{RepositoryList, RepositoryListBuilder, TagList, TagListBuilder};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde_derive::Deserialize;

use super::macros::endpoint_fn_7_levels;
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::routes::response::OciJson;
use crate::{entity, TrowServerState};

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
    let mut select = entity::repo::Entity::find().order_by_asc(entity::repo::Column::Name);
    if let Some(last) = query.last {
        select = select.filter(entity::repo::Column::Name.gt(last));
    }
    let repos = select.limit(query.n).all(&state.db).await?;
    let raw_repos = repos.into_iter().map(|r| r.name).collect::<Vec<_>>();

    Ok(OciJson::new(
        &RepositoryListBuilder::default()
            .repositories(raw_repos)
            .build()
            .unwrap(),
    ))
}

async fn list_tags(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path(repo_name): Path<String>,
    Query(query): Query<CatalogListQuery>,
) -> Result<OciJson<TagList>, Error> {
    let mut select = entity::tag::Entity::find()
        .column(entity::tag::Column::Tag)
        .filter(entity::tag::Column::Repo.eq(&repo_name))
        .order_by_asc(entity::tag::Column::Tag);
    if let Some(last) = query.last {
        select = select.filter(entity::tag::Column::Tag.gt(last));
    }
    select = select.limit(query.n);
    let tags = select.all(&state.db).await?;
    let raw_tags = tags.into_iter().map(|t| t.tag).collect::<Vec<_>>();

    Ok(OciJson::new(
        &TagListBuilder::default()
            .name(repo_name)
            .tags(raw_tags)
            .build()
            .unwrap(),
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
