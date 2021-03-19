use std::collections::HashSet;

use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{Header, Method},
};
use rocket::{Request, Response};

#[derive(Default, Clone)]
pub struct CORS {
    pub allow_origin: &'static str,
    pub expose_headers: HashSet<&'static str>,
    pub allow_credentials: Option<bool>,
    pub allow_headers: HashSet<&'static str>,
    pub allow_methods: HashSet<Method>,
    pub max_age: Option<usize>,
}
 
impl CORS {
    pub fn new() -> CORS{
        CORS { 
            allow_origin: "",
            expose_headers: HashSet::new(),
            allow_credentials: None,
            allow_headers: HashSet::new(),
            allow_methods: HashSet::new(),
            max_age: None
        }
    }
    #[allow(dead_code)]
    pub fn credentials(mut self, allow: Option<bool>) -> CORS {
        self.allow_credentials = allow;
        self
    }

    #[allow(dead_code)]
    pub fn headers(mut self, headers: Vec<&'static str>) -> CORS {
        for header in headers {
            self.allow_headers.insert(header);
        }

        self
    }

    #[allow(dead_code)]
    pub fn methods(mut self, methods: Vec<Method>) -> CORS {
        for method in methods {
            self.allow_methods.insert(method);
        }

        self
    }

    #[allow(dead_code)]
    pub fn origin(mut self, origin: &'static str) -> CORS {
        self.allow_origin = origin;
        self
    }

    pub fn build(self) -> CORS {
        CORS { 
            allow_origin: self.allow_origin,
            allow_headers: self.allow_headers,
            allow_methods: self.allow_methods,
            allow_credentials: self.allow_credentials,
            expose_headers: self.expose_headers,
            max_age: self.max_age
        }
    }
}

impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "CORS",
            kind: Kind::Response,
        }
    }

    fn on_response(&self, _: &Request, response: &mut Response) {
        if !self.allow_methods.is_empty() {
            let mut methods = String::with_capacity(self.allow_methods.len() * 7);
            for (i, method) in self.allow_methods.iter().enumerate() {
                if i != 0 { methods.push_str(", ") }
                methods.push_str(method.as_str());
            }
            response.set_header(Header::new(
                "Access-Control-Allow-Methods",
                methods,
            ));
        }

        if !self.allow_origin.is_empty() {
            response.set_header(Header::new("Access-Control-Allow-Origin", String::from(self.allow_origin)));
        }

        if !self.allow_headers.is_empty() {
            let mut headers = String::with_capacity(self.allow_headers.len() * 7);
            for (i, header) in self.allow_headers.iter().enumerate() {
                if i != 0 { headers.push_str(", ") }
                headers.push_str(header);
            }
            response.set_header(Header::new("Access-Control-Allow-Headers", headers));
        }

        
        match self.allow_credentials {
            Some(allow) => response.set_header(Header::new("Access-Control-Allow-Credentials",allow.to_string())),
            None => false
        };
    }
}
