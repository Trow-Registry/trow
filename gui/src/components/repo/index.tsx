import React, { useState, useRef, memo, useCallback, Suspense } from "react";
import {
    Grid,
    Segment,
    Menu,
    Input,
    Container,
    Header,
} from "semantic-ui-react";

import SuspenseLoader from "../loader";

import config from "../../../config";

import Details from "../details";
import Tags from "../tags";

const Repo = ({ repo }) => {
    const [activeItem, setActiveItem] = useState("tags");
    const copyRef = useRef(null);

    const handleItemClick = useCallback(
        (e, { name }) => {
            setActiveItem(name);
        },
        [activeItem]
    );

    const copyText = useCallback(() => {
        copyRef.current.select();
        document.execCommand("copy");
    }, []);

    return (
        <Suspense fallback={<SuspenseLoader />}>
            {repo ? (
                <>
                    <Grid.Column width={8}>
                        <Segment basic>
                            <Container fluid textAlign="left">
                                <Header as="h3">{repo}</Header>
                            </Container>
                            <Container textAlign="center"></Container>
                            <Container textAlign="right">
                                <Input
                                    action={{
                                        color: "teal",
                                        icon: "copy",
                                        onClick: copyText,
                                    }}
                                    value={`docker pull ${config.trow_registry_url}/${repo}`}
                                    ref={copyRef}
                                />
                            </Container>
                            <Menu pointing secondary>
                                <Menu.Item
                                    name="tags"
                                    active={activeItem === "tags"}
                                    onClick={handleItemClick}
                                />

                                <Menu.Item
                                    name="description"
                                    active={activeItem === "description"}
                                    onClick={handleItemClick}
                                />
                            </Menu>
                        </Segment>

                        <Suspense fallback={<SuspenseLoader />}>
                            <Segment basic>
                                {activeItem == "tags" ? (
                                    <Tags repo={repo} />
                                ) : (
                                    <>
                                        <Container fluid>
                                            <strong>Name:</strong> {repo}
                                        </Container>
                                    </>
                                )}
                            </Segment>
                        </Suspense>
                    </Grid.Column>
                    <Suspense fallback={<SuspenseLoader />}>
                        <Details />
                    </Suspense>
                </>
            ) : (
                <div />
            )}
        </Suspense>
    );
};

export const MemoisedRepo = memo(Repo);
