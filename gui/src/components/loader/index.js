import React from "react";
import { Segment } from "semantic-ui-react";

const SuspenseLoader = () => (
    <Segment loading basic id="loaderSegment">
        {/* <Dimmer inverted active>
            <Loader content="Loading.." />
        </Dimmer> */}
    </Segment>
);

export default SuspenseLoader;
