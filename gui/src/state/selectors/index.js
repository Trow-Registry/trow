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
    get: (repoName) => async () => {
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
    get: ({ repoName, reference }) => async () => {
        const response = await getManifest({ repoName, reference });
        console.log(response);
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
    get: ({ repoName, digest }) => async () => {
        const response = await getBlob({ repoName, digest });
        console.log(response);
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
                // digest: "sha256:a4b51fc0e8756bd76674bffdd32e5a7b0c8d027ec8787984c5bf86f4b9014fb9"
            })
        ),
});
