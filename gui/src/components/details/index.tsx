import React, { useRef, useEffect, memo } from "react";
import { Grid, Segment, Input, Card } from "semantic-ui-react";
import { useRecoilValue, useSetRecoilState, useRecoilState } from "recoil";

import config from "../../../config";

import { currentBlobQuery, currentManifestQuery } from "../../store/selectors";
import { defaultManifestSchema } from "../../store/schemas/manifest";
import { defaultBlobSchema } from "../../store/schemas/blob";

import {
    currentTagState,
    currentRepositoryState,
    currentBlobDigestState,
} from "../../store/atoms";

const Details = () => {
    const copyRef = useRef(null);
    const currentRepository = useRecoilValue(currentRepositoryState);
    const currentTag = useRecoilValue(currentTagState);

    const manifestResponse =
        useRecoilValue(currentManifestQuery) ?? defaultManifestSchema;

    const copyText = () => {
        copyRef.current.select();
        document.execCommand("copy");
    };

    const blobResponse = useRecoilValue(currentBlobQuery) ?? defaultBlobSchema;

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
                {currentTag ? (
                    <>
                        <Card fluid>
                            <Card.Content>
                                <Card.Header
                                    as="h4"
                                    content={`${currentRepository}:${currentTag}`}
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
                                    value={`docker pull ${window.location.host}/${currentRepository}:${currentTag}`}
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
