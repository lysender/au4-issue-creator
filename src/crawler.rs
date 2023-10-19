use std::time::Instant;
use reqwest::Client;
use anyhow::anyhow;

use crate::config::Config;
use crate::error::Result;
use crate::model::{CreateIssueBody, Issue, Project, ProjectMember, User};

#[derive(Debug)]
pub struct ResponseData<T> {
    pub duration: u128,
    pub data: Option<T>,
}

pub async fn fetch_me(config: &Config) -> Result<User> {
    let url = format!("{}/user", config.base_url);
    let response = Client::new()
        .get(url)
        .bearer_auth(config.token.as_str())
        .send()
        .await?;

    if response.status().is_success() {
        let user: User = response.json().await?;
        Ok(user)
    } else {
        Err(anyhow!("Unable to fetch current user. Error: {}", response.status()))
    }
}

pub async fn fetch_project(config: &Config) -> Result<Project> {
    let url = format!("{}/projects/{}", config.base_url.as_str(), config.project_id.as_str());
    let response = Client::new()
        .get(url)
        .bearer_auth(config.token.as_str())
        .send()
        .await?;

    if response.status().is_success() {
        let project: Project = response.json().await?;
        Ok(project)
    } else {
        Err(anyhow!("Unable to fetch project {}. Error: {}", config.project_id.as_str(), response.status()))
    }
}

pub async fn fetch_epics(config: &Config) -> Result<Vec<Issue>> {
    let url = format!("{}/projects/{}/issues/?type=epic&state=active", config.base_url.as_str(), config.project_id.as_str());
    let response = Client::new()
        .get(url)
        .bearer_auth(config.token.as_str())
        .send()
        .await?;

    if response.status().is_success() {
        let issues: Vec<Issue> = response.json().await?;
        Ok(issues)
    } else {
        Err(anyhow!("Unable to fetch epics. Error: {}", response.status()))
    }
}

pub async fn fetch_members(config: &Config) -> Result<Vec<ProjectMember>> {
    let url = format!("{}/iam/projects/{}/members/?status=active", config.base_url.as_str(), config.project_id.as_str());
    let response = Client::new()
        .get(url)
        .bearer_auth(config.token.as_str())
        .send()
        .await?;

    if response.status().is_success() {
        let members: Vec<ProjectMember> = response.json().await?;
        Ok(members)
    } else {
        Err(anyhow!("Unable to fetch project members. Error: {}", response.status()))
    }
}

pub async fn create_issue(config: &Config, payload: &CreateIssueBody) -> Result<ResponseData<Issue>> {
    let mut res: ResponseData<Issue> = ResponseData {
        duration: 0,
        data: None,
    };

    let d = Instant::now();
    let create_res = do_create_issue(config, payload).await;
    res.duration = d.elapsed().as_millis();
    if let Ok(issue_res) = create_res {
        let issue_copy = issue_res.clone();
        println!("{}: {} --> {} ms", issue_copy.key, issue_copy.title, res.duration);
        res.data = Some(issue_res);
    }

    Ok(res)
}

async fn do_create_issue(config: &Config, payload: &CreateIssueBody) -> Result<Issue> {
    let url = format!("{}/projects/{}/issues", config.base_url.as_str(), config.project_id.as_str());
    let post_body = serde_json::to_string(payload)?;

    let response = Client::new()
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(post_body)
        .bearer_auth(config.token.as_str())
        .send()
        .await?;

    if response.status().is_success() {
        let issue: Issue = response.json().await?;
        Ok(issue)
    } else {
        let message = format!("Unable to create issue. Error: {}", response.status());
        eprintln!("{}", message);
        Err(anyhow!(message))
    }
}

