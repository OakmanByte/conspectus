extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate ini;

use serde::Deserialize;
use serde::Serialize;
use handlebars::Handlebars;

use std::fs::File;
use std::io::prelude::*;
use ini::Ini;
use serde_json::json;

#[derive(Debug, Deserialize, Serialize)]
struct Repo {
    name: String,
    html_url: String,
    language: String,
}

fn fetch_repositories(user_name: &str, _access_token: &str) -> Result<Vec<Repo>, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.github.com/users/{}/repos", user_name);
    let response = client
        .get(&url)
        //.header("Authorization", format!("Token {}", access_token))
        .header("User-Agent", "conspectus/1.0")
        .send()?;
    let repos: Vec<Repo> = response.json()?;
    Ok(repos)
}

fn generate_report(user_name: &str, repositories: Vec<Repo>) -> Result<(), Box<dyn std::error::Error>> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file("table", "table.html")
        .expect("Failed to register template file");
    let report = handlebars
        .render("table", &json!({
        "user_name": user_name,
        "repositories": repositories,
    }))
        .expect("Failed to render template");

    let mut file = File::create("report.html")?;
    file.write_all(report.as_bytes())?;
    println!("Successfully generate report!");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    //Read and parse CONFIG ini file
    let config = Ini::load_from_file("config.ini")?;
    let section = config.section(Some("Github")).ok_or("Failed to find Github section in config file")?;
    let user_name = section.get("name").ok_or("Failed to find user_name in config file")?;
    let access_token = section.get("token").ok_or("Failed to find access_token in config file")?;

    if user_name.is_empty() || access_token.is_empty() {
        return Err(From::from("Missing username or access_token in the config file. Please see Readme file on how to setup the config file correctly."));
    }

    let repositories = fetch_repositories(user_name, access_token)?;
    generate_report(user_name, repositories)
}
