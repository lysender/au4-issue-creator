use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub key: String,
    pub name: String,
    pub preferences: Option<ProjectPreferences>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPreferences {
    pub issue_statuses: Vec<IssueStatus>,
    pub labels: Vec<Label>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Label {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssueStatus {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: String,
    pub key: String,
    pub project_id: String,
    pub epic_id: Option<String>,
    pub parent_id: Option<String>,
    pub r#type: String,
    pub title: String,
    pub description: Option<String>,
    pub estimate: Option<u32>,
    pub estimate_type: Option<String>,
    pub labels: Option<Vec<String>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectMember {
    pub id: String,
    pub user: Option<User>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueBody {
    pub r#type: String,
    pub epic_id: Option<String>,
    pub parent_id: Option<String>,
    pub assignee_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub estimate_type: Option<String>,
    pub estimate: Option<u32>,
    pub status: Option<String>,
    pub labels: Vec<String>,
}
