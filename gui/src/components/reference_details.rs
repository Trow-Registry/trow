use yew::format::Json;
use yew::services::storage::{Area, StorageService};
use yew::{prelude::*, services::fetch::FetchTask};

use crate::error::ApiError;
use crate::services::blob::BlobSvc;
use crate::services::manifest::ManifestSvc;
use crate::types::{blob::Blob, manifest::Manifest};

pub struct ReferenceDetails {
    props: Props,
    link: ComponentLink<Self>,
    fetching: bool,
    blob: Option<Blob>,
    manifest: Option<Manifest>,
    fetch_task_blob: Option<FetchTask>,
    fetch_task_manifest: Option<FetchTask>,
    manifest_svc: ManifestSvc,
    blob_svc: BlobSvc,
    storage: StorageService,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    #[prop_or(String::from(""))]
    pub reference: String,
    #[prop_or(String::from(""))]
    pub repository: String,
}
pub enum Msg {
    FetchManifestResponseReady(Result<Manifest, ApiError>),
    FetchBlobResponseReady(Result<Blob, ApiError>),
}

impl Component for ReferenceDetails {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).expect("storage was disabled by the user");

        let mut ref_details = Self {
            props,
            link,
            fetching: false,
            manifest: None,
            blob: None,
            fetch_task_blob: None,
            fetch_task_manifest: None,
            manifest_svc: ManifestSvc::new(),
            blob_svc: BlobSvc::new(),
            storage,
        };
        let repo = ref_details.props.repository.clone();
        let reference = ref_details.props.reference.clone();

        if !repo.is_empty() {
            if !reference.is_empty() {
                ref_details.fetching = true;
                let callback = ref_details.link.callback(Msg::FetchManifestResponseReady);
                let task =
                    ref_details
                        .manifest_svc
                        .fetch(repo.clone(), reference.clone(), callback);
                ref_details.fetch_task_manifest = Some(task);
            }
        }

        ref_details
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchManifestResponseReady(Ok(response)) => {
                self.fetching = false;
                self.fetch_task_manifest = None;
                self.manifest = Some(response.clone());
                let callback = self.link.callback(Msg::FetchBlobResponseReady);
                let task_blob = self.blob_svc.fetch(
                    self.props.repository.clone(),
                    response.config.digest.clone(),
                    callback,
                );
                self.fetch_task_blob = Some(task_blob);
                log::info!("{:?}", self.manifest);
                true
            }

            Msg::FetchManifestResponseReady(Err(_)) => false,

            Msg::FetchBlobResponseReady(Ok(response)) => {
                self.fetch_task_blob = None;
                self.blob = Some(response);
                log::info!("{:?}", self.blob);

                true
            }

            Msg::FetchBlobResponseReady(Err(_)) => false,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let changed = self.props != props;
        if changed {
            self.props = props.clone();
            let callback = self.link.callback(Msg::FetchManifestResponseReady);
            let task = self.manifest_svc.fetch(
                self.props.repository.clone(),
                self.props.reference.clone(),
                callback,
            );
            self.fetch_task_manifest = Some(task);
        }
        changed
    }
    fn view(&self) -> Html {
        let repo_reference = format!("{}:{}", &self.props.repository, &self.props.reference);

        html! {
            <>
                <div class="uk-section">
                    <div class="uk-card uk-card-default uk-card-body">
                        // <h6>{"Tag Info"}</h6>
                        <p class="uk-card-title">{repo_reference}</p>
                        { self.view_pull_details()}
                        <div class="uk-section">
                            { self.view_fetching_manifest()}
                            { self.view_fetching_blob()}
                            { self.view_blob_details()}
                            { self.view_manifest_details()}
                        </div>
                    </div>
                </div>
                <div class="uk-section">
                    <h6>{"Scans"}</h6>
                    <p>{"No scan data available"}</p>
                </div>
            </>
        }
    }
}

impl ReferenceDetails {
    fn view_pull_details(&self) -> Html {
        let registry_url =
            if let Json(Ok(registry_url_value)) = self.storage.restore(crate::REGISTRY_KEY) {
                registry_url_value
            } else {
                String::from(crate::DEFAULT_REGISTRY_URL)
            };

        let pull_command = format!(
            "docker pull {}/{}:{}",
            registry_url.replace("https://", ""),
            &self.props.repository,
            &self.props.reference
        );

        html! {
            <code class="uk-align-right">{pull_command}</code>
        }
    }

    fn view_manifest_details(&self) -> Html {
        if let Some(manifest) = &self.manifest {
            html! {
                <>
                    <p><span class="uk-text-italic">{"digest: "}</span> {&manifest.config.digest}</p>
                </>

            }
        } else {
            html! {
                <p></p>
            }
        }
    }

    fn view_blob_details(&self) -> Html {
        if let Some(blob) = &self.blob {
            html! {
                <>
                    <p><span class="uk-text-italic">{"os/arch: "}</span>{&blob.os} {"/"} {&blob.architecture}</p>
                </>
            }
        } else {
            html! {
                <p></p>
            }
        }
    }

    fn view_fetching_manifest(&self) -> Html {
        if self.fetch_task_manifest.is_some() {
            html! { <p>{ "Fetching manifest..." }</p> }
        } else {
            html! { <p></p> }
        }
    }

    fn view_fetching_blob(&self) -> Html {
        if self.fetch_task_blob.is_some() {
            html! { <p>{ "Fetching blob..." }</p> }
        } else {
            html! { <p></p> }
        }
    }
}
