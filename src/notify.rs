use anyhow::Result;
use reqwest::Client;
use std::env;

pub fn notify(message: String) -> Result<()> {
    let topic = env::var("NTFYSH_TOPIC")?;
    Client::new()
        .post(&format!("https://ntfy.sh/{}", topic))
        .body(message)
        .send()?;

    Ok(())
}
