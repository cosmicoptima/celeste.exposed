use anyhow::{anyhow, Result};
use bincode::{deserialize, serialize};
use nanoid::nanoid;
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize)]
pub struct PollData {
    pub votes: Vec<PollOption>,
}

#[derive(Deserialize, Serialize)]
pub struct PollOption {
    pub name: String,
    pub votes: usize,
}

#[derive(Serialize)]
pub struct Poll {
    pub id: String,
    pub data: PollData,
}

#[derive(Deserialize, Serialize)]
pub struct PollVote {
    pub poll_id: String,
    pub option: String,
    pub fingerprint: String,
}

#[derive(Deserialize, Serialize)]
pub struct PollVoteCheck {
    pub fingerprint: String,
    pub poll_id: String,
}

pub struct PollDB {
    conn: redis::Connection,
}

impl PollDB {
    pub fn new() -> Result<Self> {
        Ok(PollDB {
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

    pub fn get(&mut self, poll_id: &str) -> Result<Option<PollData>> {
        Ok(
            if let Some(data) = self.conn.get::<_, Option<String>>(poll_id)? {
                Some(deserialize::<PollData>(data.as_bytes())?)
            } else {
                None
            },
        )
    }

    pub fn create(&mut self, options: Vec<String>) -> Result<String> {
        let poll_id = nanoid!(10);

        let mut votes = vec![];
        for option in options {
            votes.push(PollOption {
                name: option,
                votes: 0,
            });
        }

        self.conn.set(&poll_id, serialize(&votes)?)?;
        Ok(poll_id)
    }

    pub fn vote(&mut self, poll_id: &str, option: &str, fingerprint: &str) -> Result<()> {
        if self.voted_on(poll_id, fingerprint)? {
            return Ok(());
        }
        self.conn
            .set(format!("{}:{}", poll_id, fingerprint), option)?;

        if let Some(poll_data) = self.get(poll_id)? {
            let mut votes = poll_data.votes;
            for vote in &mut votes {
                if vote.name == option {
                    vote.votes += 1;
                }
            }
            self.conn.set(poll_id, serialize(&votes)?)?;
        } else {
            return Err(anyhow!("Poll not found"));
        }

        Ok(())
    }

    pub fn voted_on(&mut self, poll_id: &str, fingerprint: &str) -> Result<bool> {
        Ok(self.conn.exists(format!("{}:{}", poll_id, fingerprint))?)
    }
}
