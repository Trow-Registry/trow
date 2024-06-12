use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "blob")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub digest: String,
    pub size: i32,
    pub last_accessed: String,
    pub repo: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::manifest_blob_association::Entity")]
    ManifestBlobAssociation,
    #[sea_orm(
        belongs_to = "super::repo::Entity",
        from = "Column::Repo",
        to = "super::repo::Column::id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Repo,
}

impl Related<super::manifest_blob_association::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ManifestBlobAssociation.def()
    }
}

impl Related<super::repo::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repo.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
