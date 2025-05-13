-- First add some FK I should have added from the beginning
CREATE TABLE repo_blob_assoc (
    "repo_name" TEXT NOT NULL,
    "blob_digest" TEXT,
    "manifest_digest" TEXT,
    PRIMARY KEY ("repo_name", "blob_digest", "manifest_digest"),
    FOREIGN KEY (blob_digest) REFERENCES blob(digest) ON DELETE CASCADE,
    FOREIGN KEY (manifest_digest) REFERENCES manifest(digest) ON DELETE CASCADE,
    CHECK (blob_digest IS NOT NULL OR manifest_digest IS NOT NULL)
);
INSERT INTO repo_blob_assoc (repo_name, blob_digest, manifest_digest)
SELECT
    rba.repo_name,
    CASE WHEN m.digest IS NULL THEN rba.blob_digest ELSE NULL END AS blob_digest,
    CASE WHEN m.digest IS NOT NULL THEN rba.blob_digest ELSE NULL END AS manifest_digest
FROM
    repo_blob_association rba
    LEFT JOIN manifest m ON rba.blob_digest = m.digest;
DROP TABLE repo_blob_association;

ALTER TABLE tag RENAME TO tag_old;
CREATE TABLE "tag" (
    "tag" TEXT NOT NULL,
    "repo" TEXT NOT NULL,
    "manifest_digest" TEXT NOT NULL,
    CONSTRAINT "IDX_repo_tag" PRIMARY KEY ("repo", "tag"),
    FOREIGN KEY (manifest_digest) REFERENCES manifest(digest) ON DELETE CASCADE
) STRICT;
INSERT INTO tag SELECT * FROM tag_old;
DROP TABLE tag_old;

-- Then create the manifest_blob_assoc
CREATE TABLE manifest_blob_assoc (
    manifest_digest TEXT NOT NULL,
    blob_digest TEXT NOT NULL,
    PRIMARY KEY (manifest_digest, blob_digest),
    FOREIGN KEY (manifest_digest) REFERENCES manifest(digest) ON DELETE CASCADE,
    FOREIGN KEY (blob_digest) REFERENCES blob(digest)
);
CREATE INDEX idx_manifest_blob_assoc_blob ON manifest_blob_assoc(blob_digest);

CREATE TRIGGER after_manifest_insert_blob_map
    AFTER INSERT ON manifest
    FOR EACH ROW
BEGIN
    -- Note: a manifest can reference a non existing blob (eg foreign blobs)

    -- Extract blob digests from layers array in the JSON
    INSERT INTO manifest_blob_assoc (manifest_digest, blob_digest)
    SELECT NEW.digest, json_extract(value, '$.digest')
    FROM json_each(json_extract(NEW.json, '$.layers'))
    WHERE json_extract(value, '$.digest') IS NOT NULL
        AND EXISTS (SELECT 1 FROM blob WHERE digest = json_extract(value, '$.digest'))
    ON CONFLICT DO NOTHING;

    -- Also capture config blob, if it exists
    INSERT OR IGNORE INTO manifest_blob_assoc (manifest_digest, blob_digest)
    SELECT NEW.digest, json_extract(NEW.json, '$.config.digest')
    WHERE json_extract(NEW.json, '$.config.digest') IS NOT NULL
        AND EXISTS (SELECT 1 FROM blob WHERE digest = json_extract(NEW.json, '$.config.digest'));
end;


-- Extract blob digests from layers array for all manifests
INSERT INTO manifest_blob_assoc (manifest_digest, blob_digest)
    SELECT m.digest, json_extract(value, '$.digest')
    FROM manifest m
    JOIN json_each(json_extract(m.json, '$.layers'))
    WHERE json_extract(value, '$.digest') IS NOT NULL;

-- Also capture config blobs for all manifests
INSERT OR IGNORE INTO manifest_blob_assoc (manifest_digest, blob_digest)
    SELECT m.digest, json_extract(m.json, '$.config.digest')
    FROM manifest m
    WHERE json_extract(m.json, '$.config.digest') IS NOT NULL;
