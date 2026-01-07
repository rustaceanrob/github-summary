use chrono::{DateTime, Months, Utc};
use octocrab::{Octocrab, models::pulls::PullRequest};
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use serde::Deserialize;
use std::{fs::File, io::BufReader, sync::Arc};

#[derive(Deserialize, Debug)]
struct Query {
    username: String,
    description: String,
    repositories: Vec<(String, String)>,
    model: String,
}

#[tokio::main]
async fn main() {
    let quarter = Utc::now() - Months::new(1);
    let file = File::open("./config.json").expect("could not find config.json");
    let buf_reader = BufReader::new(file);
    let query: Query = serde_json::from_reader(buf_reader).unwrap();
    let ollama = Ollama::default();
    let mut prompt = String::new();
    print_green(format!(
        "Generating summary for {} beginning {}",
        query.username, quarter
    ));
    println!(" ");
    let octocrab = octocrab::instance();
    for (owner, repo) in query.repositories.iter() {
        build_commit_summary(
            octocrab.clone(),
            quarter,
            &mut prompt,
            query.username.clone(),
            owner.as_str(),
            repo.as_str(),
        )
        .await;
        println!(" ");
        build_pr_summary(
            octocrab.clone(),
            quarter,
            &mut prompt,
            query.username.clone(),
            owner.as_str(),
            repo.as_str(),
        )
        .await;
        println!(" ");
    }
    println!(" ");
    print_green("Generating your response. This may take a few minutes.");
    let mut request = GenerationRequest::new(query.model.clone(), prompt);
    request = request.system(build_context(&query));
    request = request.think(false);
    let res = ollama.generate(request).await.unwrap();
    println!("{}", res.response);
}

fn build_context(query: &Query) -> String {
    format!(
        "You are an assistant that generates monthly reports for open source software developers. You will be given a list of commits, pull requests, and potentially more information to use in generating your report. Your job is to bolster the developer and create a cohesive theme for their work. Your response should be approximately one page. Here is a self-description of the developer: {}",
        query.description
    )
}

async fn build_commit_summary(
    octo: Arc<Octocrab>,
    quarter: DateTime<Utc>,
    ctx: &mut String,
    username: String,
    owner: &str,
    repo: &str,
) {
    let summary_str = format!("Merged commits in {owner}/{repo}");
    print_green(&summary_str);

    let mut commits = octo
        .repos(owner, repo)
        .list_commits()
        .since(quarter)
        .author(username)
        .send()
        .await
        .unwrap();
    let commit_iter = commits.take_items();
    if commit_iter.is_empty() {
        println!("None since {quarter}");
        return;
    } else {
        ctx.push_str(&summary_str);
        ctx.push_str("These are commits that have been merged in the past month. Please summarize them and find relationships between them.");
    }
    for commit in commit_iter {
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
        ctx.push_str(&commit.commit.message);
    }
}

async fn build_pr_summary(
    octo: Arc<Octocrab>,
    quarter: DateTime<Utc>,
    ctx: &mut String,
    username: String,
    owner: &str,
    repo: &str,
) {
    let summary_str = format!("Open pull requests for {owner}/{repo}");
    print_green(&summary_str);
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
        .filter(|pr| pr.updated_at.unwrap() > quarter)
        .collect::<Vec<PullRequest>>();
    if filtered_on_user.is_empty() {
        println!("None since {quarter}");
        return;
    } else {
        ctx.push_str(&summary_str);
        ctx.push_str("These are pull requests that were opened recently. Describe the changes from a high level.");
    }
    for pr in filtered_on_user {
        if let Some(text) = pr.title {
            let title_str = format!("{text} #{}", pr.number);
            println!("{title_str}");
            ctx.push_str(format!("PR title and number: {title_str}").as_str());
        }
        if let Some(body) = pr.body {
            ctx.push_str(format!("PR description: {body}").as_str());
        }
    }
}

fn print_green(str: impl AsRef<str>) {
    println!("\x1b[32m{}\x1b[0m", str.as_ref());
}
