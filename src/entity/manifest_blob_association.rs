use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "manifest_blob_association")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub manifest_id: i32,
    pub blob_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::blob::Entity",
        from = "Column::BlobId",
        to = "super::blob::Column::id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Blob,
    #[sea_orm(
        belongs_to = "super::manifest::Entity",
        from = "Column::ManifestId",
        to = "super::manifest::Column::id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Manifest,
}

impl Related<super::blob::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Blob.def()
    }
}

impl Related<super::manifest::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Manifest.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
