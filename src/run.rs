use fake::Fake;
use fake::faker::lorem::en::*;
use rand::Rng;
use reqwest::Client;

use crate::config::Config;
use crate::error::Result;
use crate::model::{CreateIssueBody, Issue, IssueStatus, Label, Project, ProjectMember, User};

pub async fn run(config: Config) -> Result<()> {
    let current_user = fetch_me(&config).await?;
    println!("Logged in as: {}", current_user.username);

    let project = fetch_project(&config).await?;
    println!("{}: {}", project.key, project.name);

    // Collect statuses and labels
    let mut labels: Vec<Label> = Vec::new();
    let mut statuses: Vec<IssueStatus> = Vec::new();

    if let Some(preferences) = project.preferences {
        labels = preferences.labels;
        statuses = preferences.issue_statuses;
    }

    let epics = fetch_epics(&config).await?;
    let members = fetch_members(&config).await?;

    let mut handles = vec![];

    for _ in 0..=config.issue_count {
        let epic = get_random_item(&epics, true);
        let member = get_random_item(&members, true);
        let status = get_random_item(&statuses, false);
        let label = get_random_item(&labels, true);

        let default_labels: Vec<String> = vec![];

        let title: Vec<String> = Words(3..5).fake();

        let mut payload = CreateIssueBody {
            r#type: String::from("user_story"),
            epic_id: None,
            parent_id: None,
            assignee_id: None,
            title: title.join(" "),
            description: Some(String::from("desc")),
            estimate_type: Some(String::from("hours")),
            estimate: Some(10),
            status: Some(String::from("status")),
            labels: default_labels,
        };

        if let Some(epic_value) = epic {
            payload.epic_id = Some(String::from(epic_value.id.as_str()));
        }
        if let Some(member_value) = member {
            if let Some(user_value) = &member_value.user {
                payload.assignee_id = Some(String::from(user_value.id.as_str()));
            }
        }
        if let Some(status_value) = status {
            payload.status = Some(String::from(status_value.id.as_str()));
        }
        if let Some(label_value) = label {
            payload.labels = vec![String::from(label_value.id.as_str())];
        }

        let config_copy = config.clone();
        let handle = tokio::spawn(async move {
            create_issue(config_copy, payload).await.unwrap();
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}

async fn fetch_me(config: &Config) -> Result<User> {
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
        return Err(Box::from("Unable to fetch current user."));
    }
}

async fn fetch_project(config: &Config) -> Result<Project> {
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
        return Err(Box::from(format!("Unable to fetch project {}", config.project_id.as_str())));
    }
}

fn get_random_item<T>(items: &Vec<T>, allow_none: bool) -> Option<&T> {
    match items.len() {
        0 => None,
        length => {
            // Allow returning None from time to time
            let max_length = match allow_none {
                true => length,
                false => length - 1,
            };
            let key = rand::thread_rng().gen_range(0..=max_length);
            items.get(key)
        }
    }
}

async fn fetch_epics(config: &Config) -> Result<Vec<Issue>> {
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
        return Err(Box::from(format!("Unable to fetch epics.")));
    }
}

async fn fetch_members(config: &Config) -> Result<Vec<ProjectMember>> {
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
        return Err(Box::from(format!("Unable to fetch project members.")));
    }
}

async fn create_issue(config: Config, payload: CreateIssueBody) -> Result<Issue> {
    let url = format!("{}/projects/{}/issues", config.base_url.as_str(), config.project_id.as_str());
    let post_body = serde_json::to_string(&payload)?;

    let response = Client::new()
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(post_body)
        .bearer_auth(config.token.as_str())
        .send()
        .await?;

    if response.status().is_success() {
        let issue: Issue = response.json().await?;
        println!("{}: {}", issue.key, issue.title);
        Ok(issue)
    } else {
        println!("{:?}", response.text().await?);
        return Err(Box::from(format!("Unable to create new issue.")));
    }
}