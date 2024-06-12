use sea_orm_migration::prelude::*;

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
                    .table(Blob::Table)
                    .col(
                        ColumnDef::new(Blob::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Blob::Digest)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Blob::Size).integer().not_null())
                    .col(ColumnDef::new(Blob::LastAccessed).timestamp().not_null())
                    .col(ColumnDef::new(Blob::Repo).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_blob_repo")
                            .from(Blob::Table, Blob::Repo)
                            .to(Repo::Table, Repo::Name)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
            .unwrap();
        manager
            .create_table(
                Table::create()
                    .table(Manifest::Table)
                    .col(
                        ColumnDef::new(Manifest::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Manifest::Digest)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Manifest::Size).integer().not_null())
                    .col(
                        ColumnDef::new(Manifest::LastAccessed)
                            .timestamp()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Manifest::Repo).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_manifest_repo")
                            .from(Manifest::Table, Manifest::Repo)
                            .to(Repo::Table, Repo::Name)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
            .unwrap();
        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .col(ColumnDef::new(Tag::Tag).string().not_null())
                    .col(ColumnDef::new(Tag::Repo).string().not_null())
                    .col(ColumnDef::new(Tag::ManifestId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_tag_manifest")
                            .from(Tag::Table, Tag::ManifestId)
                            .to(Manifest::Table, Manifest::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_tag_repo")
                            .from(Tag::Table, Tag::Repo)
                            .to(Repo::Table, Repo::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("IDX_repo_tag")
                            .table(Tag::Table)
                            .col(Tag::Repo)
                            .col(Tag::Tag)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
            .unwrap();
        manager
            .create_table(
                Table::create()
                    .table(ManifestBlobAssociation::Table)
                    .col(
                        ColumnDef::new(ManifestBlobAssociation::ManifestId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                ManifestBlobAssociation::Table,
                                ManifestBlobAssociation::ManifestId,
                            )
                            .to(Manifest::Table, Manifest::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(ManifestBlobAssociation::BlobId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                ManifestBlobAssociation::Table,
                                ManifestBlobAssociation::BlobId,
                            )
                            .to(Blob::Table, Blob::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
            .unwrap();
        manager
            .create_table(
                Table::create()
                    .table(BlobUpload::Table)
                    .col(
                        ColumnDef::new(BlobUpload::Uuid)
                            .uuid()
                            .primary_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(BlobUpload::Offset).integer().not_null())
                    .col(
                        ColumnDef::new(BlobUpload::LastAccessed)
                            .timestamp()
                            .not_null(),
                    )
                    .col(ColumnDef::new(BlobUpload::Repo).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(BlobUpload::Table, BlobUpload::Repo)
                            .to(Repo::Table, Repo::Name)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
            .unwrap();
        manager
            .create_table(
                Table::create()
                    .table(Repo::Table)
                    .col(ColumnDef::new(Repo::Name).string().primary_key().not_null())
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
            .drop_table(Table::drop().table(Manifest::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ManifestBlobAssociation::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(BlobUpload::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Repo::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Blob {
    Table,
    Id,
    Digest,
    Size,
    LastAccessed,
    Repo,
}

#[derive(Iden)]
enum Manifest {
    Table,
    Id,
    Digest,
    Size,
    LastAccessed,
    Repo,
}

#[derive(Iden)]
enum Tag {
    Table,
    Repo,
    Tag,
    ManifestId,
}

#[derive(Iden)]
enum ManifestBlobAssociation {
    Table,
    ManifestId,
    BlobId,
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
