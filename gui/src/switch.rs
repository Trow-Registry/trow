use yew_router::{components::RouterAnchor, prelude::*, switch::Permissive};

#[derive(Clone, Debug, Switch)]
pub enum AppRoute {
    #[to = "/repositories"]
    Repositories,
    #[to = "/settings"]
    Settings,
    #[to = "/!"]
    Home,
    #[to = "/page-not-found"]
    PageNotFound(Permissive<String>),
}

pub type AppRouter = Router<AppRoute>;
pub type AppAnchor = RouterAnchor<AppRoute>;

// Note: using repo/tag query strings with routes did not work due to
// repos and tags having url unsafe strings such as "/", "-"
// url encoding was an option but ended up using yew callbacks.
