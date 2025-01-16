use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

pub async fn init_db(filename: &str, in_memory: bool) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(filename)
        .create_if_missing(true)
        .in_memory(in_memory)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);

    let conn = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;

    sqlx::migrate!().run(&conn).await?;

    Ok(conn)
}
