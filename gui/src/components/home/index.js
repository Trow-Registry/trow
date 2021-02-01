import React, { memo } from "react";
import { Button, Segment, Grid } from "semantic-ui-react";
import { Link } from "react-router-dom";

import Login from "../login";

export default function Home() {
    return (
        <Segment basic>
            <Grid stackable columns={2} doubling divided id="homeGrid">
                <Grid.Column>
                    <Segment textAlign="center" basic>
                        <h2>Trow</h2>
                        <h4>The Cloud Native Registry</h4>

                        <Button
                            centered="true"
                            basic
                            as={Link}
                            to="/repositories"
                        >
                            Repositories
                        </Button>
                    </Segment>
                </Grid.Column>
                <Grid.Column>
                    <Login />
                </Grid.Column>
            </Grid>
        </Segment>
    );
}

export const MemoisedHome = memo(Home);
