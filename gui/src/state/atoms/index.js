import { atom } from "recoil";

import { repositoriesQuery } from "../selectors";

export const catalogState = atom({
    key: "repositories",
    default: repositoriesQuery,
});

export const currentRepositoryState = atom({
    key: "currentRepository",
    default: "",
});

export const currentTagState = atom({
    key: "currentTag",
    default: "",
});

export const currentBlobDigestState = atom({
    key: "currentBlobDigest",
    default: "",
});
