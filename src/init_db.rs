use sqlx::migrate::MigrateError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

pub async fn init_db(filename: &str) -> Result<(SqlitePool, SqlitePool), MigrateError> {
    let options = SqliteConnectOptions::new()
        .filename(filename)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);

    let writer_conn = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options.clone().create_if_missing(true))
        .await?;

    let reader_conn = SqlitePoolOptions::new()
        .connect_with(options.read_only(true))
        .await?;

    sqlx::migrate!().run(&writer_conn).await?;

    Ok((reader_conn, writer_conn))
}
