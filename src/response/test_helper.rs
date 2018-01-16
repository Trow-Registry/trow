#[cfg(test)]
use rocket;
#[cfg(test)]
use rocket::response::Responder;
#[cfg(test)]
use rocket::local::Client;

#[cfg(test)]
pub fn test_route<'r, A: Responder<'r>>(handler: A) -> rocket::Response<'r> {
    let rocket = rocket::Rocket::ignite();
    let client = Client::new(rocket).expect("valid rocket instance");
    let request = client.get("/");
    let request = request.inner();

    handler.respond_to(&request).unwrap()
}
