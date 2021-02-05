import "semantic-ui-css/semantic.min.css";
import "./style/App.scss";

import React, { Suspense, memo, lazy } from "react";
import { Switch, Route } from "react-router-dom";

import { MemoisedHome } from "./components/home";
import { MemoisedAbout } from "./components/about";
import { MemoisedCatalog } from "./components/catalog";

const App = () => {
    return (
        <>
            <Switch>
                <Route exact path="/" component={MemoisedHome} />
                <Route path="/repositories" component={MemoisedCatalog} />
                <Route path="/about" component={MemoisedAbout} />
            </Switch>
        </>
    );
};

export default App;
