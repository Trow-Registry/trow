use std::collections::HashSet;
// --enable-cors
// --allow-cors-origin "*"

use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
};
use rocket::{Request, Response};

#[derive(Default, Clone)]
pub struct CORS {
    pub allow_origin: String,
    pub expose_headers: HashSet<String>,
    pub allow_credentials: Option<bool>,
    pub allow_headers: HashSet<String>,
    pub allow_methods: HashSet<String>,
    pub max_age: Option<usize>,
}

impl CORS {
    pub fn new() -> CORS {
        CORS {
            allow_origin: "".to_string(),
            expose_headers: HashSet::new(),
            allow_credentials: None,
            allow_headers: HashSet::new(),
            allow_methods: HashSet::new(),
            max_age: None,
        }
    }
    #[allow(dead_code)]
    pub fn credentials(mut self, allow: Option<bool>) -> CORS {
        self.allow_credentials = allow;
        self
    }

    #[allow(dead_code)]
    pub fn headers(mut self, headers: Vec<String>) -> CORS {
        for header in headers {
            self.allow_headers.insert(header);
        }

        self
    }

    #[allow(dead_code)]
    pub fn methods(mut self, methods: Vec<String>) -> CORS {
        for method in methods {
            self.allow_methods.insert(method);
        }

        self
    }

    #[allow(dead_code)]
    pub fn origin(mut self, origin: String) -> CORS {
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
            max_age: self.max_age,
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
                if i != 0 {
                    methods.push_str(", ")
                }
                methods.push_str(method);
            }
            response.set_header(Header::new("Access-Control-Allow-Methods", methods));
        }

        if !self.allow_origin.is_empty() {
            response.set_header(Header::new(
                "Access-Control-Allow-Origin",
                self.allow_origin.clone(),
            ));
        }

        if !self.allow_headers.is_empty() {
            let mut headers = String::with_capacity(self.allow_headers.len() * 7);
            for (i, header) in self.allow_headers.iter().enumerate() {
                if i != 0 {
                    headers.push_str(", ")
                }
                headers.push_str(header);
            }
            response.set_header(Header::new("Access-Control-Allow-Headers", headers));
        }

        match self.allow_credentials {
            Some(true) => response.set_header(Header::new(
                "Access-Control-Allow-Credentials",
                true.to_string(),
            )),
            Some(false) => false,
            None => false,
        };
    }
}
