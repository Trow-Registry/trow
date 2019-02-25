use rocket::http::{Header, Status};
use rocket::http::{ContentType};
use rocket::request::Request;
use rocket::response::{Responder, Response};
/*
 * Generate a WWW-Authenticate header
 */
#[derive(Debug, Serialize)]
pub struct Authenticate {
    pub username: String,
}

impl<'r> Responder<'r> for Authenticate {
    fn respond_to(self, _: &Request)  -> Result<Response<'r>, Status> {
        let authenticate_header = Header::new("www-authenticate","Bearer realm=\"https://0.0.0.0:8443/login\",service=\"trow_registry\",scope=\"push/pull\"");
        Response::build()
            .status(Status::Unauthorized)
            .header(authenticate_header)
            .header(ContentType::JSON)
            .ok()
    }
}
/*
 * 
 */
impl<'a, 'r> FromRequest<'a, 'r> for Authenticate {
    type Error = ();
    fn from_request(req: &'a Request<'r>) -> request::Outcome<Authenticate, ()> {
        // Look in headers for an Authorization header
        let keys: Vec<_> = req.headers().get("Authorization").collect();
        if keys.len() != 1 { // no key return false in auth structure
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        // split the header on white space
        let auth_strings: Vec<String> = keys[0].to_string().split_whitespace().map(String::from).collect();
        if auth_strings.len() !=2 {
            //TODO: Maybe BadRequest?
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        // Basic token is a base64 encoded user/pass
        if auth_strings[0] != "Basic" {
            //TODO: Maybe should forward or something on Basic
            return Outcome::Failure((Status::Unauthorized, ()));
        }
        match base64::decode(&auth_strings[1].to_string()) {
            Ok(decoded) => {
                let mut count=0;
                let mut username = String::new();
                let mut password = String::new();
                while char::from(decoded[count])!=':' {
                    username.push(char::from(decoded[count]));
                    count += 1;
                }
                count+=1;
                while char::from(decoded[count])!='\n' {
                    password.push(char::from(decoded[count]));
                    count += 1;
                }
                if username == "admin" && password == "password" {
                    let authenticate = Authenticate {
                        username,
                    };
                    return Outcome::Success(authenticate);
                }
            }
            _decode_error => {
                return Outcome::Failure((Status::Unauthorized, ()));
            }
//        Outcome::Success(auth_user)
        }
        Outcome::Failure((Status::Unauthorized, ()))
    }
}

#[cfg(test)]
mod test {
    use response::authenticate::Authenticate;
    use rocket::http::Status;

    use response::test_helper::test_route;

    #[test]
    fn authenticate_ok() {
        let response = test_route(Authenticate);
        assert_eq!(response.status(), Status::Unauthorized);
    }
}
