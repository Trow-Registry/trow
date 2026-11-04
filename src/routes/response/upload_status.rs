use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub use crate::services::uploads::UploadStatus;

impl IntoResponse for UploadStatus {
    fn into_response(self) -> Response {
        let location = format!("/v2/{}/blobs/uploads/{}", self.repo, self.uuid);
        // An empty upload has no accepted bytes yet, so the range is empty.
        let range = format!("0-{}", (self.offset as u64).saturating_sub(1));

        Response::builder()
            .header("Docker-Upload-UUID", self.uuid.to_string())
            .header("Range", range)
            .header("Content-Length", "0")
            .header("Location", location)
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap()
            .into_response()
    }
}

#[cfg(test)]
mod test {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use crate::routes::response::upload_status::UploadStatus;

    #[test]
    fn no_content_with_upload_details() {
        let uuid = uuid::Uuid::new_v4();
        let response = UploadStatus {
            uuid,
            repo: "moredhel/test".to_owned(),
            offset: 6,
        }
        .into_response();

        let headers = response.headers();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            headers.get("Docker-Upload-UUID").unwrap(),
            uuid.to_string().as_str()
        );
        assert_eq!(headers.get("Range").unwrap(), "0-5");
        assert_eq!(headers.get("Content-Length").unwrap(), "0");
        assert_eq!(
            headers.get("Location").unwrap(),
            format!("/v2/moredhel/test/blobs/uploads/{uuid}").as_str()
        );
    }

    #[test]
    fn empty_upload_saturates_range() {
        let response = UploadStatus {
            uuid: uuid::Uuid::new_v4(),
            repo: "moredhel/test".to_owned(),
            offset: 0,
        }
        .into_response();

        assert_eq!(response.headers().get("Range").unwrap(), "0-0");
    }
}
