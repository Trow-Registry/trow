import React from "react";
import { Dimmer, Loader, Segment } from "semantic-ui-react";

const SuspenseLoader = () => (
    <Segment basic>
        <Dimmer inverted active>
            <Loader content="Loading.." />
        </Dimmer>
    </Segment>
);

export default SuspenseLoader;
