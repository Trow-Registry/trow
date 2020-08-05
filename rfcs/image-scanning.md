# Specification for container image scanning

## Abstract
This document describes the introduction of container images scanning to Trow thus providing additional security to a container based infrastructures.    
Container image scanning is currently supported by all the major container registries.  
The implementation is not specific to any image scanner but instead it leverages existing solutions.  
The specifications Trow follows are the same as the [Pluggable Image Vulnerability Scanning](https://github.com/goharbor/community/blob/master/proposals/pluggable-image-vulnerability-scanning_proposal.md)  
This will allow existing plugins to work for both Harbor and Trow.

## Non Goals
The document does not cover how to implement a container image scanner.  
It just leverages existing solutions, like Trivy or Clair.  

## Table of contents
1. Objectives
2. Specifications  
a. Components  
b. Architecture  


## 1. Objectives
* [ ] Security scanners integrate via the [Pluggable Image Vulnerability Scanning](https://github.com/goharbor/community/blob/master/proposals/pluggable-image-vulnerability-scanning_proposal.md) 
* [ ] Support scanning of OCI images and artifacts
* [ ] Execute a scan automatically when an image is pushed to the registry
* [ ] Store the result of the security scan for auditing purposes as a form of Artifact (ORCAS)
* [ ] Expose the result of the security scan via a dedicated Trow API to enable third party integrations or Trow web UI to consume the vulnerabilities data
* [ ] If the security scanning fails or is not configured properly, Trow **runtime** functionalities are not impacted
* [ ] **Deployment** of Trow should not be impacted by the availability of image scanners like Clair or Trivy
* [ ] Secure by default. Whenever a security scanner is available all images are automatically scanned for vulnerabilities. 
* [ ] New vulnerabilities are released every day. Trow should schedule image security scanning on a daily basis. Admins can configure the time.
* [ ] Allow to run a scan on demand by a user triggered action (api call and/or web ui) 
* [ ] Expose a webhook to subscribe to notifications regarding vulnerable images
* [ ] API endpoint exposing information about vulnerable images MUST be protected by default by authentication and authorisation methods
* [ ] Trow admission controller MUST support blocking the pulling of images in case they have SEVERE vulnerabilities. This can be disabled on a image level or namespace level.

---

## 2. Specifications
This section describes the implementation specs necessary to achieve the Objectives.  

### a. Components
This is a breakdown of all components and their descriptions.

#### Trow Security Scanner registry

**Objectives Met**
* Security scanners integration via the [Pluggable Image Vulnerability Scanning](https://github.com/goharbor/community/blob/master/proposals/pluggable-image-vulnerability-scanning_proposal.md)
* Support scanning of OCI images and artifacts: (based on security scanner capabilities)
* If the security scanning fails or is not configured properly, Trow **runtime** functionalities are not impacted
* **Deployment** of Trow should not be impacted by the availability of image scanners like Clair or Trivy

This component holds a list of all the available/configured security scanners that Trow can use to scan OCI images.    
The registry MUST store a list of Trow Security Scanner registry records.  

The registry can be an in memory MAP made of:
- KEY: security scanner URL
- VALUE: security scanner record

Map usually do not keep the order of elements, so the security scanner record should contain an `order` field with the priority: `0` is the **highest** priority.  

The registry needs to support the following operations:

- has_entries() -> bool: returns whether there are entries in the map. 
- get() -> `SecurityScannerRecord`: selects the security scanner with the highest priority which support scanning of OCI images and its API is up and running and it's enabled
- add(url: string, bearer_token: string, skip_cert_verify: bool, enabled: bool, order: integer): adds a new `SecurityScannerRecord` to the registry (map) and contacts it to check whether it supports scanning of OCI images and starts a periodic health check and capabilities and new version retrieval every `N` minutes. `N` should be configurable.  
- delete(url: String) -> bool: removes an existing `SecurityScannerRecord` and removes also the scheduling for the health check.
- update(url: string, bearer_token: string, skip_cert_verify: bool, enabled: bool, order: integer): updates an existing `SecurityScannerRecord`

A `SecurityScannerRecord` MUST contain the following information:

```
{
  "scanner": {
    "scanner_url": "http://scanner-adapter:8080/api/v1/metadata",
    "status": 1, // 0: Offline, 1: Online
    "last_health_check": "RFC 3339 date",
    "name": "Microscanner",
    "vendor": "Aqua Security",
    "version": "3.0.5",
    "beare_token": "",
    "skip_cert_verify": bool,
    "enabled": bool,
    "order": 0,
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

#### Scanner Adapter API

**Objectives Met**
* Security scanners integration via the [Pluggable Image Vulnerability Scanning](https://github.com/goharbor/community/blob/master/proposals/pluggable-image-vulnerability-scanning_proposal.md)
* Support scanning of OCI images and artifacts: (based on security scanner capabilities)
  
This is the only interface which allows communication between Trow and the security scanners.  
Trow acts as a Scanner Adapter API client. Trow is responsible to initiate the HTTP calls to the Scanner Adapter API.  
In other words `We call them they don't call us`.

  
1. Security Scanner Capabilities:
```
curl -H 'Accept: application/vnd.scanner.adapter.metadata+json; version=1.0" \
  http://scanner-adapter:8080/api/v1/metadata

Content-Type: application/vnd.scanner.adapter.scanner.metadata+json; version=1.0
Status: 200 OK

{
  "scanner": {
    "name": "Microscanner",
    "vendor": "Aqua Security",
    "version": "3.0.5",
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
```
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

{
  "error": {
    "message": "invalid registry_url"
  }
}
```

3. Submit a **VALID** scan request:
```
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

{
  "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6"
}
```

4. Retrieve the vulnerabilities report from the Security Scanner API:
```
curl -H 'Accept: application/vnd.scanner.adapter.vuln.report.harbor+json; version=1.0' \
  http://scanner-adapter:8080/api/v1/scan/3fa85f64-5717-4562-b3fc-2c963f66afa6/report

Retry-After: 15 // This is an HTTP response header
Status: 302 Found
```

5. Wait 15 seconds or use your own retry interval ... 

6. When you successfully retrieve the report:
**IMPORTANT**: The format of this JSON response is a Harbor compatible one. We will stick to the same format, so that Trow can support different security scanners.  
There is however the possibility to get also the security scanner specific JSON response. See point `7.`
```
curl -H 'Accept: application/vnd.scanner.adapter.vuln.report.harbor+json; version=1.0' \
  http://scanner-adapter:8080/api/v1/scan/3fa85f64-5717-4562-b3fc-2c963f66afa6/report

Content-Type: application/vnd.scanner.adapter.vuln.report.harbor+json; version=1.0
Status: 200 OK

{
  "generated_at": "2019-08-07T12:17:21.854Z",
  "artifact": {
    "repository": "library/mongo",
    "digest": "sha256:917f5b7f4bef1b35ee90f03033f33a81002511c1e0767fd44276d4bd9cd2fa8e"
  },
  "scanner": {
    "name": "Microscanner",
    "vendor": "Aqua Security",
    "version": "3.0.5",
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
    },
    ...
  ]
}

```

7. Proprietary vulnerability report (with an example report generated by MicroScanner in JSON format):  
```
curl -H 'Accept: application/vnd.scanner.adapter.vuln.report.raw' \
   http://scanner-adapter:8080/api/v1/scan/3fa85f64-5717-4562-b3fc-2c963f66afa6/report

Content-Type: application/vnd.scanner.adapter.vuln.report.raw
Status: 200 OK

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
```
Accept: application/vnd.anchore.policy.report+json; version=0.3
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

---

TO BE CONTINUED...