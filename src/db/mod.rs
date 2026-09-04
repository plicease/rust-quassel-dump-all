mod postgres_db;
mod sqlite_db;

pub use postgres_db::{PgConfig, PostgresDb};
pub use sqlite_db::SqliteDb;

use anyhow::Result;

/// Quassel buffer type bits (see Quassel's BufferInfo::Type).
pub mod buffer_type {
    pub const CHANNEL: i32 = 0x02;
    pub const QUERY: i32 = 0x04;
}

pub struct NetworkInfo {
    pub id: i64,
    pub name: String,
}

pub struct BufferInfo {
    pub id: i64,
    pub name: String,
    pub buffer_type: i32,
}

pub struct BacklogRow {
    pub msg_type: i32,
    #[allow(dead_code)]
    pub flags: i32,
    /// Unix timestamp, in seconds.
    pub time: i64,
    pub sender: String,
    pub message: String,
}

/// Common interface over the two supported Quassel storage backends.
pub trait QuasselDb {
    fn user_id(&mut self, username: &str) -> Result<Option<i64>>;
    fn networks(&mut self, user_id: i64) -> Result<Vec<NetworkInfo>>;
    fn buffers(&mut self, user_id: i64, network_id: i64) -> Result<Vec<BufferInfo>>;
    fn backlog(&mut self, buffer_id: i64) -> Result<Vec<BacklogRow>>;
}
