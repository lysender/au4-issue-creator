use std::time::Instant;
use bigdecimal::BigDecimal;

use fake::Fake;
use fake::faker::company::en::CatchPhase;
use rand::Rng;

use crate::config::Config;
use crate::error::Result;
use crate::model::{CreateIssueBody, IssueStatus, Label};
use crate::crawler::{fetch_me, fetch_project, fetch_epics, fetch_members, create_issue};

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

    // Remove last status, should not create issues as done
    if statuses.len() > 0 {
        statuses.pop();
    }

    let epics = fetch_epics(&config).await?;
    let members = fetch_members(&config).await?;

    let create_timer = Instant::now();

    let mut handles = vec![];

    for _ in 0..config.issue_count {
        let epic = get_random_item(&epics, 20);
        let member = get_random_item(&members, 30);
        let status = get_random_item(&statuses, 100);
        let label = get_random_item(&labels, 30);

        let default_labels: Vec<String> = vec![];

        let title: String = CatchPhase().fake();
        let description = format!(
            "{}, {}, {}, {}",
            CatchPhase().fake::<String>(),
            CatchPhase().fake::<String>(),
            CatchPhase().fake::<String>(),
            CatchPhase().fake::<String>()
        );

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
            create_issue(&config_copy, &payload).await.unwrap()
        });

        handles.push(handle);
    }

    // Gather stats
    let total_reqs: u32 = handles.len().try_into().unwrap();
    let mut failed: u32 = 0;
    let mut min_duration: u128 = 0;
    let mut max_duration: u128 = 0;
    let mut sum: u128 = 0;

    for handle in handles {
        let res = handle.await.unwrap();
        if let None = res.data {
            failed += 1;
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

    let succeed = total_reqs - failed;
    let success_ratio: f64 = (f64::from(succeed) / f64::from(total_reqs)) * 100.0;
    let success_ration_rounded = success_ratio.round();
    let big_sum = BigDecimal::from(sum);
    let big_total_reqs = BigDecimal::from(total_reqs);
    let big_avg = big_sum/ big_total_reqs.clone();

    let total_time = timer.elapsed().as_millis();
    let total_create_time = create_timer.elapsed().as_millis();
    let big_create_total_time = BigDecimal::from(total_create_time);
    let big_rps: BigDecimal = big_total_reqs / (big_create_total_time / 1000.0);
    let rps = big_rps.round(2);

    // Print stats
    println!("");
    println!("Total requests: {}", total_reqs);
    println!("Succeed: {}", succeed);
    println!("Failed: {}", failed);
    println!("Success rate: {}%", success_ration_rounded);
    println!("Min: {} ms", min_duration);
    println!("Avg: {} ms", big_avg);
    println!("Max: {} ms", max_duration);
    println!("Requests per second: {}", rps);
    println!("Run duration: {} ms", total_time);

    Ok(())
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

