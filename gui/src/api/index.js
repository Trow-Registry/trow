import axios from "axios";
import config from "../../config";

const TROW_API = config.trow_registry_url;

const api = axios.create({
    baseURL: TROW_API,
    timeout: 1000,
});

export async function getCatalog() {
    try {
        const response = await api.get("/v2/_catalog");
        return response.data.repositories;
    } catch (error) {
        console.log(error);
    }
}

export async function getRepoTags({ repoName }) {
    try {
        const response = await api.get(`/v2/${repoName}/tags/list`);
        return response.data;
    } catch (error) {
        console.log(error);
    }
}

export async function getManifest({ repoName, reference }) {
    try {
        const response = await api.get(
            `/v2/${repoName}/manifests/${reference}`
        );
        return response.data;
    } catch (error) {
        console.log(error);
    }
}

export async function getBlob({ repoName, digest }) {
    try {
        const response = await api.get(`/v2/${repoName}/blobs/${digest}`);
        return response.data;
    } catch (error) {
        console.log(error);
    }
}
