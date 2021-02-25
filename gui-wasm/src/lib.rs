#![recursion_limit="256"]

use wasm_bindgen::prelude::*;
use yew::prelude::*;


mod components;
mod switch;
mod types;
mod services;
mod error;
mod app;

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    App::<app::Model>::new().mount_to_body();
}