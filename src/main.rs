use chrono::{DateTime, Months, Utc};
use octocrab::{models::{pulls::MergeableState, IssueState, Repository}, params::repos::Sort, Octocrab};
use std::sync::Arc;

const LOGIN: &str = "rustaceanrob";

#[tokio::main]
async fn main() {
    let quarter = Utc::now() - Months::new(3);
    let octocrab = octocrab::instance();
    let user = octocrab.users(LOGIN);
    let repos = user
        .repos()
        .sort(Sort::Updated)
        .per_page(1)
        .send()
        .await
        .unwrap();
    for repo in repos.into_iter() {
        build_repository_summary(octocrab.clone(), repo, quarter).await;
    }
}

async fn build_repository_summary(octo: Arc<Octocrab>, repo: Repository, quarter: DateTime<Utc>) {
    let source = repo.parent.as_deref().unwrap_or(&repo);
    let owner = source.owner.as_ref().unwrap().login.clone();
    let repo_name = source.name.clone();
    println!("Repository {repo_name}");
    println!("Owner {owner}");
    let prs = octo
        .pulls(owner, repo_name)
        .list()
        .per_page(50)
        .send()
        .await
        .unwrap();
    let filtered_on_user = prs
        .into_iter()
        .filter(|pr| pr.user.is_some()) 
        .map(|pr| {
            println!("User login: {}", pr.user.clone().unwrap().login);
            pr
        })
        .filter(|pr| pr.user.clone().unwrap().login == LOGIN)
        .filter(|pr| pr.created_at.is_some())
        .filter(|pr| pr.created_at.unwrap() > quarter);
    for pr in filtered_on_user { 
        if let Some(text) = pr.title {
            println!("{text} #{}", pr.number);
        }
        if let Some(text) = pr.body_text {
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
        if let Some(mergable) = pr.mergeable_state {
            println!("Merge status {mergable:?}");
        }
    }
}
