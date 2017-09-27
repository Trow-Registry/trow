#[cfg(test)]
pub mod test_helpers {

    use rocket;
    use rocket::local::Client;
    use response::RegistryTrait;

    pub fn test_route<'r, A: RegistryTrait>(handler: Result<A, A>) -> rocket::Response<'r> {
        let rocket = rocket::Rocket::ignite();
        let client = Client::new(rocket).expect("valid rocket instance");
        let request = client.get("/");
        let request = request.inner();

        match handler {
            Ok(handler) => handler.ok(&request).unwrap(),
            Err(handler) => handler.err(&request).unwrap(),
        }
    }
}
