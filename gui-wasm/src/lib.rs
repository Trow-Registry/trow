#![recursion_limit = "256"]

use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod app;
mod components;
mod error;
mod services;
mod switch;
mod types;

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    App::<app::Model>::new().mount_to_body();
}
