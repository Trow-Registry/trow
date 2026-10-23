-- blob_upload.updated_at was TEXT DEFAULT CURRENT_TIMESTAMP, so a freshly created upload stored a
-- 'YYYY-MM-DD HH:MM:SS' string while update_offset stored a unix epoch. list_stale_older_than_days
-- compares against strftime('%s', ...), which under a TEXT comparison is always false for the
-- datetime form, so uploads abandoned before their first chunk were never reclaimed.
--
-- Store an epoch integer, as blob.last_accessed already does.
CREATE TABLE "blob_upload_new" (
    "uuid" TEXT NOT NULL PRIMARY KEY,
    "offset" INTEGER NOT NULL,
    "updated_at" INTEGER NOT NULL DEFAULT (unixepoch()),
    "repo" TEXT NOT NULL
) STRICT;

INSERT INTO "blob_upload_new" ("uuid", "offset", "updated_at", "repo")
SELECT "uuid",
       "offset",
       -- unixepoch() yields NULL on anything it cannot parse, which would trip the NOT NULL
       -- constraint and fail the migration (and so the boot). A stale upload row is far cheaper
       -- to not date than a registry that won't start, so fall back to 'now'.
       CASE
           -- Non-empty and all digits (so not ''): already an epoch, written by update_offset.
           WHEN "updated_at" GLOB '[0-9]*' AND "updated_at" NOT GLOB '*[^0-9]*'
               THEN CAST("updated_at" AS INTEGER)
           -- Anything else: a CURRENT_TIMESTAMP datetime, or junk.
           ELSE COALESCE(unixepoch("updated_at"), unixepoch())
       END,
       "repo"
FROM "blob_upload";

DROP TABLE "blob_upload";

ALTER TABLE "blob_upload_new" RENAME TO "blob_upload";
