import React, { useRef, useEffect, memo } from "react";
import { Grid, Segment, Input, Card } from "semantic-ui-react";
import { useRecoilValue, useSetRecoilState } from "recoil";

import config from "../../../config";

import { currentBlobQuery, currentManifestQuery } from "../../store/selectors";
import {
    currentTagState,
    currentRepositoryState,
    currentBlobDigestState,
} from "../../store/atoms";

const defaultManifestSchema = {
    schemaVersion: "",
    config: { mediaType: "", digest: "", size: "" },
    layers: [],
    mediaType: "",
    annotations: {},
};

const Details = () => {
    const manifestResponse =
        useRecoilValue(currentManifestQuery) || defaultManifestSchema;

    const copyRef = useRef(null);
    const tag = useRecoilValue(currentTagState);

    const repo = useRecoilValue(currentRepositoryState);

    const copyText = () => {
        copyRef.current.select();
        document.execCommand("copy");
    };

    const blobResponse = useRecoilValue(currentBlobQuery) || {};

    const setCurrentBlobDigest = useSetRecoilState(currentBlobDigestState);

    useEffect(() => {
        setCurrentBlobDigest(manifestResponse.config.digest);
        return function cleanup() {
            setCurrentBlobDigest("");
        };
    }, [manifestResponse]);

    return (
        <Grid.Column width={5}>
            <Segment basic>
                {tag ? (
                    <>
                        <Card fluid>
                            <Card.Content>
                                <Card.Header
                                    as="h4"
                                    content={`${repo}:${tag}`}
                                />
                                <Card.Meta>
                                    <strong>digest:</strong>{" "}
                                    {manifestResponse.config.digest}
                                </Card.Meta>
                                <Card.Meta>
                                    <strong>os/architecture:</strong>{" "}
                                    {blobResponse.os}/
                                    {blobResponse.architecture}
                                </Card.Meta>
                            </Card.Content>
                            <Card.Content extra>
                                <Input
                                    fluid
                                    action={{
                                        color: "teal",
                                        icon: "copy",
                                        onClick: copyText,
                                    }}
                                    value={`docker pull ${config.trow_registry_url}/${repo}:${tag}`}
                                    ref={copyRef}
                                />
                            </Card.Content>
                        </Card>
                    </>
                ) : (
                    <div />
                )}
            </Segment>
        </Grid.Column>
    );
};

export const MemoisedDetails = memo(Details);
export default Details;
