extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate ini;

use serde::Deserialize;

use std::fs::File;
use std::io::prelude::*;
use ini::Ini;

#[derive(Debug)]
#[derive(Deserialize)]
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
    let report = format!(
        "<!DOCTYPE html>
        <html>
        <style>
        table, th, td {{
          border:1px solid black;
        }}
        </style>
        <body>

        <h2>{} Repositories</h2>
            <table>
                <tr>
                    <th>Name</th>
                    <th>Url</th>
                    <th>Language</th>
                </tr>
                {}
            </table>
        </body>
        </html>",
        user_name,
        repositories
            .into_iter()
            .map(|repo| {
                format!(
                    "<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    </tr>",
                    repo.name,
                    repo.html_url,
                    repo.language
                )
            })
            .collect::<String>()
    );

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
