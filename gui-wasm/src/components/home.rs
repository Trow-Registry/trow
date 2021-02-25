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
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }
    fn view(&self) -> Html {
        html! {
            <div class="columns">
                <div class="column">
                    <div class="content is-centered">

                        <h4>{"Trow"}</h4>
                        <h6>{"The Cloud Native Registry"}</h6>
                        <button class="button">
                            <AppAnchor  route=AppRoute::Repositories>
                                { "Repositories" }
                            </AppAnchor>
                            </button>
                    </div>
                </div>
                <div class="column">
                    <Login />
                </div>
            </div>
        }
    }
}
