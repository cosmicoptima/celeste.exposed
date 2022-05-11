use nom::{
    bytes::complete::{is_not, tag},
    combinator::fail,
    multi::many1,
    IResult,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

#[derive(Deserialize)]
pub struct Request {
    pub prompt: String,
    pub max_tokens: usize,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
}

pub enum Error {
    ZeroMaxTokens,
    PromptTooLong,
    InvalidTemperature,
    InvalidTopP,
    UnknownError(String),
}

#[derive(Deserialize)]
struct CopilotToken {
    token: String,
}

#[derive(Serialize)]
struct CopilotRequest {
    prompt: String,
    max_tokens: usize,
    temperature: f64,
    top_p: f64,
    stream: bool,
}

fn get_copilot_token() -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let oauth_token = env::var("COPILOT_OAUTH_TOKEN")?;
    Ok(client
        .get("https://api.github.com/copilot_internal/token")
        .header("Authorization", format!("Bearer {}", oauth_token))
        .send()?
        .json::<CopilotToken>()?
        .token)
}

fn parse_response(input: &str) -> IResult<&str, String> {
    let (input, tokens) = many1(parse_token)(input)?;
    Ok((input, tokens.join("")))
}

fn parse_token(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("data: ")(input)?;
    let (input, json) = is_not("\n")(input)?;
    let (input, _) = tag("\n\n")(input)?;

    let error_str = "Could not parse JSON";

    match serde_json::from_str::<Value>(json) {
        Ok(value) => match &value["choices"][0]["text"] {
            Value::String(token) => Ok((input, token.to_string())),
            _ => fail(error_str),
        },
        Err(_) => fail(error_str),
    }
}

pub fn get_copilot(
    prompt: String,
    max_tokens: usize,
    temperature: f64,
    top_p: f64,
) -> Result<String, Error> {
    let token =
        get_copilot_token().map_err(|_| Error::UnknownError("Could not get token".to_string()))?;

    if max_tokens == 0 {
        return Err(Error::ZeroMaxTokens);
    }
    if prompt.len() > 4096 {
        return Err(Error::PromptTooLong);
    }
    if temperature < 0.0 {
        return Err(Error::InvalidTemperature);
    }
    if top_p < 0.0 || top_p > 1.0 {
        return Err(Error::InvalidTopP);
    }

    let client = reqwest::Client::new();
    let mut response = client
        .post("https://copilot-proxy.githubusercontent.com/v1/engines/copilot-codex/completions")
        .header("Authorization", format!("Bearer {}", token))
        .json(&CopilotRequest {
            prompt,
            max_tokens,
            temperature,
            top_p,
            stream: true,
        })
        .send()
        .map_err(|_| Error::UnknownError("Could not send request".to_string()))?;

    let (_, response) = parse_response(
        &response
            .text()
            .map_err(|_| Error::UnknownError("Could not get response".to_string()))?,
    )
    .map_err(|_| Error::UnknownError("Could not parse response".to_string()))?;
    Ok(response)
}
