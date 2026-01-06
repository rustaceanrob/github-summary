use chrono::{DateTime, Months, Utc};
use octocrab::{Octocrab, models::IssueState};
use serde::Deserialize;
use std::{fs::File, io::BufReader, sync::Arc};

#[derive(Deserialize, Debug)]
struct Query {
    username: String,
    repositories: Vec<(String, String)>,
}

#[tokio::main]
async fn main() {
    let quarter = Utc::now() - Months::new(3);
    let file = File::open("./config.json").expect("could not find config.json");
    let buf_reader = BufReader::new(file);
    let query: Query = serde_json::from_reader(buf_reader).unwrap();
    let octocrab = octocrab::instance();
    for (owner, repo) in query.repositories {
        build_repository_summary(
            octocrab.clone(),
            quarter,
            query.username.clone(),
            owner,
            repo,
        )
        .await;
    }
}

async fn build_repository_summary(
    octo: Arc<Octocrab>,
    quarter: DateTime<Utc>,
    username: String,
    owner: String,
    repo: String,
) {
    let prs = octo
        .pulls(owner, repo)
        .list()
        .per_page(100)
        .send()
        .await
        .unwrap();
    let filtered_on_user = prs
        .into_iter()
        .filter(|pr| pr.user.is_some())
        .filter(|pr| pr.user.clone().unwrap().login == username)
        .filter(|pr| pr.created_at.is_some())
        .filter(|pr| pr.created_at.unwrap() > quarter);
    for pr in filtered_on_user {
        if let Some(text) = pr.title {
            println!("{text} #{}", pr.number);
        }
        if let Some(text) = pr.body {
            println!("{text}");
        }
        if let Some(state) = pr.state {
            match state {
                IssueState::Open => println!("Open"),
                IssueState::Closed => println!("Closed"),
                _ => (),
            }
        }
        if let Some(comments) = pr.comments {
            println!("Comments {comments}");
        }
        if let Some(mergeable) = pr.mergeable_state {
            println!("Merge status {mergeable:?}");
        }
    }
}
