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
            <>
            <div class="field">
                <p class="control has-icons-left has-icons-right">
                <input class="input" type="username" placeholder="Username" />
                <span class="icon is-small is-left">
                    <i class="fas fa-user"></i>
                </span>
                <span class="icon is-small is-right">
                    <i class="fas fa-check"></i>
                </span>
                </p>
            </div>
            <div class="field">
                <p class="control has-icons-left">
                <input class="input" type="password" placeholder="Password" />
                <span class="icon is-small is-left">
                    <i class="fas fa-lock"></i>
                </span>
                </p>
            </div>
            <div class="field">
                <p class="control">
                <button class="button is-success">
                    {"Login"}
                </button>
                </p>
            </div>
          </>
        }
    }
}
