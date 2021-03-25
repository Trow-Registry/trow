use yew::format::Json;
use yew::prelude::*;
use yew::services::storage::{Area, StorageService};
use yew::{events::KeyboardEvent, InputData};

use crate::components::nav::Nav;

pub struct Settings {
    storage: StorageService,
    link: ComponentLink<Self>,
    state: State,
}

pub struct State {
    registry_value: String,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {}

pub enum Msg {
    UpdateRegistryValue(String),
    SaveUpdate,
    Nothing,
}

impl Component for Settings {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).expect("storage was disabled by the user");

        let state = State {
            registry_value: "".into(),
        };

        Settings {
            link,
            storage,
            state,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateRegistryValue(value) => {
                self.state.registry_value = value;
            }

            Msg::SaveUpdate => {
                if !self.state.registry_value.is_empty() {
                    self.storage
                        .store(crate::REGISTRY_KEY, Json(&self.state.registry_value));
                };
            }

            Msg::Nothing => {}
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
    fn view(&self) -> Html {
        html! {
            <div class="uk-grid uk-child-width-expand@s uk-grid-divider uk-height-viewport">
            <Nav />
            <div class="">
                <div class="uk-margin-top"></div>
                    <form  class="uk-form-horizontal">
                        <div class="uk-margin">
                            <legend class="uk-legend">{"Configure"}</legend>
                            <div class="uk-margin-top"></div>
                            <label class="uk-form-label" for="form-horizontal-text">{"Trow Registry Endpoint"}</label>
                            <div class="uk-inline">
                                <span class="uk-form-icon" uk-icon="icon: link"></span>
                                <input
                                class="uk-input"
                                type="endpoint"
                                oninput=self.link.callback(|e: InputData| Msg::UpdateRegistryValue(e.value))
                                onkeypress=self.link.callback(|e: KeyboardEvent| {
                                    if e.key() == "Enter" { Msg::SaveUpdate } else { Msg::Nothing }
                                })
                                placeholder="Trow registry endpoint"/>
                            </div>
                        </div>
                        <div class="field">
                            <p class="control">
                                <button
                                    class="uk-button uk-button-default"
                                    onclick=self.link.callback(|ev: MouseEvent| {
                                        ev.prevent_default();
                                        Msg::SaveUpdate
                                    })
                                >
                                    {"Update"}
                                </button>
                            </p>
                        </div>
                    </form>

                </div>
            </div>
        }
    }
}
