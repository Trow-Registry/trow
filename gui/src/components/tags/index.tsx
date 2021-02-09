import React, { useEffect, useRef, createRef } from "react";
import { List, Input } from "semantic-ui-react";
import { Link, useRouteMatch } from "react-router-dom";
import { useRecoilValue, useSetRecoilState } from "recoil";
import queryString from "query-string";

// import SuspenseLoader from "../loader";
import config from "../../../config";

import { currentTagState } from "../../store/atoms";
import { currentRepoTagsQuery } from "../../store/selectors";

interface RepoTagsSchema {
    tags: [];
    name: string;
}

const defaultRepoTagsSchema: RepoTagsSchema = { tags: [], name: "" };

const Tags = ({ repo }) => {
    const copyRefs = useRef([]);
    const { url } = useRouteMatch();
    const tagsResponse =
        useRecoilValue(currentRepoTagsQuery) ?? defaultRepoTagsSchema;

    const setCurrentTag = useSetRecoilState(currentTagState);
    const parsedHash: any = queryString.parse(location.hash);
    const tag: string = parsedHash.tag;

    const copyText = (index: number) => {
        copyRefs.current[index].select();
        document.execCommand("copy");
    };

    useEffect(() => {
        copyRefs.current = tagsResponse.tags.map(() => createRef());
    }, [repo]);

    useEffect(() => {
        setCurrentTag(tag);
        return function cleanup() {
            setCurrentTag("");
        };
    }, [tag]);

    return (
        <List selection verticalAlign="middle" divided animated>
            {tagsResponse.tags.map((tag, index) => (
                <List.Item key={`${tag}${index}`}>
                    <List.Content>
                        <Link
                            to={{
                                pathname: url,
                                search: `?repo=${repo}`,
                                hash: `#tag=${tag}`,
                            }}
                            key={`${tag}/${index}`}
                        >
                            {tag}
                        </Link>
                    </List.Content>
                    <List.Content floated="right">
                        <Input
                            key={index}
                            action={{
                                color: "teal",
                                icon: "copy",
                                onClick: () => copyText(index),
                            }}
                            value={`docker pull ${config.trow_registry_url}/${repo}:${tag}`}
                            ref={(el) => (copyRefs.current[index] = el)}
                        />
                    </List.Content>
                </List.Item>
            ))}
        </List>
    );
};

export default Tags;
