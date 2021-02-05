import React, { useEffect, memo, Suspense } from "react";
import { List, Grid, Segment } from "semantic-ui-react";
import { Link, useRouteMatch } from "react-router-dom";
import queryString from "query-string";
import { useRecoilValue, useSetRecoilState } from "recoil";

import { catalogState, currentRepositoryState } from "../../store/atoms";

import { MemoisedRepo } from "../repo";
import { MemoisedMainHeader } from "../header";
import SuspenseLoader from "../loader";

import NavVertical from "../nav";

const Catalog = () => {
    const defaultCatalog: string[]  = []
    const catalogList = useRecoilValue(catalogState) || defaultCatalog;
    const { url } = useRouteMatch();

    const setCurrentRepository = useSetRecoilState(currentRepositoryState);

    const parsed: any = queryString.parse(location.search);
    const repo: string = parsed.repo;

    useEffect(() => {
        setCurrentRepository(repo);
    }, [repo]);

    return (
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
                <Grid.Column width={2}>
                    <Segment basic>
                        <List selection verticalAlign="middle" divided animated>
                            {catalogList.map(
                                (catalogItem: string, index: number) => (
                                    <List.Item
                                        key={index}
                                        as={Link}
                                        repo={catalogItem}
                                        to={{
                                            pathname: url,
                                            search: `?repo=${catalogItem}`,
                                        }}
                                    >
                                        <List.Content>
                                            <List.Header>
                                                {catalogItem}
                                            </List.Header>
                                        </List.Content>
                                    </List.Item>
                                )
                            )}
                        </List>
                    </Segment>
                </Grid.Column>
                <MemoisedRepo repo={repo} />
            </Grid>
        </Suspense>
    );
};

export const MemoisedCatalog = memo(Catalog);
