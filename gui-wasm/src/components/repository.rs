use yew::{prelude::*, services::fetch::FetchTask, Callback};

use crate::error::ApiError;
use crate::services::tags::{TagsResponse, TagsSvc};

pub struct Repository {
    props: Props,
    link: ComponentLink<Self>,
    tags: Option<Vec<String>>,
    fetch_task_tags: Option<FetchTask>,
    fetching: bool,
    tags_svc: TagsSvc,
}
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    #[prop_or(String::from(""))]
    pub reference: String,
    #[prop_or(String::from(""))]
    pub repository: String,
    #[prop_or_default]
    pub callback_reference: Callback<String>,
}

pub enum Msg {
    FetchTagsResponseReady(Result<TagsResponse, ApiError>),
    SetCurrentReference(String),
}

impl Component for Repository {
    type Properties = Props;
    type Message = Msg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut repository = Self {
            props,
            link,
            tags: None,
            fetch_task_tags: None,
            fetching: false,
            tags_svc: TagsSvc::new(),
        };

        if !repository.props.repository.is_empty() {
            repository.fetching = true;
            let callback = repository.link.callback(Msg::FetchTagsResponseReady);
            let task = repository
                .tags_svc
                .fetch(repository.props.repository.clone(), callback);
            repository.fetch_task_tags = Some(task);
        }

        repository
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchTagsResponseReady(Ok(response)) => {
                self.fetching = false;
                self.fetch_task_tags = None;
                self.tags = Some(response.tags);
                true
            }

            Msg::FetchTagsResponseReady(Err(_)) => false,

            Msg::SetCurrentReference(reference) => {
                log::info!("{}", reference);
                self.props.callback_reference.emit(reference);
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let changed = self.props != props;
        if changed {
            self.props = props.clone();
            let callback = self.link.callback(Msg::FetchTagsResponseReady);
            let task = self.tags_svc.fetch(self.props.repository.clone(), callback);
            self.fetch_task_tags = Some(task);
        }
        changed
    }

    fn view(&self) -> Html {
        html! {
            <div class="content">
                <p>{&self.props.repository}</p>
                <p>{&self.props.reference}</p>
                {self.view_fetching()}
                {self.view_tags()}
            </div>
        }
    }
}

impl Repository {
    fn view_tags(&self) -> Html {
        if let Some(tags) = &self.tags {
            let tags_render = tags.iter().map(|tag| {
                let t = tag.clone();
                let onclick = self.link.callback(move |ev: MouseEvent| {
                    ev.prevent_default();
                    Msg::SetCurrentReference(t.to_string())
                });
                html! {
                     <li onclick=onclick >{tag.to_string()}</li>
                }
            });
            html! {
                <div class="content">
                    <ul class="item-list">

                        { for tags_render }
                    </ul>
                </div>
            }
        } else {
            html! {
                <p></p>
            }
        }
    }

    fn view_fetching(&self) -> Html {
        if self.fetch_task_tags.is_some() {
            html! { <p>{ "Fetching tags..." }</p> }
        } else {
            html! { <p></p> }
        }
    }
}
