# Run Trow

```bash
# Setup registries config
## nvcr
mkdir -p proxy-registry-config/nvcr;echo '{"auths":{"nvcr.io":{}}}'|jq>proxy-registry-config/nvcr/.dockerconfigjson
## quay
mkdir -p proxy-registry-config/quay;echo '{"auths":{"quay.io":{}}}'|jq>proxy-registry-config/quay/.dockerconfigjson
## ECR (requires aws creds)
mkdir -p proxy-registry-config/ecr;echo '{"auths":{"xxxxxx.dkr.ecr.eu-west-2.amazonaws.com":{"username": "AWS", "password": "eu-west-2"}}}'|jq>proxy-registry-config/ecr/.dockerconfigjson

# run debug version with cargo
RUST_LOG=debug cargo run -- --no-tls --proxy-docker-hub --allow-docker-official --allow-prefixes docker.io --port 9999 --proxy-registry-config-dir ./proxy-registry-config
# run the container
docker run -it --rm -p 9999:9999 -v $PWD/proxy-registry-config:/conf awoimbee/trow:13-09-2021-0 --no-tls --proxy-docker-hub --allow-docker-official --allow-prefixes docker.io --port 9999 --proxy-registry-config-dir /conf
```

# Dump & analyse every http request that the docker daemon does
## Experiment

```bash
# Generate and trust a ca cert
docker run --rm -it -v ~/.mitmproxy:$PWD/mitmproxy -p 8080:8080 mitmproxy/mitmproxy
sudo cp ./mitmproxy/mitmproxy-ca-cert.cer /etc/ca-certificates/trust-source/anchors/
sudo update-ca-trust

# setup proxy for docker deamon
sudo mkdir -p /etc/systemd/system/docker.service.d
sudo sh -c 'printf "[Service]\nEnvironment="HTTP_PROXY=http://localhost:8080"\nEnvironment="HTTPS_PROXY=http://localhost:8080"\n" > "/etc/systemd/system/docker.service.d/http-proxy.conf"'

sudo systemctl daemon-reload
sudo systemctl restart docker

# Run the http proxy (web edition)
docker run --rm -it -p 8080:8080 -p 127.0.0.1:8081:8081 -v $PWD/mitmproxy:/home/mitmproxy/.mitmproxy mitmproxy/mitmproxy mitmweb --web-host 0.0.0.0

# docker pull (...)

sudo rm /etc/ca-certificates/trust-source/anchors/mitmproxy-ca-cert.cer
sudo update-ca-trust
sudo rm /etc/systemd/system/docker.service.d/http-proxy.conf
sudo systemctl daemon-reload
sudo systemctl restart docker
```

## Conslusion

- Bearer auth (e.g. dockerhub):
  - GET https://registry.xyz/v2
    => HTTP 401: `Bearer realm="https://auth.registry.xyz/token",service="registry.company.xyz"`
  - GET ${realm}?account=$username&scope=???
    => HTTP 200: `token | access_token`
  - HEAD https://registry.xyz/v2/$image/manifests/$tag
    => HTTP 200
- Basic auth (e.g. ECR):
  - GET https://registry.xyz/v2
    => HTTP 401: `Basic realm="https://registry.xyz/",service="ecr.amazonaws.com"`
  - HEAD https://registry.xyz/v2/$image/manifests/$tag
    Headers: `Authorization: Basic {b64 creds}`
    => HTTP 200
