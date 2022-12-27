use anyhow::Result;
use redis::Commands;
use std::{env, net::IpAddr};

pub struct BanDB {
    conn: redis::Connection,
}

impl BanDB {
    pub fn new() -> Result<Self> {
        Ok(BanDB {
            conn: redis::Client::open(redis::ConnectionInfo {
                addr: redis::ConnectionAddr::TcpTls {
                    host: env::var("REDIS_HOST")?,
                    port: env::var("REDIS_PORT")?.parse()?,
                    insecure: false,
                },
                redis: redis::RedisConnectionInfo {
                    username: Some(env::var("REDIS_USERNAME")?),
                    password: Some(env::var("REDIS_PASSWORD")?),
                    db: 0,
                },
            })?
            .get_connection()?,
        })
    }

    pub fn ban(&mut self, ip: IpAddr) -> Result<()> {
        self.conn.set(ip.to_string(), true)?;
        Ok(())
    }

    pub fn unban(&mut self, ip: IpAddr) -> Result<()> {
        self.conn.del(ip.to_string())?;
        Ok(())
    }

    pub fn is_banned(&mut self, ip: IpAddr) -> Result<bool> {
        Ok(self.conn.exists(ip.to_string())?)
    }
}
