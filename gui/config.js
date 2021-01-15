const defaultRegistryURL = "https://trow.local:8443";
const trowRegistryURL = process.env.TROW_REGISTRY_URL
    ? process.env.TROW_REGISTRY_URL
    : defaultRegistryURL;

const config = {
    trow_registry_url: trowRegistryURL,
    app: {
        authenticated: false,
    },
};

export default config;
