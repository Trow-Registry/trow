#[derive(Clone, Debug, Deserialize)]
pub struct LycaonBackendConfig {
    listen: Service,
    bootstrap: Service,
}

impl LycaonBackendConfig {
    pub fn listen(&self) -> Service {
        self.listen.clone()
    }

    pub fn bootstrap(&self) -> Service {
        self.listen.clone()
    }
}

// DUPLICATED
#[derive(Clone, Debug, Deserialize)]
pub struct Service {
    host: String,
    port: u16,
}

impl Service {
    pub fn host(&self) -> String {
        self.host.clone()
    }

    pub fn port(&self) -> u16 {
        self.port.clone()
    }
}
