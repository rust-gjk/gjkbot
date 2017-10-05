extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use reqwest::Client;
use reqwest::header::{Accept, qitem};

use std::process::exit;

#[derive(Deserialize, Clone, Debug)]
struct Team {
    name: String,
    slug: String,
    id: i32,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Repository {
    name: String,
    description: Option<String>,
    id: i32,
}

const PREFIX: &'static str = "s_";
const ORG_NAME: &'static str = "rust-gjk";
const TEAM_NAME: &'static str = "rok-2017-2018"; //actually a slug
const AUTH_TOKEN: &'static str = "lol nope";

fn main() {
    let team = Client::new()
        .get(&format!("https://api.github.com/orgs/{}/teams", ORG_NAME))
        .basic_auth(AUTH_TOKEN, Some("x-oauth-basic"))
        .header(Accept(vec![qitem("application/vnd.github.v3+json".parse().expect("potato"))]))
        .send()
        .expect("couldn't get id")
        .json::<Vec<Team>>();

    let id = match team {
        Ok(t) => {
            match t.iter().find(|x| x.slug == TEAM_NAME) {
                Some(n) => n.id,
                None => {
                    println!("err: team not found");
                    exit(-1)
                }
            }
        }
        Err(e) => {
            println!("err: {}", e);
            exit(-1)
        }
    };

    println!("id: {}", id);

    let moved_repos = match Client::new()
              .get(&format!("https://api.github.com/teams/{}/repos", id))
              .basic_auth(AUTH_TOKEN, Some("x-oauth-basic"))
              .header(Accept(vec![qitem("application/vnd.github.v3+json"
                                            .parse()
                                            .expect("potato"))]))
              .send()
              .expect("why no repo")
              .json::<Vec<Repository>>() {
        Ok(t) => t,
        Err(e) => {
            println!("err: {}", e);
            exit(-1)
        }
    };

    let repos = match Client::new()
              .get(&format!("https://api.github.com/orgs/{}/repos", ORG_NAME))
              .basic_auth(AUTH_TOKEN, Some("x-oauth-basic"))
              .header(Accept(vec![qitem("application/vnd.github.v3+json"
                                            .parse()
                                            .expect("potato"))]))
              .send()
              .expect("why no repo")
              .json::<Vec<Repository>>() {
        Ok(t) => t,
        Err(e) => {
            println!("err: {}", e);
            exit(-1)
        }
    };

    for repo in repos
            .iter()
            .filter(|x| x.name.starts_with(PREFIX) && !moved_repos.contains(x)) {

        println!("moving {}", &repo.name);
        let res = Client::new()
            .put(&format!("https://api.github.com/teams/{}/repos/{}/{}",
                          id,
                          ORG_NAME,
                          &repo.name))
            .basic_auth(AUTH_TOKEN, Some("x-oauth-basic"))
            .header(Accept(vec![qitem("application/vnd.github.v3+json".parse().expect("potato"))]))
            .body("{ \"permission\":\"push\" }")
            .send();

        if let Err(e) = res {
            println!("err: {}", e);
        }
    }
}
