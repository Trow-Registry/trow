import React, { Fragment, useEffect } from "react";
import { List, Grid, Segment } from "semantic-ui-react";
import { Link, useRouteMatch } from "react-router-dom";
import queryString from "query-string";
import { useRecoilValue, useSetRecoilState } from "recoil";

import { catalogState, currentRepositoryState } from "../../state/atoms";

import Repo from "../repo";
import MainHeader from "../header";
import NavVertical from "../nav";

const defaultCatalogSchema = [];

export default function Catalog() {
    const catalogList = useRecoilValue(catalogState) || defaultCatalogSchema;
    const { url } = useRouteMatch();

    const setCurrentRepository = useSetRecoilState(currentRepositoryState);

    const parsed = queryString.parse(location.search);
    const repo = parsed.repo;

    useEffect(() => {
        setCurrentRepository(repo);
    }, [repo]);

    return (
        <>
            <MainHeader />
            <Grid stackable columns={4}>
                <Grid.Column width={1}>
                    <NavVertical />
                </Grid.Column>
                <Grid.Column width={2}>
                    <Segment basic>
                        <List selection verticalAlign="middle" divided animated>
                            {catalogList.map((catalogItem) => (
                                <List.Item key={catalogItem}>
                                    <List.Content
                                        as={Link}
                                        repo={catalogItem}
                                        to={`${url}?repo=${catalogItem}`}
                                    >
                                        <List.Header>{catalogItem}</List.Header>
                                    </List.Content>
                                </List.Item>
                            ))}
                        </List>
                    </Segment>
                </Grid.Column>
                <Repo repo={repo} />
            </Grid>
        </>
    );
}
