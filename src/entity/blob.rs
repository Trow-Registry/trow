use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::{NotSet, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "blob")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub digest: String,
    #[sea_orm(primary_key)]
    pub repo: String,
    pub size: i32,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub last_accessed: chrono::DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::manifest_blob_association::Entity")]
    ManifestBlobAssociation,
    #[sea_orm(
        belongs_to = "super::repo::Entity",
        from = "Column::Repo",
        to = "super::repo::Column::Name",
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

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            digest: NotSet,
            repo: NotSet,
            size: NotSet,
            last_accessed: Set(Utc::now()),
        }
    }
}
