use anyhow::Result;
use diesel::sqlite::Sqlite;
use diesel::SqliteConnection;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_async::{AsyncConnection, SimpleAsyncConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tokio::fs;

pub trait DbConnection: AsyncConnection<Backend = Sqlite> {}
impl<T> DbConnection for T where T: AsyncConnection<Backend = Sqlite> {}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub async fn establish_connection() -> Result<impl DbConnection> {
	fs::create_dir_all("data").await?;

	let mut conn = SyncConnectionWrapper::<SqliteConnection>::establish("data/gjallarbot.db").await?;

	conn.batch_execute("PRAGMA foreign_keys = ON").await?;

	Ok(conn)
}

pub async fn run_migrations<D: DbConnection + 'static>(conn: D) -> Result<()> {
	let mut async_wrapper: AsyncConnectionWrapper<D> = AsyncConnectionWrapper::from(conn);
	tokio::task::spawn_blocking(move || {
		async_wrapper.run_pending_migrations(MIGRATIONS).unwrap();
	})
	.await
	.map_err(|e| anyhow::anyhow!(e))
}
