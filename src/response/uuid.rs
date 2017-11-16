use failure::Error;
use rocket::State;
use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::request::Request;
use hostname;
use uuid::Uuid;

use config;
use errors;
use util;
use http_capnp::lycaon;

#[derive(Debug, Serialize)]
pub enum UuidResponse {
    Uuid {
        uuid: String,
        name: String,
        repo: String,
        left: u32,
        right: u32,
    },
    Empty,
}

impl UuidResponse {
    pub fn handle(
        config: State<config::Config>,
        name: String,
        repo: String,
    ) -> Result<UuidResponse, Error> {
        let uuid = gen_uuid().to_string();

        let mut handler = util::CapnpInterface::uuid_interface(&config)?;
        let mut msg = handler
            .builder
            .init_root::<lycaon::uuid_interface::uuid::Builder>();
        let proxy = handler.proxy.and_then(|proxy| {
            // TODO: this is a current hack to get around dynamic dispatch issues
            // with the proxy handler. This is _super_ fragile!
            if let util::CapnpInterface::Uuid(client) = proxy {
                Ok(client)
            } else {
                Err(errors::Server::CapnpInterfaceError("Uuid").into())
            }
        })?;
        let mut req = proxy.add_uuid_request();
        msg.set_uuid(&uuid);
        let response = req.get()
            .set_uuid(msg.as_reader())
            .map_err(|e| Error::from(e))
            .and(handler.core.and_then(|mut core| {
                core.run(req.send().promise).map_err(|e| Error::from(e))
            }))?;
        let result = response.get().map(|response| response.get_result())?;
        if result {
            Ok(UuidResponse::Uuid {
                uuid,
                name,
                repo,
                left: 0,
                right: 0,
            })
        } else {
            Err(errors::Server::CapnpInterfaceError("Uuid Response").into())
        }
    }
}

fn gen_uuid() -> Uuid {
    Uuid::new_v4()
}


/// Gets the base URL e.g. http://registry:8000 using the HOST value from the request header.
/// Falls back to hostname if it doesn't exist.
///
fn get_base_url(req: &Request) -> String {
    let host = match req.headers().get("HOST").next() {
        None => {
            hostname::get_hostname()
                .expect("I have no name")
                .to_string()
        }
        Some(shost) => shost.to_string(),
    };

    format!("http://{}", host)
}

impl<'r> Responder<'r> for UuidResponse {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        debug!("Uuid Ok");

        if let UuidResponse::Uuid {
            ref uuid,
            ref name,
            ref repo,
            ref left,
            ref right,
        } = self
        {
            let location_url = format!(
                "{}/v2/{}/{}/blobs/uploads/{}?query=true",
                get_base_url(req),
                name,
                repo,
                uuid
            );
            let upload_uuid = Header::new("Docker-Upload-UUID", uuid.clone());
            let range = Header::new("Range", format!("{}-{}", left, right));
            let length = Header::new("X-Content-Length", format!("{}", right - left));
            let location = Header::new("Location", location_url);

            debug!("Range: {}-{}, Length: {}", left, right, right - left);
            Response::build()
                .header(upload_uuid)
                .header(location)
                .header(range)
                .header(length)
                // TODO: move into the type so it is better encoded?...
                .status(Status::Accepted)
                .ok()
        } else {
            debug!("Uuid Error");
            Response::build().status(Status::NotFound).ok()
        }
    }
}

#[cfg(test)]
mod test {
    use rocket::http::Status;
    use response::uuid::UuidResponse;

    use test::test_helpers::test_route;
    fn build_response() -> UuidResponse {
        UuidResponse::Uuid {
            // TODO: keep this as a real Uuid!
            uuid: String::from("whatever"),
            name: String::from("moredhel"),
            repo: String::from("test"),
            left: 0,
            right: 0,
        }
    }

    #[test]
    fn uuid_uuid() {
        let response = test_route(build_response());
        let headers = response.headers();
        assert_eq!(response.status(), Status::Accepted);
        assert!(headers.contains("Docker-Upload-UUID"));
        assert!(headers.contains("Location"));
        assert!(headers.contains("Range"));
    }

    #[test]
    fn uuid_empty() {
        let response = test_route(UuidResponse::Empty);
        assert_eq!(response.status(), Status::NotFound);
    }
}
