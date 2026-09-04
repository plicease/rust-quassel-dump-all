use anyhow::{Context, Result};
use postgres::{Client, NoTls};

use super::{BacklogRow, BufferInfo, NetworkInfo, QuasselDb};

pub struct PgConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: Option<String>,
    pub dbname: String,
}

pub struct PostgresDb {
    client: Client,
}

impl PostgresDb {
    pub fn connect(cfg: &PgConfig) -> Result<Self> {
        let mut config = postgres::Config::new();
        config
            .host(&cfg.host)
            .port(cfg.port)
            .user(&cfg.user)
            .dbname(&cfg.dbname);
        if let Some(password) = &cfg.password {
            config.password(password);
        }
        let client = config.connect(NoTls).with_context(|| {
            format!(
                "failed to connect to postgres database {}@{}:{}/{}",
                cfg.user, cfg.host, cfg.port, cfg.dbname
            )
        })?;
        Ok(Self { client })
    }
}

impl QuasselDb for PostgresDb {
    fn user_id(&mut self, username: &str) -> Result<Option<i64>> {
        let row = self.client.query_opt(
            "SELECT userid FROM quasseluser WHERE username = $1",
            &[&username],
        )?;
        Ok(row.map(|r| r.get::<_, i32>(0) as i64))
    }

    fn networks(&mut self, user_id: i64) -> Result<Vec<NetworkInfo>> {
        let rows = self.client.query(
            "SELECT networkid, networkname FROM network WHERE userid = $1 ORDER BY networkname",
            &[&(user_id as i32)],
        )?;
        Ok(rows
            .into_iter()
            .map(|r| NetworkInfo {
                id: r.get::<_, i32>(0) as i64,
                name: r.get(1),
            })
            .collect())
    }

    fn buffers(&mut self, user_id: i64, network_id: i64) -> Result<Vec<BufferInfo>> {
        let rows = self.client.query(
            "SELECT bufferid, buffername, buffertype FROM buffer \
             WHERE userid = $1 AND networkid = $2 ORDER BY buffername",
            &[&(user_id as i32), &(network_id as i32)],
        )?;
        Ok(rows
            .into_iter()
            .map(|r| BufferInfo {
                id: r.get::<_, i32>(0) as i64,
                name: r.get(1),
                buffer_type: r.get::<_, i32>(2),
            })
            .collect())
    }

    fn backlog(&mut self, buffer_id: i64) -> Result<Vec<BacklogRow>> {
        let rows = self.client.query(
            "SELECT backlog.type, backlog.flags, EXTRACT(EPOCH FROM backlog.time)::BIGINT, \
                    sender.sender, backlog.message \
             FROM backlog JOIN sender ON backlog.senderid = sender.senderid \
             WHERE backlog.bufferid = $1 ORDER BY backlog.messageid ASC",
            &[&(buffer_id as i32)],
        )?;
        Ok(rows
            .into_iter()
            .map(|r| BacklogRow {
                msg_type: r.get::<_, i32>(0),
                flags: r.get::<_, i32>(1),
                time: r.get::<_, i64>(2),
                sender: r.get(3),
                message: r.get::<_, Option<String>>(4).unwrap_or_default(),
            })
            .collect())
    }
}
