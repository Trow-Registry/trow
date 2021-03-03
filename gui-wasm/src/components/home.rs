use yew::prelude::*;

use crate::components::login::Login;
use crate::switch::{AppAnchor, AppRoute};
pub struct Home {
    // props: Props,
// link: ComponentLink<Self>,
}
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {}

pub enum Msg {}

impl Component for Home {
    type Properties = Props;
    type Message = Msg;

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
            <div class="uk-grid uk-child-width-expand@s uk-text-center uk-height-viewport" >
                <div class="">
                    <div class="home-segment-text">
                        <h4 class="uk-heading-small">{"Trow"}</h4>
                        <h6>{"The Cloud Native Registry"}</h6>
                        <button class="uk-button uk-button-default">
                            <AppAnchor  route=AppRoute::Repositories>
                                { "Repositories" }
                            </AppAnchor>
                        </button>
                    </div>
                </div>
                <div class="login">
                    <Login />
                </div>
            </div>
        }
    }
}
