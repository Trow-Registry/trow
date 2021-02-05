export interface Blob {
    architecture: string;
    author: string;
    os: string;
    created: string;
    docker_version: string;
    config: {
        Hostname: string;
        Domainname: string;
        User: string;
        AttachStdin: boolean;
        AttachStdout: boolean;
        AttachStderr: boolean;
        Tty: boolean;
        OpenStdin: boolean;
        StdinOnce: boolean;
        Env: [];
        Cmd: [];
        Image: string;
        Volumes: null;
        WorkingDir: string;
        Entrypoint: null;
        OnBuild: null;
        Labels: null;
    };
    container: string;
    container_config: {
        Hostname: string;
        Domainname: string;
        User: string;
        AttachStdin: boolean;
        AttachStdout: boolean;
        AttachStderr: boolean;
        Tty: boolean;
        OpenStdin: boolean;
        StdinOnce: boolean;
        Env: [];
        Cmd: [];
        Image: string;
        Volumes: null;
        WorkingDir: string;
        Entrypoint: null;
        OnBuild: null;
        Labels: null;
    };
    history: [];
    rootfs: {
        type: string;
        diff_ids: [];
    };
}
