CREATE TABLE "blob" (
    "digest" TEXT NOT NULL PRIMARY KEY,
    "size" INTEGER NOT NULL,
    "last_accessed" INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;
CREATE TABLE "manifest" (
    "digest" TEXT NOT NULL PRIMARY KEY,
    -- "size" INTEGER NOT NULL,
    "last_accessed" INTEGER NOT NULL DEFAULT (unixepoch()),
    "json" BLOB NOT NULL,
    "blob" BLOB NOT NULL
) STRICT;
CREATE TABLE "blob_upload" (
    "uuid" TEXT NOT NULL PRIMARY KEY,
    "offset" INTEGER NOT NULL,
    "updated_at" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "repo" TEXT NOT NULL
) STRICT;
CREATE TABLE "repo_blob_association" (
    "repo_name" TEXT NOT NULL,
    "blob_digest" TEXT NOT NULL,
    PRIMARY KEY ("repo_name", "blob_digest")
    -- FOREIGN KEY ("blob_digest")
    --     REFERENCES "blob" ("digest")
    --     ON DELETE CASCADE
) STRICT;
CREATE TABLE "tag" (
    "tag" TEXT NOT NULL,
    "repo" TEXT NOT NULL,
    "manifest_digest" TEXT NOT NULL,
    CONSTRAINT "IDX_repo_tag" PRIMARY KEY ("repo", "tag")
    -- FOREIGN KEY ("repo", "manifest_digest")
    --     REFERENCES "repo_blob_association" ("repo_name", "blob_digest")
    --     ON DELETE CASCADE
) STRICT;
