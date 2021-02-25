use crate::components::{reference_details::ReferenceDetails, repository::Repository};
use crate::switch::{AppAnchor, AppRoute};

use yew::prelude::*;
use yew::services::fetch::FetchTask;
use yew::Properties;

use crate::error::ApiError;
use crate::services::repositories::{RepositoriesResponse, RepositoriesSvc};

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    #[prop_or(String::from(""))]
    pub reference: String,
    #[prop_or(String::from(""))]
    pub repository: String,
}

pub struct Catalog {
    repositories_svc: RepositoriesSvc,
    fetching: bool,
    repositories: Option<Vec<String>>,
    link: ComponentLink<Self>,
    fetch_catalog_task: Option<FetchTask>,
    props: Props,
}

pub enum Msg {
    FetchCatalogResponseReady(Result<RepositoriesResponse, ApiError>),
    SetCurrentRepository(String),
    SetCurrentReference(String),
}

impl Component for Catalog {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut catalog = Self {
            props,
            link,
            repositories_svc: RepositoriesSvc::new(),
            repositories: None,
            fetching: false,
            fetch_catalog_task: None,
        };

        catalog.fetching = true;
        let callback = catalog.link.callback(Msg::FetchCatalogResponseReady);
        let task = catalog.repositories_svc.fetch(callback);
        catalog.fetch_catalog_task = Some(task);
        catalog
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchCatalogResponseReady(Ok(response)) => {
                self.fetching = false;
                self.fetch_catalog_task = None;
                self.repositories = Some(response.repositories);
                true
            }

            Msg::FetchCatalogResponseReady(Err(_)) => false,

            Msg::SetCurrentRepository(repository) => {
                self.props.repository = repository;
                self.props.reference = String::from("");
                true
            }

            Msg::SetCurrentReference(reference) => {
                self.props.reference = reference;
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let changed = self.props != props;
        if changed {
            self.props = props;
            log::info!("{}", self.props.repository)
        }
        changed
    }
    fn view(&self) -> Html {
        html! {
            <div class="columns">
                <div class="column is-1">
                    <aside class="menu">
                        <p class="menu-label">
                        <AppAnchor  route=AppRoute::Home>
                            { "Trow" }
                        </AppAnchor>
                        </p>

                        <ul class="menu-list">
                            <li><AppAnchor  route=AppRoute::Repositories>
                                { "Repositories" }
                            </AppAnchor>
                            </li>
                        </ul>

                    </aside>
                </div>

                <div class="column is-1">

                { self.view_fetching() }
                { self.view_repositories()}
                </div>

                <div class="column">
                    {self.view_repo()}
                </div>
                <div class="column">
                    {self.view_reference_details()}
                </div>

            </div>
        }
    }
}

impl Catalog {
    fn view_repositories(&self) -> Html {
        if let Some(repositories) = &self.repositories {
            let repos_render = repositories.iter().map(|repo| {
                let r = repo.clone();
                let onclick = self.link.callback(move |ev: MouseEvent| {
                    ev.prevent_default();
                    Msg::SetCurrentRepository(r.to_string())
                });
                html! {
                    <li onclick=onclick >{repo.to_string()}</li>
                }
            });
            html! {
                <div class="content">
                    <ul class="repo-list">
                        { for repos_render}
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
        if self.fetch_catalog_task.is_some() {
            html! { <p>{ "Fetching repositories..." }</p> }
        } else {
            html! { <p></p> }
        }
    }

    fn view_nav(&self) -> Html {
        unimplemented!()
    }

    fn view_repo(&self) -> Html {
        if !self.props.repository.is_empty() {
            let callback_reference = self.link.callback(Msg::SetCurrentReference);
            html! { <Repository reference=&self.props.reference repository=&self.props.repository callback_reference=callback_reference/> }
        } else {
            html! { <p></p> }
        }
    }

    fn view_reference_details(&self) -> Html {
        if !self.props.reference.is_empty() {
            html! { <ReferenceDetails reference=&self.props.reference repository=&self.props.repository/> }
        } else {
            html! { <p></p> }
        }
    }
}
