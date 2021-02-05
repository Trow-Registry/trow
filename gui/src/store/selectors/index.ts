import { selector, selectorFamily } from "recoil";

import { getCatalog, getRepoTags, getManifest, getBlob } from "../../api";
import {
    currentRepositoryState,
    currentTagState,
    currentBlobDigestState,
} from "../atoms";

// Catalog
export const repositoriesQuery = selector({
    key: "repositoriesQuery", // unique ID (with respect to other atoms/selectors)
    get: async () => {
        const response = await getCatalog();

        return response;
    },
});

// Tags
export const tagsQuery = selectorFamily({
    key: "tagsQuery",
    get: (repoName: string) => async () => {
        const response = await getRepoTags({ repoName });
        return response;
    },
});

export const currentRepoTagsQuery = selector({
    key: "currentRepoTagsQuery",
    get: ({ get }) => get(tagsQuery(get(currentRepositoryState))),
});

// Manifests
export const manifestQuery = selectorFamily({
    key: "manifestQuery",
    get: ({
        repoName,
        reference,
    }: {
        repoName: string;
        reference: string;
    }) => async () => {
        const response = await getManifest({ repoName, reference });
        return response;
    },
});

export const currentManifestQuery = selector({
    key: "currentManifestQuery",
    get: ({ get }) =>
        get(
            manifestQuery({
                repoName: get(currentRepositoryState),
                reference: get(currentTagState),
            })
        ),
});

// Blobs
export const blobQuery = selectorFamily({
    key: "blobQuery",
    get: ({
        repoName,
        digest,
    }: {
        repoName: string;
        digest: string;
    }) => async () => {
        const response = await getBlob({ repoName, digest });
        return response;
    },
});

export const currentBlobQuery = selector({
    key: "currentBlobQuery",
    get: ({ get }) =>
        get(
            blobQuery({
                repoName: get(currentRepositoryState),
                digest: get(currentBlobDigestState),
            })
        ),
});
