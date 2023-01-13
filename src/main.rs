extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate ini;

use serde::Deserialize;

use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use ini::Ini;

#[derive(Debug)]
#[derive(Deserialize)]
struct Repo {
    name: String,
}

fn fetch_repositories(user_name: &str, access_token: &str) -> Result<Vec<Repo>, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.github.com/users/{}/repos", user_name);
    println!("{}", url);
    let response = client
        .get(&url)
        //.header("Authorization", format!("Token {}", access_token))
        .header("User-Agent", "request")
        .send()?;
    println!("{:?}", response);
    let repos: Vec<Repo> = response.json()?;
    println!("{:?}", repos);
    Ok(repos)
}

fn main() {

    //Read and parse config ini file
    let conf = Ini::load_from_file("config.ini").unwrap();
    let section = conf.section(Some("Github")).unwrap();
    let user_name = section.get("name").unwrap();
    let access_token = section.get("token").unwrap();

    let repositories = match fetch_repositories(user_name, access_token) {
        Ok(repositories) => repositories,
        Err(err) => { panic!("Error fetching repositories: {}", err); }
    };

    let repository_names: HashSet<String> = repositories.into_iter().map(|r| r.name).collect();

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
        user_name,
        user_name,
        repository_names
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