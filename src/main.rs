extern crate reqwest;
#[macro_use]
extern crate serde_derive;

use reqwest::Client;
use reqwest::header::{Accept, qitem};

use std::io::Write;
use std::process::{exit, Command, Stdio};

#[derive(Deserialize)]
struct Team {
    name: String,
    slug: String,
    id: i32,
}

#[derive(Deserialize, PartialEq)]
struct Repository {
    name: String,
    description: Option<String>,
    html_url: String,
    has_issues: bool,
    id: i32,
}

#[derive(Deserialize)]
struct Topics {
    names: Vec<String>,
}

#[derive(Serialize)]
struct Issue {
    title: String,
}

const ORG_NAME: &'static str = "rust-gjk";
const TEAM_NAME: &'static str = "rok-2017-2018"; //actually slug of the team
const AUTH_TOKEN: &'static str = include!("auth_token");
const WHITE_LIST: [&'static str; 6] = ["znamkovani", "gjkbot", "rustgrade", "Materialy", "Halp", "snapshot"];

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
            println!("{} err: {}", line!(), e);
            exit(-1)
        }
    };

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
            println!("{} err: {}", line!(), e);
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
            println!("{} err: {}", line!(), e);
            exit(-1)
        }
    };

    // move repositories to team
    for repo in
        repos
            .iter()
            .filter(|x| WHITE_LIST.contains(&x.name.as_ref()) && !moved_repos.contains(x)) {

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
            println!("{} err: {}", line!(), e);
        }
    }

    // check for handed in repositories - 'r' topic
    // 'n' topic makes the repo ignored and topics et al. unchanged
    // 'o' means handed in and recognized by the bot
    for repo in moved_repos {
        let topics = match Client::new()
                  .get(&format!("https://api.github.com/repos/{}/{}/topics",
                                ORG_NAME,
                                &repo.name))
                  .basic_auth(AUTH_TOKEN, Some("x-oauth-basic"))
                  .header(Accept(vec![qitem("application/vnd.github.mercy-preview+json"
                                                .parse()
                                                .expect("potato"))]))
                  .send()
                  .expect("why no topic")
                  .json::<Topics>() {
            Ok(t) => t,
            Err(e) => {
                println!("{} err: {}", line!(), e);
                exit(-1)
            }
        };

        let topics = topics.names;

        if topics.contains(&"r".to_string()) && !topics.contains(&"n".to_string()) {
            println!("handing in: {}", &repo.name);

            let res = Client::new()
                .put(&format!("https://api.github.com/repos/{}/{}/topics",
                              ORG_NAME,
                              &repo.name))
                .basic_auth(AUTH_TOKEN, Some("x-oauth-basic"))
                .header(Accept(vec![qitem("application/vnd.github.mercy-preview+json"
                                              .parse()
                                              .expect("potato"))]))
                .body("{ \"names\": [\"o\"] }")
                .send();

            if let Err(e) = res {
                println!("{} err: {}", line!(), e);
                exit(-1)
            }

            let msg = &format!("Subject: Úkol odevzdán: {}\
	         \n\nRepozitář byl právě označen jako odevzdaný.\
	           \nURL:{}\n",
                               &repo.name,
                               &repo.html_url);
            let p = Command::new("ssmtp")
                .arg("luk.hozda@gmail.com")
                .stdin(Stdio::piped())
                .spawn()
                .expect("why no ssmtp");

            if let Err(e) = p.stdin.unwrap().write_all(msg.as_bytes()) {
                println!("{} err: {}", line!(), e);
                exit(-1)
            }

            Client::new()
                .post(&format!("https://api.github.com/repos/{}/{}/issues",
                               ORG_NAME,
                               &repo.name))
                .basic_auth(AUTH_TOKEN, Some("x-oauth-basic"))
                .header(Accept(vec![qitem("application/vnd.github.mercy-preview+json"
                                              .parse()
                                              .expect("potato"))]))
                .json(&Issue { title: "odevzdáno".to_string() })
                .send()
                .expect("why no issue");
        }
    }
}
