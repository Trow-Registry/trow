use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::routing::get;
use oci_spec::image::ImageIndex;

use super::macros::endpoint_fn_7_levels;
use super::response::OciJson;
use crate::TrowServerState;
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;

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
    let index = state
        .services
        .referrers
        .list_referrers(repo, digest)
        .await?;
    let content_type = index.media_type().as_ref().unwrap().as_ref();
    Ok(OciJson::new(&index).set_content_type(content_type))
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

    use crate::test_utilities;
    use crate::utils::digest::Digest;

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn test_get_referrers() {
        let tmp_dir = test_temp_dir!();
        let (state, router) = test_utilities::trow_router(|_| {}, &tmp_dir).await;

        let dummy_blob_digest =
            "sha256:44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a";
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
                    oci_spec::image::Digest::from_str(dummy_blob_digest).unwrap(),
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
                    oci_spec::image::Digest::from_str(dummy_blob_digest).unwrap(),
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
            .execute(state.services.repos().db_rw())
            .await
            .unwrap();
            sqlx::query!(
                r#"INSERT INTO repo_blob_assoc (repo_name, manifest_digest) VALUES ("test", $1)"#,
                digest
            )
            .execute(state.services.repos().db_rw())
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
