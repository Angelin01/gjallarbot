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

#[cfg(test)]
pub mod tests {
	use super::*;

	pub async fn setup_test_db() -> SyncConnectionWrapper<SqliteConnection> {
		let conn = SyncConnectionWrapper::<SqliteConnection>::establish(":memory:")
			.await
			.expect("Failed to create in-memory database");

		let async_wrapper: AsyncConnectionWrapper<_> = AsyncConnectionWrapper::from(conn);
		let conn = tokio::task::spawn_blocking(move || {
			let mut wrapper = async_wrapper;
			wrapper
				.run_pending_migrations(MIGRATIONS)
				.expect("Failed to run migrations");
			wrapper.into_inner()
		})
		.await
		.expect("Migration task panicked");

		let mut conn = SyncConnectionWrapper::from(conn);
		conn.batch_execute("PRAGMA foreign_keys = ON")
			.await
			.expect("Failed to enable foreign keys");

		conn
	}
}
