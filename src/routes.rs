/*
Routes of a 2.0 Registry

Version Check of the registry
GET /v2/

# Responses
200 - We Exist (and you are authenticated)
401 - Please Authorize (WWW-Authenticate header with instuctions).

# Headers
Docker-Distribution-API-Version: registry/2.0

---
Pulling an image
GET /v2/<name>/manifests/<reference>

# Parameters
name - The name of the image
reference - either a tag or a digest

# Client Headers
Accept: manifest-version

# Headers
Accept: manifest-version
?Docker-Content-Digest: digest of manifest file

# Returns
200 - return the manifest
404 - manifest not known to the registry

---
Check for existance
HEAD /v2/<name>/manifests/<reference>

# Parameters
name - The name of the image
reference - either a tag or a digest

# Headers
Content-Length: size of manifest
?Docker-Content-Digest: digest of manifest file

# Returns
200 - manifest exists
404 - manifest does not exist

---
Pulling a Layer
GET /v2/<name>/blobs/<digest>
name - name of the repository
digest - unique identifier for the blob to be downoaded

# Responses
200 - blob is downloaded
307 - redirect to another service for downloading[1]

---
Pushing a Layer
POST /v2/<name>/blobs/uploads/
name - name of repository

# Headers
Location: /v2/<name>/blobs/uploads/<uuid>
Range: bytes=0-<offset>
Content-Length: 0
Docker-Upload-UUID: <uuid>

# Returns
202 - accepted

---
Check for existing layer
HEAD /v2/<name>/blobs/<digest>
name - name of repository
digest - digest of blob to be checked

# Headers
Content-Length: <length of blob>
Docker-Content-Digest: <digest>

# Returns
200 - exists
404 - does not exist

---
Upload Progress
GET /v2/<name>/blobs/uploads/<uuid>
name - name of registry
uuid - unique id for the upload that is to be checked

# Client Headers
Host: <registry host>

# Headers
Location: /v2/<name>/blobs/uploads/<uuid>
Range: bytes=0-<offset>
Docker-Upload-UUID: <uuid>

# Returns
204

---
Monolithic Upload
PUT /v2/<name>/blobs/uploads/<uuid>?digest=<digest>
Content-Length: <size of layer>
Content-Type: application/octet-stream

<Layer Binary Data>


---
Chunked Upload (Don't implement until Monolithic works)
PATCH /v2/<name>/blobs/uploads/<uuid>
Content-Length: <size of chunk>
Content-Range: <start of range>-<end of range>
Content-Type: application/octet-stream

<Layer Chunk Binary Data>

---
Cancelling an upload
DELETE /v2/<name>/blobs/uploads/<uuid>


---
Cross repo blob mounting (validate how regularly this is used)
POST /v2/<name>/blobs/uploads/?mount=<digest>&from=<repository name>

---
Delete a layer
DELETE /v2/<name>/blobs/<digest>

---
Pushing an image manifest
PUT /v2/<name>/manifests/<reference>
Content-Type: <manifest media type>

---
Listing Repositories
GET /v2/_catalog

---
Listing Image Tags
GET /v2/<name>/tags/list

---
Deleting an Image
DELETE /v2/<name>/manifests/<reference>

---
[1]: Could possibly be used to redirect a client to a local cache
*/
