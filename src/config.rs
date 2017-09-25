/// This encapsulates any stateful data that needs to be preserved and
/// passed around during execution.
#[derive(Debug)]
pub struct State {
    pub address: String,
    pub port: u16,
}
