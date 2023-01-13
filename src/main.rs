extern crate reqwest;
extern crate serde;
extern crate serde_json;

use serde::Deserialize;

use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use reqwest::Response;
use reqwest::blocking::Client;


#[derive(Deserialize)]
struct Repo {
    name: String,
    description: String,
    language: String,
}

#[derive(Deserialize)]
struct Team {
    repos: Vec<Repo>,
}

pub fn get_repos(url: &str, client: &Client, token: &str) -> Response {
    let res = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()?;

    let mut res = match res {
        Ok(res) => res,
        Err(err) => {
            println!("Error making request to GitHub API: {}", err);
        }
    };
    if !res.status().is_success() {
        let error_text = res.text().unwrap_or("unknown error".to_string());
        println!("Error: {}", error_text);
    }
    return res;
}

fn main() {
    let team_name = "team-name";
    let token = "your-token";
    let client = Client::new();
    let mut repos = HashSet::new();

    let url = format!("https://api.github.com/teams/{}/repos", team_name);

    let repo_response = get_repos(&url, &client, &token);


    let team: Team = match repo_response.json() {
        Ok(team) => team,
        Err(err) => {
            println!("Error parsing JSON response: {}", err);
            return;
        }
    };

    for repo in team.repos {
        repos.insert(repo.name);
    }

    let report = format!(
        "<html>
        <head>
            <title>{} Repositories</title>
        </head>
        <body>
            <h1>{} Repositories</h1>
            <table>
                <tr>
                    <th>Repository Name</th>
                </tr>
                {}
            </table>
        </body>
    </html>",
        team_name,
        team_name,
        repos
            .iter()
            .map(|name| {
                format!(
                    "<tr>
                    <td>{}</td>
                </tr>",
                    name
                )
            })
            .collect::<String>()
    );

    match File::create("report.html") {
        Ok(mut file) => {
            match file.write_all(report.as_bytes()) {
                Ok(_) => println!("Report generated successfully!"),
                Err(err) => println!("Error writing to report file: {}", err),
            }
        }
        Err(err) => {
            println!("Error creating report file: {}", err);
            return;
        }
    };
}