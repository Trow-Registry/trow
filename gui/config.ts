const defaultRegistryURL: string = "https://trow.local:8443";

interface AppConfig {
    trow_registry_url: string;
    app: app
}

interface app {
    authenticated: boolean;
}

// const trowRegistryURL = process.env.TROW_REGISTRY_URL
//     ? process.env.TROW_REGISTRY_URL
//     : defaultRegistryURL;

// const config = {
//     trow_registry_url: trowRegistryURL,
//     app: {
//         authenticated: false,
//     },
// };

const config: AppConfig = require("./config.json");

export default config;
