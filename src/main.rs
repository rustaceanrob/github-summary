use chrono::{DateTime, Months, Utc};
use octocrab::{Octocrab, models::pulls::PullRequest};
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest, models::ModelOptions};
use serde::Deserialize;
use std::{fs::File, io::BufReader, sync::Arc};

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct Query {
    username: String,
    name: String,
    description: String,
    repositories: Vec<(String, String)>,
    model: String,
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let use_llm = args.next().is_none();
    let quarter = Utc::now() - Months::new(3);
    let file = File::open("./config.json").expect("could not find config.json");
    let buf_reader = BufReader::new(file);
    let query: Query = serde_json::from_reader(buf_reader).unwrap();
    print_green(format!(
        "Generating summary for {} beginning {}",
        query.username, quarter
    ));
    println!(" ");
    let initial_description = format!("The developer's name is {}.\n", query.name.clone());
    let octocrab = octocrab::instance();
    for (owner, repo) in query.repositories.iter() {
        println!("Summarizing {owner}/{repo}");
        let mut prompt = String::new();
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
        if use_llm && !prompt.is_empty() {
            let mut prefix = initial_description.clone();
            prefix.push_str(&prompt);
            print_green("Generating your response. This may take a few minutes.");
            create_repo_summary(&query, prefix, owner.as_str(), repo.as_str()).await;
        }
    }
    println!(" ");
    print_green("Done!");
}

fn build_context(_query: &Query) -> String {
    "You are an assistant that generates quaterly reports for open source software developers. You will be given a list of commits, pull requests, and potentially more information about contributions to a repository. Your job is to bolster the developer and create a cohesive theme for their work. Your response should be approximately one paragraph. Be as concise as possible, the reader has limited time.".to_string()
}

async fn create_repo_summary(query: &Query, mut prompt: String, owner: &str, repo: &str) -> String {
    let prefix_string = format!(
        "Use the above information for {owner}/{repo} to build a summary. Avoid preambles like 'here is a summary' and delve directly into the material. Emphasize high-potential PRs and commits, along with useful review. Mention refactors, but do not over-state importance.\n"
    );
    prompt.push_str(&prefix_string);
    let ollama = Ollama::default();
    let mut model_opts = ModelOptions::default();
    model_opts = model_opts.repeat_penalty(1.5);
    model_opts = model_opts.num_ctx(16384);
    let mut request = GenerationRequest::new(query.model.clone(), prompt);
    request = request.system(build_context(query));
    request = request.options(model_opts);
    let res = ollama.generate(request).await.unwrap();
    println!("{}\n", res.response);
    res.response
}

async fn build_commit_summary(
    octo: Arc<Octocrab>,
    quarter: DateTime<Utc>,
    ctx: &mut String,
    username: String,
    owner: &str,
    repo: &str,
) {
    let summary_str = format!("Merged commits in {owner}/{repo}\n");
    print_green(&summary_str);
    for page in 1..5u32 {
        let mut commits = octo
            .repos(owner, repo)
            .list_commits()
            .per_page(100)
            .page(page)
            .since(quarter)
            .author(&username)
            .send()
            .await
            .unwrap();
        let commit_iter = commits.take_items();
        if commit_iter.is_empty() {
            if page == 1 {
                println!("None since {quarter}");
            }
            return;
        } else if page == 1 {
            ctx.push_str(&summary_str);
            ctx.push_str("These are commits that have been merged in the past month. Use their messages as context for the summary.");
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
            ctx.push_str(format!("{}\n", commit.commit.message).as_str());
        }
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
    let summary_str = format!("Open pull requests for {owner}/{repo}\n");
    print_green(&summary_str);
    for page in 1..5u32 {
        let prs = octo
            .pulls(owner, repo)
            .list()
            .per_page(100)
            .page(page)
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
            if page == 1 {
                println!("None since {quarter}");
            }
            return;
        } else if page == 1 {
            ctx.push_str(&summary_str);
            ctx.push_str("These are pull requests that were opened recently. Describe the changes from a high level.");
        }
        for pr in filtered_on_user {
            if let Some(text) = pr.title {
                let title_str = format!("{text} #{}", pr.number);
                println!("{title_str}");
                ctx.push_str(format!("PR title and number: {title_str}\n").as_str());
            }
            if let Some(body) = pr.body {
                ctx.push_str(format!("PR description: {body}\n").as_str());
            }
        }
    }
}

fn print_green(str: impl AsRef<str>) {
    println!("\x1b[32m{}\x1b[0m", str.as_ref());
}
