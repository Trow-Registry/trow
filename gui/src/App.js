import "semantic-ui-css/semantic.min.css";
import "./style/App.scss";

import React, { Suspense, lazy } from "react";
import ReactDOM from "react-dom";
import { BrowserRouter as Router, Switch, Route } from "react-router-dom";
import { RecoilRoot } from "recoil";
import { createBrowserHistory } from "history";

// WIP code splitting
const Home = lazy(() => import("./components/home"));
const Login = lazy(() => import("./components/login"));
const About = lazy(() => import("./components/about"));
const Catalog = lazy(() => import("./components/catalog"));

import SuspenseLoader from "./components/loader";

let history = createBrowserHistory();

const App = () => {
    return (
        <div>
            <RecoilRoot>
                <Suspense fallback={<SuspenseLoader />}>
                    <Router history={history}>
                        <Switch>
                            <Route exact path="/" component={Home} />
                            <Route path="/login" component={Login} />
                            <Route path="/repositories" component={Catalog} />
                            <Route path="/about" component={About} />
                        </Switch>
                    </Router>
                </Suspense>
            </RecoilRoot>
        </div>
    );
};

ReactDOM.render(<App />, document.getElementById("root"));
