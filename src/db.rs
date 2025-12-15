use anyhow::Error;
use diesel::sqlite::Sqlite;
use diesel::SqliteConnection;
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_async::{AsyncConnection, SimpleAsyncConnection};
use tokio::fs;

pub trait DbConnection: AsyncConnection<Backend = Sqlite> {}
impl<T> DbConnection for T where T: AsyncConnection<Backend = Sqlite> {}

pub async fn establish_connection() -> Result<impl AsyncConnection<Backend = Sqlite>, Error> {
	fs::create_dir_all("data").await?;

	let mut conn = SyncConnectionWrapper::<SqliteConnection>::establish("data/gjallarbot.db").await?;

	conn.batch_execute("PRAGMA foreign_keys = ON").await?;

	Ok(conn)
}
