import React, { Suspense } from "react";
import { Grid } from "semantic-ui-react";

import { MemoisedMainHeader } from "../header";
import NavVertical from "../nav";

import SuspenseLoader from "../loader";

export default function About() {
    return (
        <>
            <Suspense fallback={<SuspenseLoader />}>
                <MemoisedMainHeader />
                <Grid
                    stackable
                    columns={4}
                    id="catalogGrid"
                    padded="vertically"
                    divided
                >
                    <Grid.Column width={1}>
                        <NavVertical />
                    </Grid.Column>
                    <Grid.Column width={14}></Grid.Column>
                </Grid>
            </Suspense>
        </>
    );
}
