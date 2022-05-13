use anyhow::Result;
use reqwest::Client;
use std::env;

pub fn notify(message: &str, tag: &str) -> Result<()> {
    let topic = env::var("NTFYSH_TOPIC")?;
    Client::new()
        .post(&format!("https://ntfy.sh/{}", topic))
        .header("Tags", tag)
        .body(message.to_string())
        .send()?;

    Ok(())
}
