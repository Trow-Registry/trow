use anyhow::Result;
use sqlx::sqlite::SqlitePool;

pub async fn init_db(path: &Option<String>) -> Result<SqlitePool> {
    let path = path.as_ref().map_or("./trow.db", |c| c.as_str());

    let conn = SqlitePool::connect(path).await?;
    sqlx::migrate!().run(&conn).await?;

    // async_conn
    //     .call(|conn| Ok(conn.pragma_update(None, "journal_mode", "WAL")))
    //     .await
    //     .unwrap()
    //     .unwrap();
    // async_conn
    //     .call(|conn| Ok(conn.pragma_update(None, "foreign_keys", "ON")))
    //     .await
    //     .unwrap()
    //     .unwrap();

    Ok(conn)
}
