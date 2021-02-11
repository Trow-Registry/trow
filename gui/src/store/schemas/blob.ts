export interface Blob {
    architecture: string;
    author?: string;
    os: string;
    created?: string;
    config?: config;
    container: string;
    container_config?: config;
    history?: blobHistory[];
    rootfs: blobRootFS;
}

interface blobHistory {
    created?: string;
    author?: string;
    created_by?: string;
    comment?: string;
    empty_layer?: boolean;
}

interface config {
    Hostname?: string;
    Domainname?: string;
    User?: string;
    AttachStdin?: boolean;
    AttachStdout?: boolean;
    AttachStderr?: boolean;
    Tty?: boolean;
    OpenStdin?: boolean;
    StdinOnce?: boolean;
    Env?: string[];
    Cmd?: string[];
    Image?: string;
    Volumes?: blobConfigVolumes;
    WorkingDir?: string;
    Entrypoint?: string[];
    ExposedPorts?: configExposedPorts;
    StopSignal?: string;
    Labels: configLabels;
}

interface configLabels {
    [key: string]: string;
}

interface blobConfigVolumes {
    [key: string]: {};
}

interface configExposedPorts {
    [key: string]: {};
}

interface blobRootFS {
    diff_ids: string[];
    type: string;
}

export const defaultBlobSchema: Blob = {
    os: "",
    architecture: "",
    rootfs: {
        type: "",
        diff_ids: [],
    },
    container: "",
};
