export interface Manifest {
    schemaVersion: number;
    config: descriptor;
    layers: descriptor[];
    mediaType: string;
    annotations?: annotations;
}

interface descriptor {
    mediaType: string;
    digest: string;
    size: string;
    urls?: string[];
    annotations?: annotations;
}

interface annotations {
    [key: string]: string;
}

export const defaultManifestSchema: Manifest = {
    schemaVersion: 2,
    config: {
        mediaType: "",
        digest: "",
        size: "",
    },
    layers: [],
    mediaType: "",
};
