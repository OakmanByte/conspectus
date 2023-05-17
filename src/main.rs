extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate ini;
extern crate prettytable;


use prettytable::{Table, row};
use serde::Deserialize;
use serde::Serialize;
use ini::Ini;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use url::Url;
use std::env;
use rayon::prelude::*;
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
//Used for storing the data we get back from Github when we fetch repositories for a user/team
pub struct GithubRepo {
    name: String,
    html_url: String,
    //Github api returns either the most used language or can return null if there is no identifiable language, therefore we need to use Option.
    language: Option<String>,
    archived: bool,
    pushed_at: String,
}

//Used to store additional repository information in addition to what is stored in the repo field.
#[derive(Debug, Deserialize, Serialize)]
pub struct CustomRepo {
    repo: GithubRepo,
    dependabot_exists: bool,
    number_of_open_pull_requests: u16,
    codeowners_exists: bool,
    dependabot_alerts: u16,
    secret_scanning_alerts: u16,
    code_scanning_alerts: u16,
}

//Combination of the GithubRepo & CustomRepo struct for the finalized struct that has all repository data that we can use for later operations.
#[derive(Debug, Deserialize, Serialize)]
pub struct FullRepo {
    name: String,
    html_url: String,
    language: String,
    archived: bool,
    pushed_at: String,
    dependabot_exists: bool,
    codeowners_exists: bool,
    dependabot_alerts: u16,
    secret_scanning_alerts: u16,
    code_scanning_alerts: u16,
    number_of_open_pull_requests: u16,
}


#[derive(PartialEq)]
enum Mode {
    User,
    Org,
}

const AUTHORIZATION: &str = "Authorization";
const USER_AGENT: (&str, &str) = ("User-Agent", "conspectus/1.0");


fn get_number_of_open_pull_requests(client: &Client, mode: &Mode, user_name: &str, org: &str, access_token: &str, repository_name: &str) -> Result<u16, Box<dyn std::error::Error>> {
    let url: Url;
    if *mode == Mode::User {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/pulls?state=open", user_name, repository_name))?;
    } else {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/pulls?state=open", org, repository_name))?;
    }

    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("token {}", access_token))
        .header(USER_AGENT.0, USER_AGENT.1)
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
        .header(AUTHORIZATION, format!("token {}", access_token))
        .header(USER_AGENT.0, USER_AGENT.1)
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

fn codeowners_file_exists(client: &Client, mode: &Mode, user_name: &str, org: &str, access_token: &str, repository_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let url: Url;
    if *mode == Mode::User {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/contents/.github/CODEOWNERS", user_name, repository_name))?;
    } else {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/contents/.github/CODEOWNERS", org, repository_name))?;
    }

    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("token {}", access_token))
        .header(USER_AGENT.0, USER_AGENT.1)
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

fn number_of_dependabot_alers(client: &Client, mode: &Mode, user_name: &str, org: &str, access_token: &str, repository_name: &str) -> Result<u16, Box<dyn std::error::Error>> {
    let url: Url;
    if *mode == Mode::User {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/vulnerability-alerts", user_name, repository_name))?;
    } else {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/vulnerability-alerts", org, repository_name))?;
    }

    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("token {}", access_token))
        .header(USER_AGENT.0, USER_AGENT.1)
        .send()?;

    match response.status() {
        StatusCode::OK => Ok(response.json::<Vec<Value>>()?.len() as u16),
        StatusCode::NOT_FOUND => Ok(0),
        s => {
            println!("Strange status code found when getting dependabot contents, status: {:?}", s);
            Ok(0)
        }
    }
}

fn number_of_secret_scan_alerts(client: &Client, mode: &Mode, user_name: &str, org: &str, access_token: &str, repository_name: &str) -> Result<u16, Box<dyn std::error::Error>> {
    let url: Url;
    if *mode == Mode::User {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/secret-scanning/alerts", user_name, repository_name))?;
    } else {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/secret-scanning/alerts", org, repository_name))?;
    }

    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("token {}", access_token))
        .header(USER_AGENT.0, USER_AGENT.1)
        .send()?;

    match response.status() {
        StatusCode::OK => Ok(response.json::<Vec<Value>>()?.len() as u16),
        StatusCode::NOT_FOUND => Ok(0),
        s => {
            println!("Strange status code found when getting dependabot contents, status: {:?}", s);
            Ok(0)
        }
    }
}

fn number_of_code_scan_alerts(client: &Client, mode: &Mode, user_name: &str, org: &str, access_token: &str, repository_name: &str) -> Result<u16, Box<dyn std::error::Error>> {
    let url: Url;
    if *mode == Mode::User {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/code-scanning/alerts", user_name, repository_name))?;
    } else {
        url = Url::parse(&*format!("https://api.github.com/repos/{}/{}/code-scanning/alerts", org, repository_name))?;
    }

    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("token {}", access_token))
        .header(USER_AGENT.0, USER_AGENT.1)
        .send()?;

    match response.status() {
        StatusCode::OK => Ok(response.json::<Vec<Value>>()?.len() as u16),
        StatusCode::NOT_FOUND => Ok(0),
        s => {
            println!("Strange status code found when getting dependabot contents, status: {:?}", s);
            Ok(0)
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
        .header(AUTHORIZATION, format!("token {}", access_token))
        .header(USER_AGENT.0, USER_AGENT.1)
        .send()?;

    let mut repos: Vec<GithubRepo> = response.json()?;

    //Remove archived repositories if they shouldn't be included
    // Can't write !repo.archived for some reason, must be syntax specific
    if !include_archived {
        repos.retain(|repo| repo.archived == false)
    }
    Ok(repos)
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

    println!("Selected mode: {}", mode);

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
            codeowners_exists: false,
            dependabot_alerts: 0,
            secret_scanning_alerts: 0,
            code_scanning_alerts: 0,

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
        repo.codeowners_exists = match codeowners_file_exists(&client, &selected_mode, user_name, org_name, access_token, &repo.repo.name) {
            Ok(exists) => exists,
            Err(error) => {
                println!("Error: {:?}", error);
                false
            }
        };
        repo.dependabot_alerts = match number_of_dependabot_alers(&client, &selected_mode, user_name, org_name, access_token, &repo.repo.name) {
            Ok(exists) => exists,
            Err(error) => {
                println!("Error: {:?}", error);
                0
            }
        };
        repo.secret_scanning_alerts = match number_of_secret_scan_alerts(&client, &selected_mode, user_name, org_name, access_token, &repo.repo.name) {
            Ok(exists) => exists,
            Err(error) => {
                println!("Error: {:?}", error);
                0
            }
        };
        repo.code_scanning_alerts = match number_of_code_scan_alerts(&client, &selected_mode, user_name, org_name, access_token, &repo.repo.name) {
            Ok(exists) => exists,
            Err(error) => {
                println!("Error: {:?}", error);
                0
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
    generate_report(custom_repos)
}

fn generate_report(repositories: Vec<CustomRepo>) -> Result<(), Box<dyn std::error::Error>> {
    let full_repositories: Vec<FullRepo> = repositories.into_iter().map(|r| {
        FullRepo {
            name: r.repo.name,
            html_url: r.repo.html_url,
            language: r.repo.language.unwrap_or("None".parse().unwrap()),
            archived: r.repo.archived,
            pushed_at: r.repo.pushed_at,
            dependabot_exists: r.dependabot_exists,
            codeowners_exists: r.codeowners_exists,
            dependabot_alerts: r.dependabot_alerts,
            secret_scanning_alerts: r.secret_scanning_alerts,
            code_scanning_alerts: r.code_scanning_alerts,
            number_of_open_pull_requests: r.number_of_open_pull_requests,
        }
    }).collect();

    let mut table = Table::new();
    table.add_row(row![
        "Name",
        "URL",
        "Language",
        "Archived",
        "Last Pushed",
        "Dependabot file exists",
        "CODEOWNERS file exists",
        "Dependabot Alerts",
        "Secret Scanning Alerts",
        "Code Scanning Alerts",
        "Open PRs",
    ]);

    for repo in full_repositories {
        let archived_str = if repo.archived { "Yes" } else { "No" };
        let dependabot_str = if repo.dependabot_exists { "Yes" } else { "No" };
        let codeowners_str = if repo.codeowners_exists { "Yes" } else { "No" };
        table.add_row(row![
            repo.name,
            repo.html_url,
            repo.language,
            archived_str,
            repo.pushed_at,
            dependabot_str,
            codeowners_str,
            repo.dependabot_alerts,
            repo.secret_scanning_alerts,
            repo.code_scanning_alerts,
            repo.number_of_open_pull_requests,
        ]);
    }


    table.printstd();
    return Ok(());
}