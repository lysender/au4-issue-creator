use std::time::Instant;

use fake::Fake;
use fake::faker::company::en::CatchPhase;
use rand::Rng;
use reqwest::Client;

use crate::config::Config;
use crate::error::Result;
use crate::model::{CreateIssueBody, Issue, IssueStatus, Label, Project, ProjectMember, User};

pub async fn run(config: Config) -> Result<()> {
    let timer = Instant::now();
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

    for _ in 0..config.issue_count {
        let epic = get_random_item(&epics, 20);
        let member = get_random_item(&members, 30);
        let status = get_random_item(&statuses, 100);
        let label = get_random_item(&labels, 30);

        let default_labels: Vec<String> = vec![];

        let title: String = CatchPhase().fake();
        let description = format!("{}, {}, {}, {}", CatchPhase().fake::<String>(), CatchPhase().fake::<String>(), CatchPhase().fake::<String>(), CatchPhase().fake::<String>());

        let mut payload = CreateIssueBody {
            r#type: String::from("user_story"),
            epic_id: None,
            parent_id: None,
            assignee_id: None,
            title,
            description: Some(description),
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
            create_issue(config_copy, payload).await.unwrap()
        });

        handles.push(handle);
    }

    // Gather stats
    let total_reqs: u128 = handles.len().try_into().unwrap();
    let mut min_duration: u128 = 0;
    let mut max_duration: u128 = 0;
    let avg_duration: u128;
    let mut sum: u128 = 0;

    for handle in handles {
        let res = handle.await.unwrap();
        if let Some(issue) = res.data {
            println!("{}: {} --> {} ms", issue.key, issue.title, res.duration);
        }

        sum += res.duration;

        if min_duration == 0 {
            min_duration = res.duration;
        } else if res.duration < min_duration {
            min_duration = res.duration;
        }

        if res.duration > max_duration {
            max_duration = res.duration;
        }
    }

    avg_duration = sum / total_reqs;

    let rps: u128 = (sum / 1000) / total_reqs;
    let total_time = timer.elapsed().as_millis();

    // Print stats
    println!("");
    println!("Total requests: {}", total_reqs);
    println!("Min: {} ms", min_duration);
    println!("Avg: {} ms", avg_duration);
    println!("Max: {} ms", max_duration);
    println!("Request per seconds: {}", rps);
    println!("Total duration: {} ms", total_time);

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
        Err(Box::from("Unable to fetch current user."))
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
        Err(Box::from(format!("Unable to fetch project {}", config.project_id.as_str())))
    }
}

fn get_item_chance(chance: u32) -> bool {
    if chance > 100 {
        panic!("Chance must be between 0 to 100")
    }

    let value = rand::thread_rng().gen_range(0..=100);
    value <= chance
}

fn get_random_item<T>(items: &Vec<T>, chance: u32) -> Option<&T> {
    let length = items.len();
    let return_item = get_item_chance(chance);

    if length > 0 && return_item {
        let max_length = length - 1;
        let key = rand::thread_rng().gen_range(0..=max_length);
        return items.get(key);
    }
    None
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
        Err(Box::from(format!("Unable to fetch epics.")))
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
        Err(Box::from(format!("Unable to fetch project members.")))
    }
}

#[derive(Debug)]
struct ResponseData<T> {
    duration: u128,
    data: Option<T>,
}

async fn create_issue(config: Config, payload: CreateIssueBody) -> Result<ResponseData<Issue>> {
    let mut res: ResponseData<Issue> = ResponseData {
        duration: 0,
        data: None,
    };

    let d = Instant::now();
    let create_res = do_create_issue(config, payload).await;
    if let Ok(issue_res) = create_res {
        res.data = Some(issue_res);
    }

    res.duration = d.elapsed().as_millis();
    Ok(res)
}

async fn do_create_issue(config: Config, payload: CreateIssueBody) -> Result<Issue> {
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
        Ok(issue)
    } else {
        eprintln!("{:?}", response.text().await?);
        Err(Box::from(format!("Unable to create new issue.")))
    }
}
