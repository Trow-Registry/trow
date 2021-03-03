use yew::prelude::*;

pub struct Login {
    // props: Props,
// link: ComponentLink<Self>,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {}

pub enum Msg {}

impl Component for Login {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        unimplemented!()
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
                            <input class="uk-input" type="username" placeholder="Username"/>
                        </div>
                    </div>

                    <div class="uk-margin">
                        <div class="uk-inline">
                            <span class="uk-form-icon" uk-icon="icon: lock"></span>
                            <input class="uk-input" type="password" placeholder="Password"/>
                        </div>
                    </div>

                    <div class="field">
                        <p class="control">
                        <button class="uk-button uk-button-default">
                            {"Login"}
                        </button>
                        </p>
                    </div>
                </form>

            </div>
        }
    }
}
