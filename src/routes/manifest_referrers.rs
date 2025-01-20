use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;
use digest::Digest;
use oci_spec::image::{Descriptor, ImageIndex, MediaType};
use sqlx::types::Json;

use super::macros::endpoint_fn_7_levels;
use super::response::OciJson;
use crate::registry::digest;
use crate::registry::manifest::OCIManifest;
use crate::registry::server::PROXY_DIR;
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::TrowServerState;

/*
---
Listing Referrers
GET /v2/<name>/referrers/<digest>

# Parameters
name - The namespace of the repository
digest - The digest of the manifest specified in the subject field.

# Query Parameters
(TODO) artifactType: The type of artifact to list referrers for.

 */
async fn get_referrers(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, digest)): Path<(String, String)>,
) -> Result<OciJson<ImageIndex>, Error> {
    if repo.starts_with(PROXY_DIR) {
        return Err(Error::UnsupportedForProxiedRepo);
    }
    let _ = Digest::try_from_raw(&digest)?;
    let referrers = sqlx::query!(
        r#"
        SELECT json(m.json) as "content!: Json<OCIManifest>",
            m.digest,
            length(m.blob) as "size!: i64"
        FROM manifest m
        INNER JOIN repo_blob_association rba ON rba.blob_digest = m.digest
        WHERE rba.repo_name = $1
            AND (m.json -> 'subject' ->> 'digest') = $2
        "#,
        repo,
        digest
    )
    .fetch_all(&mut *state.db.acquire().await?)
    .await?;

    let mut descriptors = vec![];
    for row in referrers {
        let parsed_manifest = row.content.0;

        let mediatype = parsed_manifest
            .media_type()
            .clone()
            .unwrap_or(MediaType::ImageConfig);

        let mut descriptor = Descriptor::new(
            mediatype,
            row.size as u64,
            oci_spec::image::Digest::from_str(&row.digest).unwrap(),
        );
        descriptor.set_artifact_type(parsed_manifest.artifact_type());
        descriptor.set_annotations(parsed_manifest.annotations().clone());
        descriptors.push(descriptor);
    }

    let mut response_manifest = ImageIndex::default();
    response_manifest.set_manifests(descriptors);
    response_manifest.set_media_type(Some(MediaType::ImageIndex));
    let content_type = response_manifest.media_type().as_ref().unwrap().as_ref();

    Ok(OciJson::new(&response_manifest).set_content_type(content_type))
}

endpoint_fn_7_levels!(
    get_referrers(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name, reference: String]
    ) -> Result<OciJson<ImageIndex>, Error>
);

pub fn route(mut app: Router<Arc<TrowServerState>>) -> Router<Arc<TrowServerState>> {
    #[rustfmt::skip]
    route_7_levels!(
        app,
        "/v2" "/referrers/{digest}",
        get(get_referrers, get_referrers_2level, get_referrers_3level, get_referrers_4level, get_referrers_5level, get_referrers_6level, get_referrers_7level)
    );
    app
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use axum::body::Body;
    use hyper::{Request, StatusCode};
    use oci_spec::image::{Descriptor, ImageIndex, ImageManifestBuilder, MediaType};
    use test_temp_dir::test_temp_dir;
    use tower::ServiceExt;

    use super::*;
    use crate::test_utilities;

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn test_get_referrers() {
        let tmp_dir = test_temp_dir!();
        let (state, router) = test_utilities::trow_router(|_| {}, &tmp_dir).await;

        let man_referred = serde_json::to_vec(&ImageIndex::default()).unwrap();
        let man_referred_digest = Digest::digest_sha256_slice(&man_referred);
        let subj = serde_json::to_vec(
            &ImageManifestBuilder::default()
                .schema_version(2u32)
                .layers([])
                .media_type(MediaType::ImageManifest)
                .config(Descriptor::new(
                    MediaType::EmptyJSON,
                    2,
                    oci_spec::image::Digest::from_str(
                        "sha256:44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
                    )
                    .unwrap(),
                ))
                .subject(Descriptor::new(
                    MediaType::ImageIndex,
                    2,
                    oci_spec::image::Digest::from_str(man_referred_digest.as_str()).unwrap(),
                ))
                .build()
                .unwrap(),
        )
        .unwrap();
        let subj_digest = Digest::digest_sha256_slice(&subj);
        let nosubj = serde_json::to_vec(
            &ImageManifestBuilder::default()
                .schema_version(2u32)
                .layers([])
                .media_type(MediaType::ImageManifest)
                .config(Descriptor::new(
                    MediaType::EmptyJSON,
                    2,
                    oci_spec::image::Digest::from_str(
                        "sha256:1111111355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
                    )
                    .unwrap(),
                ))
                .build()
                .unwrap(),
        )
        .unwrap();

        for man in [man_referred, subj, nosubj] {
            let digest = Digest::digest_sha256_slice(&man).to_string();
            sqlx::query!(
                "INSERT INTO manifest (digest, json, blob) VALUES ($1, jsonb($2), $2)",
                digest,
                man,
            )
            .execute(&mut *state.db.acquire().await.unwrap())
            .await
            .unwrap();
            sqlx::query!(
                r#"INSERT INTO repo_blob_association (repo_name, blob_digest) VALUES ("test", $1)"#,
                digest
            )
            .execute(&mut *state.db.acquire().await.unwrap())
            .await
            .unwrap();
        }

        let resp = router
            .oneshot(
                Request::get(format!("/v2/test/referrers/{man_referred_digest}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "unexpected status code: {resp:?}"
        );
        let val: ImageIndex = test_utilities::response_body_json(resp).await;
        let descriptors = val.manifests();
        assert_eq!(descriptors.len(), 1);
        assert_eq!(
            descriptors[0].media_type().clone(),
            MediaType::ImageManifest
        );
        assert_eq!(descriptors[0].digest().as_ref(), subj_digest.as_str());
    }
}
