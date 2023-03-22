#![feature(never_type)]

// mod copilot;
mod ban;
mod notify;
mod poll;

use ban::BanDB;
use poll::PollDB;

#[macro_use]
extern crate rocket;
use rocket::{
    fs::NamedFile,
    http::{Status, CookieJar},
    response::status,
    serde::json::Json,
    shield::{Frame, Shield},
    Request
};
use rocket_client_addr::ClientAddr;
use rocket_cors::CorsOptions;
use rocket_dyn_templates::Template;

use comrak::{markdown_to_html, ComrakExtensionOptions, ComrakOptions, ComrakRenderOptions};
use dotenv::dotenv;
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    fmt,
    fmt::{Display, Formatter},
    fs,
    net::IpAddr,
    path::{Path, PathBuf},
};

// pages

#[derive(Serialize)]
struct Page {
    title: String,
    body: String,
}

fn render(title: &str, path: &str, address: IpAddr) -> Result<Template, Status> {
    let mut ban_db = BanDB::new().unwrap();
    if ban_db.is_banned(address).unwrap() {
        return Err(Status::Forbidden);
    }
    
    let body = fs::read_to_string(path).map_err(|_| Status::NotFound)?;
    let options = ComrakOptions {
        extension: ComrakExtensionOptions {
            footnotes: true,
            ..ComrakExtensionOptions::default()
        },
        render: ComrakRenderOptions {
            unsafe_: true,
            ..ComrakRenderOptions::default()
        },
        ..ComrakOptions::default()
    };
    Ok(Template::render(
        "page",
        &Page {
            title: title.to_string(),
            body: markdown_to_html(&body, &options).to_string(),
        },
    ))
}

#[catch(403)]
fn on_403() -> status::Custom<()> {
    status::Custom(Status::Forbidden, ())
}

#[catch(404)]
async fn on_404(request: &Request<'_>) -> Result<Template, Status> {
    render("404", "pages/404.md", request.client_ip().unwrap())
}

#[get("/")]
async fn index_page(address: ClientAddr) -> Result<Template, Status> {
    render("index", "pages/index.md", address.ip)
}

#[get("/<file>")]
async fn other_file(file: String, address: ClientAddr) -> Result<NamedFile, Result<Template, Status>> {
    match NamedFile::open(Path::new("static/").join(PathBuf::from(&file)))
        .await
        .ok()
    {
        Some(file) => Ok(file),
        None => Err(render(
            file.clone().as_str(),
            format!("pages/{}.md", file).as_str(),
            address.ip,
        )),
    }
}

// api

fn json_error(status: Status, message: String) -> status::Custom<Json<Value>> {
    status::Custom(status, Json(json!({ "error": message })))
}

// #[post("/api/copilot", format = "json", data = "<data>")]
// fn copilot_endpoint(
//     data: Json<copilot::Request>,
// ) -> Result<Json<Value>, status::Custom<Json<Value>>> {
//     let copilot::Request {
//         prompt,
//         max_tokens,
//         temperature,
//         top_p,
//     } = data.into_inner();
//     let temperature = temperature.unwrap_or(1.0);
//     let top_p = top_p.unwrap_or(0.9);

//     match notify::notify(
//         format!("copilot request with prompt: {}", prompt.clone()).as_str(),
//         "airplane",
//     ) {
//         Ok(_) => (),
//         Err(e) => return Err(json_error(Status::InternalServerError, e.to_string())),
//     }

//     let output = copilot::get_copilot(prompt, max_tokens, temperature, top_p);
//     match output {
//         Ok(output) => Ok(Json(json!({ "ok": true, "output": output }))),
//         Err(_) => Err(json_error(Status::BadRequest, "...".to_string())),
//     }
// }

#[get("/api/ban")]
fn ban_endpoint(address: ClientAddr) -> status::Custom<()> {
    let mut ban_db = BanDB::new().unwrap();
    ban_db.ban(address.ip).unwrap();

    let _ = notify::notify(format!("BANNED: {}", address.ip).as_str(), "no_entry");
    status::Custom(Status::Forbidden, ())
}

#[get("/api/unban")]
fn unban_endpoint(address: ClientAddr) -> status::Custom<()> {
    let mut ban_db = BanDB::new().unwrap();
    ban_db.unban(address.ip).unwrap();

    let _ = notify::notify(format!("UNBANNED: {}", address.ip).as_str(), "white_check_mark");
    status::Custom(Status::Ok, ())
}

#[derive(serde::Deserialize)]
struct Feedback {
    feedback: String,
}

#[post("/api/feedback", format = "json", data = "<feedback>")]
fn feedback_endpoint(feedback: Json<Feedback>) {
    let Feedback { feedback } = feedback.into_inner();
    notify::notify(
        format!("NEW FEEDBACK: {}", feedback).as_str(),
        "love_letter",
    )
    .unwrap()
}

#[get("/api/poll/get/<poll_id>")]
fn get_poll_endpoint(poll_id: &str) -> Result<Json<Value>, status::Custom<Json<Value>>> {
    match PollDB::new().unwrap().get(poll_id) {
        Ok(Some(poll)) => Ok(Json(json!({ "ok": true, "poll": poll }))),
        Ok(None) => Err(json_error(
            Status::NotFound,
            "this poll doesn't exist".to_string(),
        )),
        Err(_) => Err(json_error(
            Status::InternalServerError,
            "unknown error".to_string(),
        )),
    }
}

#[post("/api/poll/create", format = "json", data = "<options>")]
fn new_poll_endpoint(options: Json<Vec<String>>) -> Json<Value> {
    let poll_id = PollDB::new().unwrap().create(options.into_inner()).unwrap(); // :(
    notify::notify(
        format!("NEW POLL: https://celeste.exposed/poll/{}", poll_id).as_str(),
        "ballot_box",
    )
    .unwrap();
    Json(json!({
        "ok": true,
        "url": format!("https://celeste.exposed/poll/{}", poll_id)
    }))
}

#[post("/api/poll/vote", format = "json", data = "<data>")]
fn vote_poll_endpoint(data: Json<poll::PollVote>) -> Json<Value> {
    let poll::PollVote {
        poll_id,
        option,
        fingerprint,
    } = data.into_inner();
    match PollDB::new()
        .unwrap()
        .vote(poll_id.as_str(), option.as_str(), fingerprint.as_str())
    {
        Ok(()) => Json(json!({ "ok": true })),
        Err(e) => Json(json!({ "ok": false, "error": format!("{}", e) })),
    }
}

#[post("/api/poll/voted", format = "json", data = "<data>")]
fn voted_poll_endpoint(data: Json<poll::PollVoteCheck>) -> Json<Value> {
    let poll::PollVoteCheck {
        poll_id,
        fingerprint,
    } = data.into_inner();
    match PollDB::new()
        .unwrap()
        .voted_on(fingerprint.as_str(), poll_id.as_str())
    {
        Ok(voted) => Json(json!({ "ok": true, "voted": voted })),
        Err(e) => Json(json!({ "ok": false, "error": format!("{}", e) })),
    }
}

#[get("/poll/<poll_id>")]
fn poll_page(poll_id: &str) -> Result<Template, Status> {
    let res = poll::PollDB::new().unwrap().get(poll_id.clone());
    match res {
        Ok(Some(data)) => Ok(Template::render(
            "poll",
            poll::Poll {
                id: poll_id.to_string(),
                data,
            },
        )),
        Ok(None) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[derive(serde::Deserialize)]
struct PageVisit {
    url: String,
}

#[derive(serde::Deserialize)]
struct IPLocation {
    city: String,
    country: String,
}

impl Display for IPLocation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}, {}", self.city, self.country)
    }
}

#[post("/api/visited", format = "json", data = "<data>")]
fn visited_endpoint(data: Json<PageVisit>, address: ClientAddr, jar: &CookieJar<'_>) {
    let cookie = jar.get("ip");

    let mut ip = match cookie {
        Some(cookie) => cookie.value(),
        None => "",
    };

    let server_ip = format!("{}", address.ip);

    if ip == "" {
        ip = &server_ip;
    }

    let PageVisit { url } = data.into_inner();

    let mut req = reqwest::get(&format!("http://ip-api.com/json/{}", ip)).unwrap();
    let location = format!("{}", req.json::<IPLocation>().unwrap());
    
    notify::notify(
        format!("VISIT: {} from {}", url, location).as_str(),
        "eye",
    )
    .unwrap();
}

#[derive(serde::Deserialize)]
struct TextSynthRequest {
    api_key: String,
    model: String,
    prompt: String,
    max_tokens: u32,
    n: u32,
    temperature: f32,
    top_p: f32,
}

#[post("/api/textsynth", format = "json", data = "<data>")]
fn textsynth_endpoint(data: Json<TextSynthRequest>) -> Json<Value> {
    let TextSynthRequest {
        api_key,
        model,
        prompt,
        max_tokens,
        n,
        temperature,
        top_p,
    } = data.into_inner();

    let client = reqwest::Client::new();
    let url = format!("https://api.textsynth.com/v1/engines/{}/completions", model);
    eprintln!("url: {}", url);
    let response: Value = client.post(&url)
        .json(&json!({
            "prompt": prompt,
            "max_tokens": max_tokens,
            "n": n,
            "temperature": temperature,
            "top_p": top_p,
        }))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .unwrap()
        .json()
        .unwrap();

    Json(json!({
        "ok": true,
        "response": response
    }))
}

// main

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    rocket::build()
        .mount(
            "/",
            routes![
                index_page,
                other_file,
                ban_endpoint,
                unban_endpoint,
                // copilot_endpoint,
                feedback_endpoint,
                get_poll_endpoint,
                new_poll_endpoint,
                vote_poll_endpoint,
                voted_poll_endpoint,
                poll_page,
                visited_endpoint,
                textsynth_endpoint,
            ],
        )
        .register("/", catchers![on_403, on_404])
        .attach(Template::fairing())
        .attach(Shield::default().disable::<Frame>())
        .attach(CorsOptions::default().to_cors().unwrap())
}
