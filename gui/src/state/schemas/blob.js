export const defaultBlobSchema = {
    architecture: "",
    author: "",
    os: "",
    created: "",
    docker_version: "",
    config: {
        Hostname: "",
        Domainname: "",
        User: "",
        AttachStdin: false,
        AttachStdout: false,
        AttachStderr: false,
        Tty: false,
        OpenStdin: false,
        StdinOnce: false,
        Env: [],
        Cmd: [],
        Image: "",
        Volumes: null,
        WorkingDir: "",
        Entrypoint: null,
        OnBuild: null,
        Labels: null,
    },
    container: "",
    container_config: {
        Hostname: "",
        Domainname: "",
        User: "",
        AttachStdin: false,
        AttachStdout: false,
        AttachStderr: false,
        Tty: false,
        OpenStdin: false,
        StdinOnce: false,
        Env: [],
        Cmd: [],
        Image: "",
        Volumes: null,
        WorkingDir: "",
        Entrypoint: null,
        OnBuild: null,
        Labels: null,
    },
    history: [],
    rootfs: {
        type: "",
        diff_ids: [],
    },
};
