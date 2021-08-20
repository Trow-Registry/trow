use rocket::{fairing::Fairing, Rocket};

pub trait AttachConditionalFairing {
    fn attach_if(self, condition: bool, fairing: impl Fairing) -> Self;
}

impl AttachConditionalFairing for Rocket<rocket::Build> {
    #[inline]
    fn attach_if(self, condition: bool, fairing: impl Fairing) -> Self {
        if condition {
            self.attach(fairing)
        } else {
            self
        }
    }
}
