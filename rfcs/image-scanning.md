# Specification for container image scanning

## Abstract
This document describes the introduction of container images scanning to Trow thus providing additional security 
to a container based infrastructures.    
Container image scanning is currently supported by all the major container registries.  
The implementation is not specific to any image scanner but instead it leverages existing solutions.  
The specifications Trow follows are the same as the 
[Pluggable Image Vulnerability Scanning](https://github.com/goharbor/community/blob/master/proposals/pluggable-image-vulnerability-scanning_proposal.md)  
This will allow existing plugins to work for both Harbor and Trow.

## Non Goals
* The document does not cover how to implement a container image scanner.    
It just leverages existing solutions, like Trivy or Clair.  
* The document does not cover the possibility to run multiple security scanners and combine their output, yet.  
If there is an interest in such a feature, please open an issue in the repository.   


## Table of contents
1. Objectives
2. Specifications  
a. Components  
b. Architecture
3. Future improvements  


## 1. Objectives
* [ ] Security scanners integrate via the [Pluggable Image Vulnerability Scanning](https://github.com/goharbor/community/blob/master/proposals/pluggable-image-vulnerability-scanning_proposal.md) 
* [ ] Support scanning of OCI images and artifacts
* [ ] Execute a scan automatically when an image is pushed to the registry
* [ ] Store the result of the security scan for auditing purposes as a form of Artifact (ORAS client can consume the reports)
* [ ] Expose the result of the security scan via a dedicated Trow API to enable third party integrations or 
Trow web UI to consume the vulnerabilities data
* [ ] If the security scanning fails or is not configured properly, Trow **runtime** functionalities are not impacted
* [ ] **Deployment** of Trow should not be impacted by the availability of image scanners like Clair or Trivy
* [ ] Secure by default. Whenever a security scanner is available all images are automatically scanned for vulnerabilities. 
* [ ] New vulnerabilities are released every day. Trow should schedule image security scanning on a daily basis. 
Admins can configure the scheduling period.
* [ ] Allow to run a scan on demand by a user triggered action (api call and/or web ui) 
* [ ] Expose a webhook to subscribe to notifications regarding vulnerable images
* [ ] API endpoint exposing information about vulnerable images MUST be protected by default by authentication and authorisation methods
* [ ] Trow admission controller MUST support blocking the pulling of images in case they have SEVERE vulnerabilities. 
This can be disabled on an image level or namespace level.

---

## 2. Specifications
This section describes the implementation specs necessary to achieve the Objectives.  

### a. Components
This is a breakdown of all components and their descriptions.

#### Trow Security Scanner registry
This component holds a list of all the available/configured security scanners that Trow can use to scan OCI images.    
The registry MUST store a list of Trow Security Scanner registry records.  

The registry can be an in memory MAP made of:
- KEY: security scanner URL
- VALUE: security scanner record

Map usually do not keep the order of elements, so the security scanner record should contain an `order` field 
with the priority: `0` is the **highest** priority.  

The registry needs to support the following operations:

- has_entries() -> bool: returns whether there are entries in the map. 
- get() -> `SecurityScannerRecord`: selects the security scanner with the highest priority which support scanning of 
OCI images and its API is up and running and it's enabled
- add(url: string, bearer_token: string, skip_cert_verify: bool, enabled: bool, order: integer): adds a new `
SecurityScannerRecord` to the registry (map) and contacts it to check whether it supports scanning of OCI images 
and starts a periodic health check and capabilities and new version retrieval every `N` minutes. `N` should be configurable.  
- delete(url: String) -> bool: removes an existing `SecurityScannerRecord` and removes also the scheduling for the health check.
- update(url: string, bearer_token: string, skip_cert_verify: bool, enabled: bool, order: integer): updates an existing `SecurityScannerRecord`

A `SecurityScannerRecord` MUST contain the following information:

```rust
enum ScannerMimeTypes {
  OCI_MANIFEST(String),
  DOCKER_MANIFEST(String),
  REPORT_HARBOR(String),
  REPORT_RAW(String)
}

enum ScannerAdapterStatus {
  OFFLINE = 0,
  ONLINE = 1
}

// This is the security scanner record/entry or as defined in the RFC: SecurityScannerRecord
pub struct SecurityScanner {
  // stores the URL for the external security scanner: "http://scanner-adapter:8080"
  scanner_url: String,
  // stores the status of the external security scanner, by checking "http://scanner-adapter:8080/api/v1/metadata"
  status: ScannerAdapterStatus,
  // stores the last time the health check was executed
  last_health_check: chrono::DateTime, // rfc3339
  // this is the bearer token for authenticating with the external security scanner API
  bearer_token: String,
  // whether or not the TLS certificate should be validated
  skip_cert_verify: bool,
  // whether this specific external security scanner is enabled
  enabled: bool,
  // the order of this security scanner
  order: i8,
  // the name of the scanner: "Microscanner"
  name: String,
  // The name od the vendor: "Aqua Security"  
  vendor: String,
  // the version of the scanner: 3.0.5
  version: String,
  // Supported scanner capabilities
  capabilities: Vec<Capabilities>,
  // Supported scanner properties
  properties: HashMap,
  // This is the client which interacts with the HTTP API exposed by the security scanners
  security_scanner_client: SecurityScannerClient
}
struct SecurityScannerClient {
  http_client: reqwest::Client
}


struct Capabilities {
   consumes_mime_types: Vec<ScannerMimeTypes>,
   produces_mime_types: Vec<ScannerMimeTypes>,
}
```

The `/metadata` endpoint returns a set of properties which we need to parse into the `SecurityScannerRecord`
```json
{
  "scanner": {        
    "name": "Microscanner",
    "vendor": "Aqua Security",
    "version": "3.0.5"    
  },
  "capabilities": [
    {
      "consumes_mime_types": [
        "application/vnd.oci.image.manifest.v1+json",
        "application/vnd.docker.distribution.manifest.v2+json"
      ],
      "produces_mime_types": [
        "application/vnd.scanner.adapter.vuln.report.harbor+json; version=1.0",
        "application/vnd.scanner.adapter.vuln.report.raw"
      ]
    }
  ],
  "properties": {
    "harbor.scanner-adapter/scanner-type": "os-package-vulnerability",
    "harbor.scanner-adapter/vulnerability-database-updated-at": "2019-08-13T08:16:33.345Z"
  }
}
```

Trow Security Scanner Registry MUST:
- [ ] periodically contact all the configured security scanners to check whether new versions were deployed and capabilities were added or removed
- [ ] periodically contact all the configured security scanners to check whether their API is up and running and store the status in the record.
- [ ] be contacted by Trow Security Scanner Controller to check whether there at least 1 available security scanner which is on-line and which supports scanning of OCI images 
- [ ] Log the status of a security scanner when it comes online or goes offline

Having a period health check does not guarantee that the security scanner is always going to be available when Trow starts the actual image scanning.  
Since we need to periodically check whether new versions of security scanners have been deployed, we can also keep the status of the security scanner in memory, so in case it is down we don't have to keep contacting it, until it is back.

**IMPORTANT:** 
* make sure the capabilities that the security scanner advertise allow scanning OCI images.
* The health check API endpoint is: `/api/v1/metadata`
* if 2 `SecurityScannerRecord` have the same priority order them alphabetically by their name.

#### Trow Security Scanner Client
This component is the one responsible to interact with the security scanners HTTP api.  
In this section there is a summary of all the HTTP API calls the security scanner client need to implement in order to interact with the external security scanners (Clair, Trivy).  
Trow security scanner registry and Trow security scanner controller (detailed below, after the http client spec) leverage the Trow security scanner Client to execute security scanners operations detailed below.  
This means that there should be an instance of this client for each entry of the Trow security scanner record `SecurityScannerRecord`
  
This is the only interface which allows communication between Trow and the security scanners.  
Trow acts as a Scanner Adapter API client. Trow is responsible to initiate the HTTP calls to the Scanner Adapter API.  
In other words `We call them they don't call us`.

  
1. Security Scanner Capabilities:
```bash
curl -H 'Accept: application/vnd.scanner.adapter.metadata+json; version=1.0" \
  http://scanner-adapter:8080/api/v1/metadata

Content-Type: application/vnd.scanner.adapter.scanner.metadata+json; version=1.0
Status: 200 OK
```
```json
{
  "scanner": {
    "name": "Microscanner",
    "vendor": "Aqua Security",
    "version": "3.0.5"
  },
  "capabilities": [
    {
      "consumes_mime_types": [
        "application/vnd.oci.image.manifest.v1+json",
        "application/vnd.docker.distribution.manifest.v2+json"
      ],
      "produces_mime_types": [
        "application/vnd.scanner.adapter.vuln.report.harbor+json; version=1.0",
        "application/vnd.scanner.adapter.vuln.report.raw"
      ]
    }
  ],
  "properties": {
    "harbor.scanner-adapter/scanner-type": "os-package-vulnerability",
    "harbor.scanner-adapter/vulnerability-database-updated-at": "2019-08-13T08:16:33.345Z"
  }
}
```

2. Submit an **INVALID** scan request:
```bash
curl http://scanner-adapter:8080/api/v1/scan \
-H 'Content-Type: application/vnd.scanner.adapter.scan.request+json; version=1.0' \
-d @- << EOF
{
  "registry": {
    "url": "INVALID_REGISTRY_URL",
    "authorization": "Bearer JWTTOKENGOESHERE"
  },
  "artifact": {
    "repository": "library/mongo",
    "digest": "sha256:917f5b7f4bef1b35ee90f03033f33a81002511c1e0767fd44276d4bd9cd2fa8e"
  }
}
EOF

Status: 422 Unprocessable Entity
Content-Type: application/vnd.scanner.adapter.error+json; version=1.0'
```
```json
{
  "error": {
    "message": "invalid registry_url"
  }
}
```

3. Submit a **VALID** scan request:
```bash
curl http://scanner-adapter:8080/api/v1/scan \
-H 'Content-Type: application/vnd.scanner.adapter.scan.request+json; version=1.0' \
-d @- << EOF
{
  "registry": {
    "url": "harbor-harbor-registry:5000",
    "authorization": "Bearer: JWTTOKENGOESHERE"
  },
  "artifact": {
    "repository": "library/mongo",
    "digest": "sha256:917f5b7f4bef1b35ee90f03033f33a81002511c1e0767fd44276d4bd9cd2fa8e"
  }
}
EOF

Status: 202 Accepted
Content-Type: application/vnd.scanner.adapter.scan.response+json; version=1.0'
```
```json
{
  "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6"
}
```

4. Retrieve the vulnerabilities report from the Security Scanner API:
```bash
curl -H 'Accept: application/vnd.scanner.adapter.vuln.report.harbor+json; version=1.0' \
  http://scanner-adapter:8080/api/v1/scan/3fa85f64-5717-4562-b3fc-2c963f66afa6/report

Retry-After: 15 // This is an HTTP response header
Status: 302 Found
```

5. Wait 15 seconds or use your own retry interval ... 

6. When you successfully retrieve the report:
**IMPORTANT**: The format of this JSON response is a Harbor compatible one. We will stick to the same format, 
so that Trow can support different security scanners.  
There is however the possibility to get also the security scanner specific JSON response. See point `7.`
```bash
curl -H 'Accept: application/vnd.scanner.adapter.vuln.report.harbor+json; version=1.0' \
  http://scanner-adapter:8080/api/v1/scan/3fa85f64-5717-4562-b3fc-2c963f66afa6/report

Content-Type: application/vnd.scanner.adapter.vuln.report.harbor+json; version=1.0
Status: 200 OK
```
```json
{
  "generated_at": "2019-08-07T12:17:21.854Z",
  "artifact": {
    "repository": "library/mongo",
    "digest": "sha256:917f5b7f4bef1b35ee90f03033f33a81002511c1e0767fd44276d4bd9cd2fa8e"
  },
  "scanner": {
    "name": "Microscanner",
    "vendor": "Aqua Security",
    "version": "3.0.5"
  },
  "severity": "High",
  "vulnerabilities": [
    {
      "id": "CVE-2017-8283",
      "package": "dpkg",
      "version": "1.17.27",
      "fix_version": "1.18.0",
      "severity": "High",
      "description": "...",
      "links": [
        "https://security-tracker.debian.org/tracker/CVE-2017-8283"
      ]
    }
  ]
}

```

7. Proprietary vulnerability report (with an example report generated by MicroScanner in JSON format):  
```bash
curl -H 'Accept: application/vnd.scanner.adapter.vuln.report.raw' \
   http://scanner-adapter:8080/api/v1/scan/3fa85f64-5717-4562-b3fc-2c963f66afa6/report

Content-Type: application/vnd.scanner.adapter.vuln.report.raw
Status: 200 OK
```
```json
{
  "scan_started": {
    "seconds": 1561386673,
    "nanos": 390482870
  },
  "scan_duration": 2,
  "digest": "b3c8bc6c39af8e8f18f5caf53eec3c6c4af60a1332d1736a0cd03e710388e9c8",
  "os": "debian",
  "version": "8",
  "resources": [
    {
      "resource": {
        "format": "deb",
        "name": "apt",
        "version": "1.0.9.8.5",
        "arch": "amd64",
        "cpe": "pkg:/debian:8:apt:1.0.9.8.5",
        "name_hash": "583f72a833c7dfd63c03edba3776247a"
      },
      "scanned": true,
      "vulnerabilities": [
        {
          "name": "CVE-2011-3374",
          "vendor_score_version": "Aqua",
          "vendor_severity": "negligible",
          "vendor_statement": "Not exploitable in Debian, since no keyring URI is defined",
          "vendor_url": "https://security-tracker.debian.org/tracker/CVE-2011-3374",
          "classification": "..."
        }
      ]
    }
  ]
}
```

**IMPORTANT** Request specifications:
* The Accept request header is required to indicate to Scanner Adapter an intended scan report format
* If the client does not specify the Accept header it's assumed to be Harbor vulnerability report with the MIME type application/vnd.scanner.adapter.vuln.report.harbor+json; version=1.0.
* New scan report MIME types might be introduced without breaking the backward compatibility of the API and introducing new URL paths to the Scanner Adapter API spec. Example:
```bash
Accept: application/vnd.anchore.policy.report+json; version=0.3
```
```json
[
  {
    "sha256:57334c50959f26ce1ee025d08f136c2292c128f84e7b229d1b0da5dac89e9866": {
      "docker.io/alpine:latest": [
        {
          "detail": {},
          "last_evaluation": "2019-08-07T06:33:48Z",
          "policyId": "2c53a13c-1765-11e8-82ef-23527761d060",
          "status": "pass"
        }
      ]
    }
  }
]
```

#### Trow Security Scanner Controller
Trow security scanner controller is the component responsible for querying Trow security scanner registry about the availability of security scanners.  
When the scanner is available, it will be the interface for all the following points:
1. Start a security scan
2. Poll for the security scan result
3. Store the security scan result as an Artifact in Trow registry
4. Scheduling of periodic security scans of the OCI images and artifacts stored in Trow
5. Retrieving existing scan reports for specific images and artifacts in a performant way
6. Emit events when an image is found to be vulnerable. (Necessary to expose these events via webhook)

Every time a security scanning operation needs to be triggered, the Scanner Controller will check for the presence of a Trow Security Scanner.  
This will allow to detect the presence of new security scanners, or the fact that a security scanner might not be available at the moment.

Most of the above operations are quite straightforward, however two of them will require more details: 
1. emitting events when an image is vulnerable
2. storing the scan results as an artifact in trow

For the first point, we can use a `std::sync::mpsc::channel`. In this way the receiver (the webhook component) 
is notified about new events and can call the external end points.    

For the second point, the scan report storage, the idea is to leverage the existing OCI specifications for storing artifacts, 
however there is no `mime type` defined, at the moment of writing, regarding security vulnerabilities.  
Looking at other projects, like [Azure acr registry](https://github.com/Azure/acr/blob/5da11319e5f8e202ad5fd78a890cf04bde60f031/docs/artifact-media-types.json) or
the [Open Policy Agent](https://github.com/open-policy-agent/opa/issues/1413), they seem to define their own mime types.  
As of now, however, none are registered with [IANA](https://www.iana.org/assignments/media-types/media-types.xhtml) 
OCI make a similar example with image vulnerability scanning and [suggest](https://github.com/opencontainers/artifacts/blob/master/artifact-authors.md#defining-a-unique-artifact-type)
to register new mime types.   

Here an example of the work needed: [application/vnd.oci.image.manifest.v1+json](https://www.iana.org/assignments/media-types/application/vnd.oci.image.manifest.v1+json)

The mime type we propose is the following:

* `application/vnd.oci.vuln.report.v1+[json|yaml]`:  This is the raw json scan result as defined in the `application/vnd.scanner.adapter.vuln.report.harbor+json; version=1.0`

`application/vnd.oci.vuln.report.v1+[json|yaml]` Example:
```json
{
  "generated_at": "2019-08-07T12:17:21.854Z",
  "artifact": {
    "repository": "library/mongo",
    "digest": "sha256:917f5b7f4bef1b35ee90f03033f33a81002511c1e0767fd44276d4bd9cd2fa8e"
  },
  "scanner": {
    "name": "Microscanner",
    "vendor": "Aqua Security",
    "version": "3.0.5"
  },
  "severity": "High",
  "vulnerabilities": [
    {
      "id": "CVE-2017-8283",
      "package": "dpkg",
      "version": "1.17.27",
      "fix_version": "1.18.0",
      "severity": "High",
      "description": "...",
      "links": [
        "https://security-tracker.debian.org/tracker/CVE-2017-8283"
      ]
    }
  ]
}
```

If a client is interested in downloading/pulling the vulnerability report as an artifact, then the recommendation, at this stage is to use ORAS client.  
ORAS client, in fact, supports extending the mime types via [config](https://github.com/deislabs/oras/pull/56)  
Open Policy Agent also [opted](https://github.com/open-policy-agent/opa/issues/1413) for the definition of well-known mime types:

Given the above assumptions regarding mime types, we can proceed implementing the storage of the vulnerabilities report as follow:

* Step 1: Store the security scanning report JSON response as json/yaml `application/vnd.oci.vuln.report.v1+[json|yaml]` file and calculate its sha256 hash
* Step 2: Write the following `manifest.json`:  
```json

{
  "schemaVersion": 2,
  "config": {
    "mediaType": "vnd.unknown.config.v1+json"
  },
  "layers": [
    {
      "mediaType": "application/vnd.oci.vuln.report.v1+json",
      "size": 32654,
      "digest": "sha256:9834876dcfb05cb167a5c24953eba58c4ac89b1adf57f28f2f9d09af107ee8f0"
    }    
  ]
}
```
We do not need a config manifest for the vulnerability scanning report, so the mediaType is set to: `vnd.unknown.config.v1+json`  
See Azure container [documentation](https://docs.microsoft.com/en-us/azure/container-registry/container-registry-oci-artifacts)  

* Step 3: Add an entry to the `index` file of the image which was scanned.  
This will also serve as a quick way to check whether an image/artifact was already scanned 
(there is no info about the date and time of the scanning though, so we will need to lookup the artifact as well in case it's needed).    
Example:
```json
{
  "schemaVersion": 2,
  "manifests": [
    {
      "mediaType": "application/vnd.oci.image.manifest.v1+json",
      "size": 7143,
      "digest": "sha256:e692418e4cbaf90ca69d05a66403747baa33ee08806650b51fab815ad7fc331f",
      "platform": {
        "architecture": "ppc64le",
        "os": "linux"
      }
    },
    {
      "mediaType": "application/vnd.oci.vuln.meta.v1+json",
      "size": 7143,
      "digest": "sha256:e692418e4cbaf90ca69d05a66403747baa33ee08806650b51fab815ad7fc331f",
      "platform": {
        "architecture": "ppc64le",
        "os": "linux"
      }
    },
    {
      "mediaType": "application/vnd.oci.image.manifest.v1+json",
      "size": 7682,
      "digest": "sha256:5b0bcabd1ed22e9fb1310cf6c2dec7cdef19f0ad69efa1f392e94a4333501270",
      "platform": {
        "architecture": "amd64",
        "os": "linux"
      }
    },
    {
      "mediaType": "application/vnd.oci.vuln.meta.v1+json",
      "size": 7682,
      "digest": "sha256:5b0bcabd1ed22e9fb1310cf6c2dec7cdef19f0ad69efa1f392e94a4333501270",
      "platform": {
        "architecture": "amd64",
        "os": "linux"
      }
    }    
  ],
  "annotations": {
    "com.example.key1": "value1",
    "com.example.key2": "value2"
  }
}
``` 

#### Trow WebHook
Once an image is found to have vulnerabilities we can allow pushing this information out to third party HTTP/s end points.  
In this section we define the format of these events and the authentication method.  

In order to keep things simple at this stage the payload will be the same as the mime type: `application/vnd.oci.vuln.report.v1+[json|yaml]`  
```json
{
  "generated_at": "2019-08-07T12:17:21.854Z",
  "artifact": {
    "repository": "library/mongo",
    "digest": "sha256:917f5b7f4bef1b35ee90f03033f33a81002511c1e0767fd44276d4bd9cd2fa8e"
  },
  "scanner": {
    "name": "Microscanner",
    "vendor": "Aqua Security",
    "version": "3.0.5"
  },
  "severity": "High",
  "vulnerabilities": [
    {
      "id": "CVE-2017-8283",
      "package": "dpkg",
      "version": "1.17.27",
      "fix_version": "1.18.0",
      "severity": "High",
      "description": "...",
      "links": [
        "https://security-tracker.debian.org/tracker/CVE-2017-8283"
      ]
    }
  ]
}
```

The authentication method uses a secret key `S` which is configured for each webhook to sign the payload which is sent to the webhook client.  
The signature is generate via HMAC(`S`, json payload).  
The signature value is then hex encoded and send to the webhook client in an http header called: `X-Trow-Signature`  
---

### Architecture



---

#### Future potential improvements

The usage of HMAC for signing files could be an interesting security feature of Trow.  
HMAC in fact guarantees that nobody has tempered with the local files, as long as the attacker gains access to the secret key.  
The secret key would need to be loaded externally from either Vault or a key management system or a secure enclave (Intel SGX ?).
To be discussed further. 