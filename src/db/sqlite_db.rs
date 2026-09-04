use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

use super::{BacklogRow, BufferInfo, NetworkInfo, QuasselDb};

pub struct SqliteDb {
    conn: Connection,
}

impl SqliteDb {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("failed to open sqlite database {}", path.display()))?;
        Ok(Self { conn })
    }
}

impl QuasselDb for SqliteDb {
    fn user_id(&mut self, username: &str) -> Result<Option<i64>> {
        let mut stmt = self
            .conn
            .prepare("SELECT userid FROM quasseluser WHERE username = ?1")?;
        let id = stmt.query_row([username], |row| row.get::<_, i64>(0)).ok();
        Ok(id)
    }

    fn networks(&mut self, user_id: i64) -> Result<Vec<NetworkInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT networkid, networkname FROM network WHERE userid = ?1 ORDER BY networkname",
        )?;
        let rows = stmt
            .query_map([user_id], |row| {
                Ok(NetworkInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn buffers(&mut self, user_id: i64, network_id: i64) -> Result<Vec<BufferInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT bufferid, buffername, buffertype FROM buffer \
             WHERE userid = ?1 AND networkid = ?2 ORDER BY buffername",
        )?;
        let rows = stmt
            .query_map([user_id, network_id], |row| {
                Ok(BufferInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    buffer_type: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn backlog(&mut self, buffer_id: i64) -> Result<Vec<BacklogRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT backlog.type, backlog.flags, backlog.time, sender.sender, backlog.message \
             FROM backlog JOIN sender ON backlog.senderid = sender.senderid \
             WHERE backlog.bufferid = ?1 ORDER BY backlog.messageid ASC",
        )?;
        let rows = stmt
            .query_map([buffer_id], |row| {
                // Quassel stores backlog.time as milliseconds since the epoch in sqlite.
                let time_ms: i64 = row.get(2)?;
                let message: Option<String> = row.get(4)?;
                Ok(BacklogRow {
                    msg_type: row.get(0)?,
                    flags: row.get(1)?,
                    time: time_ms / 1000,
                    sender: row.get(3)?,
                    message: message.unwrap_or_default(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}
