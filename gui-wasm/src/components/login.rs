use yew::prelude::*;
use yew::{
    format::Json,
    services::{
        fetch::FetchTask,
        storage::{Area, StorageService},
    },
};

use crate::error::ApiError;
use crate::services::auth::{AuthResponse, AuthSvc};

pub struct Login {
    link: ComponentLink<Self>,
    state: State,
    authenticating: bool,
    auth_svc: AuthSvc,
    storage: StorageService,
    auth_task: Option<FetchTask>,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {}

pub struct State {
    username: String,
    password: String,
}
pub enum Msg {
    UpdateUsername(String),
    UpdatePassword(String),
    Authenticate,
    AuthenticateReady(Result<AuthResponse, ApiError>),
    Nothing,
}

impl Component for Login {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Session).expect("storage was disabled by the user");

        let state = State {
            username: "".into(),
            password: "".into(),
        };

        Self {
            state,
            link,
            authenticating: false,
            auth_svc: AuthSvc::new(),
            storage,
            auth_task: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateUsername(username_value) => {
                self.state.username = username_value;
                false
            }
            Msg::UpdatePassword(password_value) => {
                self.state.password = password_value;
                false
            }

            Msg::Authenticate => {
                log::debug!("auth user {}", self.state.username);
                let callback = self.link.callback(Msg::AuthenticateReady);
                let task = self.auth_svc.login(
                    self.state.username.clone(),
                    self.state.password.clone(),
                    callback,
                );
                self.auth_task = Some(task);
                true
            }
            Msg::AuthenticateReady(Ok(response)) => {
                self.authenticating = false;
                self.auth_task = None;
                log::debug!("token: {}", response.token);

                self.storage
                    .store(crate::AUTH_TOKEN_KEY, Json(&response.token));

                true
            }

            Msg::AuthenticateReady(Err(_)) => {
                self.authenticating = false;
                false
            }

            Msg::Nothing => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
    fn view(&self) -> Html {
        html! {

            <div class="home-segment-login">
                <form>
                    <div class="uk-margin">
                        <div class="uk-inline">
                            <span class="uk-form-icon" uk-icon="icon: user"></span>
                            <input
                                class="uk-input"
                                type="username"
                                placeholder="Username"
                                oninput=self.link.callback(|e: InputData| Msg::UpdateUsername(e.value))
                            />
                        </div>
                    </div>

                    <div class="uk-margin">
                        <div class="uk-inline">
                            <span class="uk-form-icon" uk-icon="icon: lock"></span>
                            <input
                                class="uk-input"
                                type="password"
                                placeholder="Password"
                                oninput=self.link.callback(|e: InputData| Msg::UpdatePassword(e.value))
                                onkeypress=self.link.callback(|e: KeyboardEvent| {
                                    if e.key() == "Enter" { Msg::Authenticate } else { Msg::Nothing }
                                })
                                />
                        </div>
                    </div>

                    <div class="field">
                        <p class="control">
                        <button
                            onclick=self.link.callback(|ev: MouseEvent| {
                                ev.prevent_default();
                                Msg::Authenticate
                            })
                            class="uk-button uk-button-default">
                            {"Login"}

                        </button>
                        </p>
                    </div>
                </form>

            </div>
        }
    }
}
