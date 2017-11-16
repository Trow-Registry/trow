// TODO: move me somewhere else
#[derive_FromForm]
#[derive(Debug)]
pub struct DigestStruct {
    pub query: bool,
    pub digest: String,
}
