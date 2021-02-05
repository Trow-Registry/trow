import React from "react";
import { Segment, Dimmer, Loader } from "semantic-ui-react";

const SuspenseLoader = () => (
    <Segment basic id="loaderSegment">
        <Dimmer inverted active>
            <Loader content="Loading.." />
        </Dimmer>
    </Segment>
);

export default SuspenseLoader;
