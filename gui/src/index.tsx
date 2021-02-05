import React, { Suspense } from "react";
import ReactDOM from "react-dom";
import { BrowserRouter as Router } from "react-router-dom";
import { RecoilRoot } from "recoil";

import SuspenseLoader from "./components/loader";

import App from "./App";

ReactDOM.render(
    <Router>
        <Suspense fallback={<SuspenseLoader />}>
            <RecoilRoot>
                <App />
            </RecoilRoot>
        </Suspense>
    </Router>,
    document.getElementById("root")
);
