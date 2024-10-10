use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240516_000001_initial_migration"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Bakery table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Repo::Table)
                    .col(string(Repo::Name).primary_key())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Blob::Table)
                    .col(string(Blob::Digest).primary_key())
                    .col(integer(Blob::Size))
                    .col(boolean(Blob::IsManifest))
                    .col(
                        timestamp(Blob::LastAccessed)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    // .index(IndexCreateStatement::new().col(Blob::IsManifest))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(RepoBlobAssociation::Table)
                    .col(string(RepoBlobAssociation::RepoName))
                    .col(string(RepoBlobAssociation::BlobDigest))
                    .primary_key(
                        Index::create()
                            .col(RepoBlobAssociation::RepoName)
                            .col(RepoBlobAssociation::BlobDigest),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_repo_blob_repo_name")
                            .from(RepoBlobAssociation::Table, RepoBlobAssociation::RepoName)
                            .to(Repo::Table, Repo::Name)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_repo_blob_blob_digest")
                            .from(RepoBlobAssociation::Table, RepoBlobAssociation::BlobDigest)
                            .to(Blob::Table, Blob::Digest)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .col(string(Tag::Tag))
                    .col(string(Tag::Repo))
                    .col(string(Tag::ManifestDigest))
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_repo_blob_assoc")
                            .from_tbl(Tag::Table)
                            .from_col(Tag::Repo)
                            .from_col(Tag::ManifestDigest)
                            .to_tbl(RepoBlobAssociation::Table)
                            .to_col(RepoBlobAssociation::RepoName)
                            .to_col(RepoBlobAssociation::BlobDigest)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .name("IDX_repo_tag")
                            .table(Tag::Table)
                            .col(Tag::Repo)
                            .col(Tag::Tag)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(BlobBlobAssociation::Table)
                    .col(string(BlobBlobAssociation::ParentDigest))
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_blob_blob_association_parent")
                            .from(
                                BlobBlobAssociation::Table,
                                BlobBlobAssociation::ParentDigest,
                            )
                            .to(Blob::Table, Blob::Digest)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(string(BlobBlobAssociation::ChildDigest))
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_blob_blob_association_child")
                            .from(BlobBlobAssociation::Table, BlobBlobAssociation::ChildDigest)
                            .to(Blob::Table, Blob::Digest),
                    )
                    .primary_key(
                        Index::create()
                            .col(BlobBlobAssociation::ParentDigest)
                            .col(BlobBlobAssociation::ChildDigest),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(BlobUpload::Table)
                    .col(uuid(BlobUpload::Uuid).primary_key())
                    .col(integer(BlobUpload::Offset))
                    .col(
                        timestamp(BlobUpload::LastAccessed)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(string(BlobUpload::Repo).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_blob_upload_repo")
                            .from(BlobUpload::Table, BlobUpload::Repo)
                            .to(Repo::Table, Repo::Name)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Blob::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BlobBlobAssociation::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BlobUpload::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Repo::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RepoBlobAssociation::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Blob {
    Table,
    Digest,
    Size,
    IsManifest,
    LastAccessed,
}

#[derive(Iden)]
enum Tag {
    Table,
    Repo,
    Tag,
    ManifestDigest,
}

#[derive(Iden)]
enum BlobBlobAssociation {
    Table,
    ParentDigest,
    ChildDigest,
}

#[derive(Iden)]
enum BlobUpload {
    Table,
    Repo,
    Uuid,
    Offset,
    LastAccessed,
}

#[derive(Iden)]
enum Repo {
    Table,
    Name,
}

#[derive(Iden)]
enum RepoBlobAssociation {
    Table,
    RepoName,
    BlobDigest,
}
