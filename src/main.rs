use chrono::{DateTime, Months, Utc};
use octocrab::Octocrab;
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
    print_green(format!(
        "Generating summary for {} beginning {}",
        query.username, quarter
    ));
    println!(" ");
    let octocrab = octocrab::instance();
    for (owner, repo) in query.repositories {
        build_commit_summary(
            octocrab.clone(),
            quarter,
            query.username.clone(),
            owner.as_str(),
            repo.as_str(),
        )
        .await;
        println!(" ");
        build_pr_summary(
            octocrab.clone(),
            quarter,
            query.username.clone(),
            owner.as_str(),
            repo.as_str(),
        )
        .await;
        println!(" ");
    }
}

async fn build_commit_summary(
    octo: Arc<Octocrab>,
    quarter: DateTime<Utc>,
    username: String,
    owner: &str,
    repo: &str,
) {
    print_green(format!("Merged commits summary for {owner}/{repo}"));
    let commits = octo
        .repos(owner, repo)
        .list_commits()
        .since(quarter)
        .author(username)
        .send()
        .await
        .unwrap();
    for commit in commits {
        let first_line = commit
            .commit
            .message
            .lines()
            .next()
            .expect("commit messages must be at least one line");
        if first_line.contains("Merge ") {
            continue;
        }
        println!("{first_line}");
    }
}

async fn build_pr_summary(
    octo: Arc<Octocrab>,
    quarter: DateTime<Utc>,
    username: String,
    owner: &str,
    repo: &str,
) {
    print_green(format!("Open pull request summary for {owner}/{repo}"));
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
        .filter(|pr| pr.updated_at.is_some())
        .filter(|pr| pr.updated_at.unwrap() > quarter);
    for pr in filtered_on_user {
        if let Some(text) = pr.title {
            println!("{text} #{}", pr.number);
        }
        if let Some(comments) = pr.comments {
            println!("Comments {comments}");
        }
        if let Some(mergeable) = pr.mergeable_state {
            println!("Merge status {mergeable:?}");
        }
    }
}

fn print_green(str: impl AsRef<str>) {
    println!("\x1b[32m{}\x1b[0m", str.as_ref());
}
