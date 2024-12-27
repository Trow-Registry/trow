CREATE TABLE "blob" (
    "digest" varchar NOT NULL PRIMARY KEY,
    "size" integer NOT NULL,
    "is_manifest" boolean NOT NULL,
    "last_accessed" integer NOT NULL DEFAULT (unixepoch())
);
CREATE TABLE "blob_blob_association" (
    "parent_digest" varchar NOT NULL,
    "child_digest" varchar NOT NULL,
    PRIMARY KEY ("parent_digest", "child_digest"),
    FOREIGN KEY ("parent_digest")
        REFERENCES "blob" ("digest")
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    FOREIGN KEY ("child_digest")
        REFERENCES "blob" ("digest")
);
CREATE TABLE "blob_upload" (
    "uuid" TEXT NOT NULL PRIMARY KEY,
    "offset" integer NOT NULL,
    "updated_at" timestamp_text NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "repo" varchar NOT NULL
);
CREATE TABLE "repo_blob_association" (
    "repo_name" varchar NOT NULL,
    "blob_digest" varchar NOT NULL,
    PRIMARY KEY ("repo_name", "blob_digest"),
    FOREIGN KEY ("blob_digest")
        REFERENCES "blob" ("digest")
        ON DELETE CASCADE
);
CREATE TABLE "tag" (
    "tag" varchar NOT NULL,
    "repo" varchar NOT NULL,
    "manifest_digest" varchar NOT NULL,
    CONSTRAINT "IDX_repo_tag" PRIMARY KEY ("repo", "tag"),
    FOREIGN KEY ("repo", "manifest_digest")
        REFERENCES "repo_blob_association" ("repo_name", "blob_digest")
        ON DELETE CASCADE
);
