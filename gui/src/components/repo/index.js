import React, { useState, useRef } from "react";
import {
    Grid,
    Segment,
    Menu,
    Input,
    Container,
    Header,
} from "semantic-ui-react";

import config from "../../../config";

import Details from "../details";
import Tags from "../tags";

export default function Repo({ repo }) {
    const [activeItem, setActiveItem] = useState("tags");
    const copyRef = useRef(null);

    const handleItemClick = (e, { name }) => {
        setActiveItem(name);
    };

    const copyText = () => {
        copyRef.current.select();
        document.execCommand("copy");
    };

    return (
        <>
            {repo ? (
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
                </Grid.Column>
            ) : (
                <div />
            )}
            <Details />
        </>
    );
}
