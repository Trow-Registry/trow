import "semantic-ui-css/semantic.min.css";
import "./style/App.scss";

import React, { Suspense, memo, lazy } from "react";
import ReactDOM from "react-dom";
import { BrowserRouter as Router, Switch, Route } from "react-router-dom";
import { RecoilRoot } from "recoil";
// import { createBrowserHistory } from "history";

// WIP code splitting
const MemoisedHome = lazy(() => import("./components/home"));
const About = lazy(() => import("./components/about"));
const MemoisedCatalog = lazy(() => import("./components/catalog"));

import SuspenseLoader from "./components/loader";

// let history = createBrowserHistory();

const App = () => {
    return (
        <>
            <RecoilRoot>
                <Suspense fallback={<SuspenseLoader />}>
                    <Router>
                        <Switch>
                            <Route exact path="/" component={MemoisedHome} />
                            <Route
                                path="/repositories"
                                component={MemoisedCatalog}
                            />
                            <Route path="/about" component={About} />
                        </Switch>
                    </Router>
                </Suspense>
            </RecoilRoot>
        </>
    );
};

const MemoisedApp = memo(App);

ReactDOM.render(<MemoisedApp />, document.getElementById("root"));
