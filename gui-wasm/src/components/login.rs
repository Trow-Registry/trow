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
    session_storage: StorageService,
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
    Logout,
    Nothing,
}

impl Component for Login {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let session_storage =
            StorageService::new(Area::Session).expect("storage was disabled by the user");
        let storage = StorageService::new(Area::Local).expect("storage was disabled by the user");

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
            session_storage,
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

                self.storage
                    .store(crate::USERNAME_KEY, Json(&self.state.username));

                self.auth_task = Some(task);
                true
            }

            Msg::AuthenticateReady(Ok(response)) => {
                self.authenticating = false;
                self.auth_task = None;
                self.session_storage
                    .store(crate::AUTH_TOKEN_KEY, Json(&response.token));

                true
            }

            Msg::AuthenticateReady(Err(_)) => {
                self.authenticating = false;
                false
            }

            Msg::Logout => {
                self.storage.remove(crate::USERNAME_KEY);

                self.session_storage.remove(crate::AUTH_TOKEN_KEY);

                true
            }

            Msg::Nothing => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
    fn view(&self) -> Html {
        let username = if let Json(Ok(username_value)) = self.storage.restore(crate::USERNAME_KEY) {
            username_value
        } else {
            "".to_string()
        };

        if username.is_empty() {
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
        } else {
            html! {
                <div class="home-segment-login username">
                    <div class="uk-container-large">
                        <div class="uk-heading-small">{username}</div>
                    </div>
                    <div class="field">
                        <p class="control">
                        <button
                            onclick=self.link.callback(|ev: MouseEvent| {
                                ev.prevent_default();
                                Msg::Logout
                            })
                            class="uk-button uk-button-default">
                            {"Logout"}

                        </button>
                        </p>
                    </div>
                </div>
            }
        }
    }
}
