use std::time::Instant;
use reqwest::Client;
use anyhow::anyhow;

use crate::config::Config;
use crate::error::Result;
use crate::model::{CreateIssueBody, Issue, Project, ProjectMember, User, Label, IssueStatus, PaginationResult};

#[derive(Debug)]
pub struct ResponseData<T> {
    pub duration: u128,
    pub data: Option<T>,
}

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36";

pub async fn fetch_me(config: &Config) -> Result<User> {
    let url = format!("{}/user", config.base_url);
    let response = Client::new()
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
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

pub async fn fetch_projects(config: &Config, page: u32, per_page: u32) -> Result<PaginationResult<Project>> {
    let url = format!("{}/projects", config.base_url.as_str());
    let query_params = vec![
        ("status", "active".to_string()),
        ("page", page.to_string()),
        ("per_page", per_page.to_string()),
        ("sort", "-lastActivityDate".to_string()),
        ("include", "meta,activeSprint,members,organisation".to_string()),
    ];

    let response = Client::new()
        .get(url)
        .query(&query_params)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .bearer_auth(config.token.as_str())
        .send()
        .await?;

    if response.status().is_success() {
        let result: PaginationResult<Project> = response.json().await?;
        Ok(result)
    } else {
        let message = format!("Unable to fetch project listing. Error: {}", response.status());
        eprintln!("{}", message);
        Err(anyhow!(message))
    }
}


pub async fn fetch_project(config: &Config, project_id: &str) -> Result<Project> {
    let url = format!("{}/projects/{}", config.base_url.as_str(), project_id);
    let query_params = vec![
        ("include", "organisation".to_string())
    ];
    let response = Client::new()
        .get(url)
        .query(&query_params)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
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

pub async fn fetch_labels(config: &Config, project_id: &str) -> Result<Vec<Label>> {
    let url = format!("{}/projects/{}/labels", config.base_url.as_str(), project_id);
    let response = Client::new()
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .bearer_auth(config.token.as_str())
        .send()
        .await?;

    if response.status().is_success() {
        let labels: Vec<Label> = response.json().await?;
        Ok(labels)
    } else {
        Err(anyhow!("Unable to fetch project labels {}. Error: {}", project_id, response.status()))
    }
}

pub async fn fetch_statuses(config: &Config, project_id: &str) -> Result<Vec<IssueStatus>> {
    let url = format!("{}/projects/{}/issueStatuses", config.base_url.as_str(), project_id);
    let response = Client::new()
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .bearer_auth(config.token.as_str())
        .send()
        .await?;

    if response.status().is_success() {
        let statuses: Vec<IssueStatus> = response.json().await?;
        Ok(statuses)
    } else {
        Err(anyhow!("Unable to fetch project issue statuses {}. Error: {}", project_id, response.status()))
    }
}

pub async fn fetch_epics(config: &Config, project_id: &str) -> Result<Vec<Issue>> {
    let url = format!("{}/projects/{}/issues", config.base_url.as_str(), project_id);
    let query_params = vec![
        ("type", "epic".to_string()),
        ("state", "active".to_string()),
        ("page", "1".to_string()),
        ("per_page", "50".to_string()),
        ("sort", "-createdAt".to_string()),
        ("include", "createdBy,assignee,developmentUpdates,isFollower,subtasksCount".to_string()),
    ];
    let response = Client::new()
        .get(url)
        .query(&query_params)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
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

pub async fn fetch_members(config: &Config, project_id: &str) -> Result<Vec<ProjectMember>> {
    let url = format!("{}/iam/projects/{}/members/?status=active", config.base_url.as_str(), project_id);
    let response = Client::new()
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
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

pub async fn create_issue(config: &Config, project_id: &str, payload: &CreateIssueBody) -> Result<ResponseData<Issue>> {
    let mut res: ResponseData<Issue> = ResponseData {
        duration: 0,
        data: None,
    };

    let d = Instant::now();
    let create_res = do_create_issue(config, project_id, payload).await;
    res.duration = d.elapsed().as_millis();
    if let Ok(issue_res) = create_res {
        println!("{}: {} --> {} ms", issue_res.key, issue_res.title, res.duration);
        res.data = Some(issue_res);
    }

    Ok(res)
}

async fn do_create_issue(config: &Config, project_id: &str, payload: &CreateIssueBody) -> Result<Issue> {
    let url = format!("{}/projects/{}/issues", config.base_url.as_str(), project_id);
    let post_body = serde_json::to_string(payload)?;

    let response = Client::new()
        .post(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
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

pub async fn fetch_issues(config: &Config, project_id: &str, page: u32, per_page: u32) -> Result<PaginationResult<Issue>> {
    let url = format!("{}/projects/{}/issues", config.base_url.as_str(), project_id);
    let query_params = vec![
        ("state", "active".to_string()),
        ("page", page.to_string()),
        ("per_page", per_page.to_string()),
        ("sort", "-createdAt".to_string()),
        ("include", "createdBy,assignee,developmentUpdates,isFollower,subtasksCount,meta".to_string()),
    ];

    let response = Client::new()
        .get(url)
        .query(&query_params)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .bearer_auth(config.token.as_str())
        .send()
        .await?;

    if response.status().is_success() {
        let result: PaginationResult<Issue> = response.json().await?;
        Ok(result)
    } else {
        let message = format!("Unable to fetch issue listing. Error: {}", response.status());
        eprintln!("{}", message);
        Err(anyhow!(message))
    }
}

pub async fn fetch_issue(config: &Config, project_id: &str, issue_id: &str) -> Result<ResponseData<Issue>> {
    let mut res: ResponseData<Issue> = ResponseData {
        duration: 0,
        data: None,
    };

    let d = Instant::now();
    let create_res = do_fetch_issue(config, project_id, issue_id).await;
    res.duration = d.elapsed().as_millis();
    if let Ok(issue_res) = create_res {
        println!("{}: {} --> {} ms", issue_res.key, issue_res.title, res.duration);
        res.data = Some(issue_res);
    }

    Ok(res)
}

async fn do_fetch_issue(config: &Config, project_id: &str, issue_id: &str) -> Result<Issue> {
    let url = format!("{}/projects/{}/issues/{}", config.base_url.as_str(), project_id, issue_id);
    let query_params = vec![
        ("include", "isCreator,isAssignee,isFollower,initiative,epic,parent,commitment,subtasksCount".to_string()),
    ];

    let response = Client::new()
        .get(url)
        .query(&query_params)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .bearer_auth(config.token.as_str())
        .send()
        .await?;

    if response.status().is_success() {
        let issue: Issue = response.json().await?;
        Ok(issue)
    } else {
        let message = format!("Unable to fetch issue. Error: {}", response.status());
        eprintln!("{}", message);
        Err(anyhow!(message))
    }
}
