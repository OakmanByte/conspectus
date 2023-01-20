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
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_json::json;
use url::Url;
use std::env;
use rayon::prelude::*;


#[derive(Debug, Deserialize, Serialize)]
//Used for storing the data we get back from Github when we fetch repositories for a user/team
struct GithubRepo {
    name: String,
    html_url: String,
    //Github api returns either the most used language or can return null if there is no identifiable language, therefore we need to use Option.
    language: Option<String>,
    archived: bool,
    pushed_at: String,
}

//Used to store additional repository information in addition to what is stored in the repo field.
#[derive(Debug, Deserialize, Serialize)]
struct CustomRepo {
    repo: GithubRepo,
    dependabot_exists: bool,
    number_of_open_pull_requests: u16,
}

//Combination of the GithubRepo & CustomRepo struct for the finalized struct that has all repository data that we can use for later operations.
#[derive(Debug, Deserialize, Serialize)]
struct FullRepo {
    name: String,
    html_url: String,
    language: String,
    archived: bool,
    pushed_at: String,
    dependabot_exists: bool,
    number_of_open_pull_requests: u16,
}

#[derive(PartialEq)]
enum Mode {
    User,
    Org,
}

fn get_number_of_open_pull_requests(client: &Client, mode: &Mode, user_name: &str, org: &str, access_token: &str, repository_name: &str) -> Result<u16, Box<dyn std::error::Error>> {
    let url: Url;
    if *mode == Mode::User {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/pulls?state=open", user_name, repository_name))?;
    } else {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/pulls?state=open", org, repository_name))?;
    }

    let response = client
        .get(url)
        .header("Authorization", format!("token {}", access_token))
        .header("User-Agent", "conspectus/1.0")
        .send()?;

    let json: serde_json::Value = response.json()?;
    if json.as_array().is_none() {
        return Ok(0);
    }
    Ok(json.as_array().unwrap().len() as u16)
}

fn dependabot_file_exists(client: &Client, mode: &Mode, user_name: &str, org: &str, access_token: &str, repository_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let url: Url;
    if *mode == Mode::User {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/contents/.github/dependabot.yml", user_name, repository_name))?;
    } else {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/contents/.github/dependabot.yml", org, repository_name))?;
    }

    let response = client
        .get(url)
        .header("Authorization", format!("token {}", access_token))
        .header("User-Agent", "conspectus/1.0")
        .send()?;

    match response.status() {
        StatusCode::OK => Ok(true),
        StatusCode::NOT_FOUND => Ok(false),
        s => {
            println!("Strange status code found when getting dependabot contents, status: {:?}", s);
            Ok(false)
        }
    }
}

fn fetch_repositories(client: &Client, mode: &Mode, org: &str, team_name: &str, access_token: &str, include_archived: bool) -> Result<Vec<GithubRepo>, Box<dyn std::error::Error>> {
    let url: Url;
    if *mode == Mode::User {
        url = Url::parse("https://api.github.com/user/repos")?;
    } else {
        url = Url::parse(&*format!("https://api.github.com/orgs/{}/teams/{}/repos?per_page=100", org, team_name))?;
    }
    let response = client
        .get(url)
        .header("Authorization", format!("token {}", access_token))
        .header("User-Agent", "conspectus/1.0")
        .send()?;

    let mut repos: Vec<GithubRepo> = response.json()?;

    //Remove archived repositories if they shouldn't be included
    // Can't write !repo.archived for some reason, must be syntax specific
    if !include_archived {
        repos.retain(|repo| repo.archived == false)
    }
    Ok(repos)
}

fn generate_report(user_name: &str, repositories: Vec<CustomRepo>) -> Result<(), Box<dyn std::error::Error>> {
    let full_repositories: Vec<FullRepo> = repositories.into_iter().map(|r| {
        FullRepo {
            name: r.repo.name,
            html_url: r.repo.html_url,
            language: r.repo.language.unwrap_or("None".parse().unwrap()),
            archived: r.repo.archived,
            pushed_at: r.repo.pushed_at,
            dependabot_exists: r.dependabot_exists,
            number_of_open_pull_requests: r.number_of_open_pull_requests,
        }
    }).collect();

    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file("table", "table.html")
        .expect("Failed to register template file");
    let report = handlebars
        .render("table", &json!({
         "user_name": user_name,
         "repositories": full_repositories,
     }))
        .expect("Failed to render template");

    let mut file = File::create("report.html")?;
    file.write_all(report.as_bytes())?;
    println!("Successfully generate report!");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut org_name: &str = "";
    let mut team_name: &str = "";
    let mut user_name: &str = "";

    let mode: String = match args.iter().position(|arg| arg == "--mode" || arg == "-mode") {
        Some(i) => args.get(i + 1),
        None => None
    }.expect("No mode was provided, provide either user or org").to_string();

    let selected_mode: Mode = match mode.to_lowercase().as_str() {
        "user" => Mode::User,
        "org" => Mode::Org,
        _ => return Err(From::from(format!("unsupported mode was given: {}, supported modes are user or org", mode)))
    };

    println!("THIS:{}", mode);

    //Necessary in both modes
    let config = Ini::load_from_file("config.ini")?;
    let section = config.section(Some("Github")).ok_or_else(|| "Failed to find Github section in config file")?;
    let access_token = section.get("token").ok_or_else(|| "Failed to find access_token in config file")?;

    //Required config fields for org mode
    if selected_mode == Mode::Org {
        org_name = section.get("organization_name").ok_or_else(|| "Failed to find organization_name in config file")?;
        team_name = section.get("team_name").ok_or_else(|| "Failed to find team_name in config file")?;
    }
    //Required config field for user mode
    else {
        user_name = section.get("user_name").ok_or_else(|| "Failed to find user_name in config file")?;
    }


    let client = Client::new();
    let repositories = fetch_repositories(&client, &selected_mode, org_name, team_name, access_token, false)?;
    let mut custom_repos: Vec<CustomRepo> = repositories.into_iter().map(|r| {
        CustomRepo {
            repo: r,
            dependabot_exists: false,
            number_of_open_pull_requests: 0,
        }
    }).collect();

    custom_repos.par_iter_mut().for_each(|repo| {
        repo.repo.pushed_at = repo.repo.pushed_at.split("T").next().unwrap_or("").to_string();
        repo.dependabot_exists = match dependabot_file_exists(&client, &selected_mode, user_name, org_name, access_token, &repo.repo.name) {
            Ok(exists) => exists,
            Err(error) => {
                println!("Error: {:?}", error);
                false
            }
        };
        repo.number_of_open_pull_requests = match get_number_of_open_pull_requests(&client, &selected_mode, user_name, org_name, access_token, &repo.repo.name) {
            Ok(num) => num,
            Err(error) => {
                println!("Error: {:?}", error);
                0
            }
        };
    });
    generate_report(user_name, custom_repos)
}
