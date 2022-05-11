use anyhow::{anyhow, Result};
use bincode::{deserialize, serialize};
use lazy_static::lazy_static;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sled::Db;

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

lazy_static! {
    static ref DB: Db = sled::open("data").unwrap();
}

pub fn get_poll(poll_id: String) -> Result<Option<PollData>> {
    Ok(if let Some(options_bin) = DB.get(poll_id)? {
        Some(deserialize(&options_bin)?)
    } else {
        None
    })
}

pub fn create_poll(options: Vec<String>) -> String {
    let poll_id = nanoid!(6);

    let mut votes = vec![];
    for option in options {
        votes.push(PollOption {
            name: option,
            votes: 0,
        });
    }

    DB.insert(poll_id.clone(), serialize(&PollData { votes }).unwrap())
        .unwrap();
    poll_id
}

pub fn vote_poll(poll_id: String, option: String, fingerprint: String) -> Result<()> {
    if voted_on(fingerprint.clone(), poll_id.clone())? {
        return Ok(());
    }

    if let Some(mut poll_data) = get_poll(poll_id.clone())? {
        for vote in &mut poll_data.votes {
            if vote.name == option {
                vote.votes += 1;
            }
        }
        DB.insert(poll_id.clone(), serialize(&poll_data)?)?;

        match DB.get(fingerprint.clone())? {
            Some(data) => {
                let mut poll_ids: Vec<String> = deserialize(&data)?;
                if !poll_ids.contains(&poll_id) {
                    poll_ids.push(poll_id);
                    DB.insert(fingerprint, serialize(&poll_ids)?)?;
                }
            }
            None => {
                DB.insert(fingerprint, serialize(&vec![poll_id])?)?;
            }
        }

        Ok(())
    } else {
        Err(anyhow!("Poll not found"))
    }
}

pub fn voted_on(fingerprint: String, poll_id: String) -> Result<bool> {
    if DB.contains_key(fingerprint.clone())? {
        let poll_ids: Vec<String> = deserialize(&DB.get(fingerprint)?.unwrap())?;
        Ok(poll_ids.contains(&poll_id))
    } else {
        Ok(false)
    }
}
