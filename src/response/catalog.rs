use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::request::Request;

use response::json_response;

#[derive(Debug, Serialize)]
pub struct Catalog;

#[derive(Serialize, Deserialize)]
struct CatalogList {
    repositories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Test {
    one: String,
    two: String,
}

impl<'r> Responder<'r> for Catalog {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        warn!("Turn this into a dynamic call!");
        let repositories = CatalogList {
            repositories: vec![String::from("moredhel/test"), String::from("FIX/ME")],
        };

        json_response(req, &repositories)
    }
}

#[cfg(test)]
mod test {
    // use rocket::http::Status;

    // use test::test_helpers::test_route;

    #[test]
    fn catalog_ok() {
        assert_eq!(true, false);
    }
}
