use yew::prelude::*;

use crate::switch::{AppAnchor, AppRoute};

pub struct Nav {
    // props: Props,
// link: ComponentLink<Self>,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {}

pub enum Msg {}

impl Component for Nav {
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
            <div class="uk-width-small">
                <div class="uk-padding-small">
                    <ul class="uk-nav uk-nav-default uk-nav-center">
                        <li class="uk-nav-header">
                            <AppAnchor  route=AppRoute::Home>
                            { "Trow" }
                            </AppAnchor>
                        </li>

                        <li>
                                <a class="uk-link-text">
                                    <span class="uk-icon" uk-icon="icon: folder"></span>
                                    <AppAnchor  route=AppRoute::Repositories>{ "Repositories" }</AppAnchor>
                                </a>
                        </li>

                        <li>
                                <a class="uk-link-text">
                                    <span class="uk-icon" uk-icon="icon: settings"></span>
                                    <AppAnchor  route=AppRoute::Settings>{ "Settings" }</AppAnchor>
                                </a>
                        </li>
                    </ul>
                </div>
            </div>
        }
    }
}
