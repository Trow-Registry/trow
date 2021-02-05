export interface Manifest {
    schemaVersion: string;
    config: {
        mediaType: string;
        digest: string;
        size: string;
    };
    layers: [];
    mediaType: string;
    annotations: {
        [key: string]: annotations;
    };
}

interface annotations {}
