const defaultRegistryURL: string = "https://trow.local:8443";

interface AppConfig {
    trow_registry_url: string;
    app: app;
}

interface app {
    authenticated: boolean;
}

const config: AppConfig = require("./config.json");

export default config;
