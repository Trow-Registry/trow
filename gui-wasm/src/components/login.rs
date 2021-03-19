use yew::prelude::*;

pub struct Login {
    // props: Props,
    link: ComponentLink<Self>,
    state: State,
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
    Nothing,
}

impl Component for Login {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let state = State {
            username: "".into(),
            password: "".into(),
        };

        Self { state, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateUsername(username_value) => {
                self.state.username = username_value;
            }
            Msg::UpdatePassword(password_value) => {
                self.state.password = password_value;
            }

            Msg::Authenticate => {
                let _ = self.state.password;
                log::debug!("auth user {}", self.state.username)
            }
            Msg::Nothing => {}
        }
        false
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
