use yew::prelude::*;
use yew_router::{route::Route, switch::Permissive};

use crate::components::{catalog::Catalog, home::Home, settings::Settings};
use crate::switch::{AppRoute, AppRouter};

pub struct Model {
    // link: ComponentLink<Self>,
}

pub enum Msg {}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Model {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        unimplemented!()
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <AppRouter
                render=AppRouter::render(Self::switch)
                redirect=AppRouter::redirect(|route: Route| {
                    AppRoute::PageNotFound(Permissive(Some(route.route)))
                })
            />
        }
    }
}

impl Model {
    fn switch(switch: AppRoute) -> Html {
        match switch {
            AppRoute::Repositories => {
                html! { <Catalog  /> }
            }

            AppRoute::Home => {
                html! { <Home /> }
            }

            AppRoute::Settings => {
                html! { <Settings /> }
            }

            AppRoute::PageNotFound(Permissive(Some(route))) => {
                html! { format!("Page '{:?}' not found", route) }
            }

            AppRoute::PageNotFound(Permissive(None)) => {
                html! {"Page not found"}
            }
        }
    }
}
