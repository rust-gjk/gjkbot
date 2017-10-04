extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use reqwest::{Client};
use reqwest::header::{Accept, qitem};

use std::process::exit;

#[derive(Deserialize, Clone, Debug)]
struct Team
{
	name: String,
	id: i32,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Repository
{
	name: String,
	id: i32
}

const ORG_NAME: &'static str = "rust-gjk";
const TEAM_NAME: &'static str = "rok-2017-2018";
const AUTH_TOKEN: &'static str = "lol nope";

fn main()
{
	let team = Client::new()
		.get(&format!("https://api.github.com/orgs/{}/teams", ORG_NAME))
		.basic_auth(AUTH_TOKEN, Some("x-oauth-basic"))
		.header(Accept(vec![qitem("application/vnd.github.v3+json".parse().expect("potato"))]))
		.send()
		.expect("why no team")
		.json::<Vec<Team>>();

	let id = match team
	{
		Ok(t) => match t.iter().find(|ref x| x.name == TEAM_NAME.to_string())
		{
			Some(ref n) => n.id,
			None => {println!("err: team not found"); exit(-1)}
		}
		Err(e) => { println!("err: {}", e); exit(-1)}
	};

	println!("id: {}", id);

	let moved_repos = match Client::new()
		.get(&format!("https://api.github.com/teams/{}/repos", id))
		.basic_auth(AUTH_TOKEN, Some("x-oauth-basic"))
		.header(Accept(vec![qitem("application/vnd.github.v3+json".parse().expect("potato"))]))
		.send()
		.expect("why no repo")
		.json::<Vec<Repository>>()
	{
		Ok(t) => t,
		Err(e) => { println!("err: {}", e); exit(-1); }
	};

	let repos = match Client::new()
		.get(&format!("https://api.github.com/orgs/{}/repos", ORG_NAME))
		.basic_auth(AUTH_TOKEN, Some("x-oauth-basic"))
		.header(Accept(vec![qitem("application/vnd.github.v3+json".parse().expect("potato"))]))
		.send()
		.expect("why no repo")
		.json::<Vec<Repository>>()
	{
		Ok(t) => t,
		Err(e) => { println!("err: {}", e); exit(-1); }
	};

	for repo in repos
	{
		if repo.name.starts_with("s_") && !moved_repos.contains(&repo)
		{
			println!("moving {}", &repo.name);
			let res = Client::new()
				.put(&format!("https://api.github.com/teams/{}/repos/{}/{}", id, ORG_NAME, &repo.name))
				.basic_auth(AUTH_TOKEN, Some("x-oauth-basic"))
				.header(Accept(vec![qitem("application/vnd.github.v3+json".parse().expect("potato"))]))
				.body("{ \"permission\":\"push\" }")
				.send();

			if let Err(e) = res
			{
				println!("err: {}", e);
			}
		}
	}
}
