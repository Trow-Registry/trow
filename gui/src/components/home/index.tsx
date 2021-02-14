import React, { memo } from "react";
import { Button, Segment, Grid } from "semantic-ui-react";
import { Link } from "react-router-dom";
import styled from "styled-components";

const TextSegment = styled(Segment)`
    padding-top: 40% !important;
`;

import Login from "../login";

const Home = () => {
    return (
        <Segment basic>
            <Grid stackable columns={2} doubling divided id="homeGrid">
                <Grid.Column>
                    <TextSegment textAlign="center" basic>
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
                    </TextSegment>
                </Grid.Column>
                <Grid.Column color="teal">
                    <Login />
                </Grid.Column>
            </Grid>
        </Segment>
    );
};

export const MemoisedHome = memo(Home);
