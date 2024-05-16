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
                    .col(ColumnDef::new(Blob::Digest).string().not_null())
                    .col(ColumnDef::new(Blob::Size).integer().not_null())
                    .col(ColumnDef::new(Blob::LastAccessed).timestamp().not_null())
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
                    .col(ColumnDef::new(Manifest::Repo).string().not_null())
                    .col(ColumnDef::new(Manifest::Digest).string().not_null())
                    .col(ColumnDef::new(Manifest::Size).integer().not_null())
                    .col(
                        ColumnDef::new(Manifest::LastAccessed)
                            .timestamp()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Manifest::ArtifactType).string().not_null())
                    .col(ColumnDef::new(Manifest::MediaType).string().null())
                    .col(ColumnDef::new(Manifest::Annotations).json().null())
                    .to_owned(),
            )
            .await
            .unwrap();
        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .col(ColumnDef::new(Tag::Tag).string().not_null())
                    .col(ColumnDef::new(Tag::ManifestId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_tag_manifest")
                            .to(Manifest::Table, Manifest::Id)
                            .from(Tag::Table, Tag::ManifestId)
                            .on_delete(ForeignKeyAction::Cascade),
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
                            .to(Manifest::Table, Manifest::Id)
                            .from(
                                ManifestBlobAssociation::Table,
                                ManifestBlobAssociation::ManifestId,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(ManifestBlobAssociation::BlobId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .to(Blob::Table, Blob::Id)
                            .from(
                                ManifestBlobAssociation::Table,
                                ManifestBlobAssociation::BlobId,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
            .unwrap();
        manager
            .create_table(
                Table::create()
                    .table(Upload::Table)
                    .col(ColumnDef::new(Upload::Uuid).uuid().primary_key().not_null())
                    .col(ColumnDef::new(Upload::Offset).integer().not_null())
                    .col(ColumnDef::new(Upload::LastAccessed).timestamp().not_null())
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
            .drop_table(Table::drop().table(Upload::Table).to_owned())
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
}

#[derive(Iden)]
enum Manifest {
    Table,
    Id,
    Repo,
    Digest,
    Size,
    LastAccessed,
    ArtifactType,
    MediaType,
    Annotations,
}

#[derive(Iden)]
enum Tag {
    Table,
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
enum Upload {
    Table,
    Uuid,
    Offset,
    LastAccessed,
}
