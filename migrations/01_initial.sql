CREATE TABLE "blob" (
    "digest" TEXT NOT NULL PRIMARY KEY,
    "size" INTEGER NOT NULL,
    "last_accessed" INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;
CREATE TABLE "manifest" (
    "digest" TEXT NOT NULL PRIMARY KEY,
    -- "size" INTEGER NOT NULL,
    "last_accessed" INTEGER NOT NULL DEFAULT (unixepoch()),
    "content" BLOB NOT NULL
) STRICT;
CREATE TABLE "blob_blob_association" (
    "parent_digest" TEXT NOT NULL,
    "parent_is_manifest" INTEGER NOT NULL,
    "child_digest" TEXT NOT NULL,
    PRIMARY KEY ("parent_digest", "child_digest")
    -- FOREIGN KEY ("parent_digest")
    --     REFERENCES "blob" ("digest")
    --     ON DELETE CASCADE
    --     ON UPDATE CASCADE,
    -- FOREIGN KEY ("child_digest")
    --     REFERENCES "blob" ("digest")
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
