mod copilot;
mod notify;
mod poll;

#[macro_use]
extern crate rocket;
use rocket::{
    http::Status,
    response::status,
    serde::json::Json,
    shield::{Frame, Shield},
};
use rocket_dyn_templates::Template;

use comrak::{markdown_to_html, ComrakExtensionOptions, ComrakOptions, ComrakRenderOptions};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;

// static

#[derive(Responder)]
#[response(content_type = "text/css")]
struct CSSResponse(String);

#[derive(Responder)]
#[response(content_type = "text/javascript")]
struct JSResponse(String);

#[derive(Responder)]
#[response(content_type = "font/woff2")]
struct FontResponse(Vec<u8>);

#[derive(Responder)]
#[response(content_type = "image/x-icon")]
struct IconResponse(Vec<u8>);

#[get("/embed.css")]
fn embed_css() -> CSSResponse {
    CSSResponse(include_str!("../static/embed.css").to_string())
}

#[get("/style.css")]
fn css() -> CSSResponse {
    CSSResponse(include_str!("../static/style.css").to_string())
}

#[get("/index.js")]
fn index_js() -> JSResponse {
    JSResponse(include_str!("../static/index.js").to_string())
}

#[get("/alignment.js")]
fn alignment_js() -> JSResponse {
    JSResponse(include_str!("../static/alignment.js").to_string())
}

#[get("/copilot.js")]
fn copilot_js() -> JSResponse {
    JSResponse(include_str!("../static/copilot.js").to_string())
}

#[get("/LibreBaskerville-400.woff2")]
fn libre_baskerville_400() -> FontResponse {
    FontResponse(include_bytes!("../static/LibreBaskerville-400.woff2").to_vec())
}

#[get("/LibreBaskerville-700.woff2")]
fn libre_baskerville_700() -> FontResponse {
    FontResponse(include_bytes!("../static/LibreBaskerville-700.woff2").to_vec())
}

#[get("/SourceCodePro-400.woff2")]
fn source_code_pro_400() -> FontResponse {
    FontResponse(include_bytes!("../static/SourceCodePro-400.woff2").to_vec())
}

#[get("/SourceCodePro-700.woff2")]
fn source_code_pro_700() -> FontResponse {
    FontResponse(include_bytes!("../static/SourceCodePro-700.woff2").to_vec())
}

#[get("/favicon.ico")]
fn favicon() -> IconResponse {
    IconResponse(include_bytes!("../static/favicon.ico").to_vec())
}

// pages

#[derive(Serialize)]
struct Page {
    title: String,
    desc: String,
    body: String,
}

fn render(title: &str, desc: &str, path: &str) -> Result<Template, Status> {
    match notify::notify(
        format!("page visited: {}", title).as_str(),
        "page_with_curl",
    ) {
        Ok(_) => {}
        Err(_) => return Err(Status::InternalServerError),
    }

    let body = fs::read_to_string(path).map_err(|_| Status::InternalServerError)?;
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
            desc: desc.to_string(),
            body: markdown_to_html(&body, &options).to_string(),
        },
    ))
}

#[catch(404)]
fn on_404() -> Result<Template, Status> {
    render("404", "nonsense useless page", "pages/404.md")
}

#[get("/")]
fn index_page() -> Result<Template, Status> {
    render("index", "celeste homepage", "pages/index.md")
}

#[get("/about")]
fn about_page() -> Result<Template, Status> {
    render("about", "...", "pages/about.md")
}

#[get("/ads")]
fn ads_page() -> Result<Template, Status> {
    render("ads", "...", "pages/ads.md")
}

#[get("/alignment")]
fn alignment_page() -> Result<Template, Status> {
    render("alignment", "...", "pages/alignment.md")
}

#[get("/poll")]
fn poll_page() -> Result<Template, Status> {
    render(
        "poll",
        "create a twitter poll with more than four options",
        "pages/poll.md",
    )
}

// api

fn json_error(status: Status, message: String) -> status::Custom<Json<Value>> {
    status::Custom(status, Json(json!({ "error": message })))
}

#[post("/api/copilot", format = "json", data = "<data>")]
fn copilot_endpoint(
    data: Json<copilot::Request>,
) -> Result<Json<Value>, status::Custom<Json<Value>>> {
    let copilot::Request {
        prompt,
        max_tokens,
        temperature,
        top_p,
    } = data.into_inner();
    let temperature = temperature.unwrap_or(1.0);
    let top_p = top_p.unwrap_or(0.9);

    match notify::notify(
        format!("copilot request with prompt: {}", prompt.clone()).as_str(),
        "airplane",
    ) {
        Ok(_) => (),
        Err(e) => return Err(json_error(Status::InternalServerError, e.to_string())),
    }

    let output = copilot::get_copilot(prompt, max_tokens, temperature, top_p);
    match output {
        Ok(output) => Ok(Json(json!({ "ok": true, "output": output }))),
        Err(_) => Err(json_error(Status::BadRequest, "...".to_string())),
    }
}

#[get("/api/poll/get/<poll_id>")]
fn get_poll_endpoint(poll_id: String) -> Result<Json<Value>, status::Custom<Json<Value>>> {
    match poll::get_poll(poll_id) {
        Ok(Some(poll)) => Ok(Json(json!({ "ok": true, "poll": poll }))),
        Ok(None) => Err(json_error(Status::NotFound, "...".to_string())), // TODO placeholder
        Err(_) => Err(json_error(Status::InternalServerError, "...".to_string())), // TODO placeholder
    }
}

#[post("/api/poll/create", format = "json", data = "<options>")]
fn new_poll_endpoint(options: Json<Vec<String>>) -> Json<Value> {
    let poll_id = poll::create_poll(options.into_inner());
    notify::notify(
        format!(
            "new poll with url: https://celeste.exposed/poll/{}",
            poll_id
        )
        .as_str(),
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
    match poll::vote_poll(poll_id, option, fingerprint) {
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
    match poll::voted_on(fingerprint, poll_id) {
        Ok(voted) => Json(json!({ "ok": true, "voted": voted })),
        Err(e) => Json(json!({ "ok": false, "error": format!("{}", e) })),
    }
}

#[get("/poll/<poll_id>")]
fn poll_page_with_id(poll_id: String) -> Result<Template, Status> {
    let res = poll::get_poll(poll_id.clone());
    match res {
        Ok(Some(data)) => Ok(Template::render("poll", poll::Poll { id: poll_id, data })),
        Ok(None) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

// main

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                css,
                index_js,
                alignment_js,
                copilot_js,
                libre_baskerville_400,
                libre_baskerville_700,
                source_code_pro_400,
                source_code_pro_700,
                favicon,
                index_page,
                about_page,
                ads_page,
                alignment_page,
                poll_page,
                copilot_endpoint,
                get_poll_endpoint,
                new_poll_endpoint,
                vote_poll_endpoint,
                voted_poll_endpoint,
                poll_page_with_id,
            ],
        )
        .register("/", catchers![on_404])
        .attach(Template::fairing())
        .attach(Shield::default().disable::<Frame>())
}
