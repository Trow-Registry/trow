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
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }
    fn view(&self) -> Html {
        html! {

            <div class="content home-segment-login">

                <form class="ui form">
                    <div class="field">
                    <p class="control has-icons-left has-icons-right">
                    <input class="uk-input" type="username" placeholder="Username"/>
                    <span class="icon is-small is-left">
                        <i class="fas fa-user"></i>
                    </span>
                    </p>
                </div>
                <div class="field">
                    <p class="control has-icons-left">
                    <input class="uk-input" type="password" placeholder="Password"/>
        
                    </p>
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
