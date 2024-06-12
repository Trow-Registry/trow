use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "tag")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub tag: String,
    pub manifest_id: i32,
    pub repo_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::manifest::Entity",
        from = "Column::ManifestId",
        to = "super::manifest::Column::id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Manifest,
    #[sea_orm(
        belongs_to = "super::repo::Entity",
        from = "Column::RepoId",
        to = "super::repo::Column::id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Repo,
}

impl Related<super::manifest::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Manifest.def()
    }
}

impl Related<super::repo::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repo.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
