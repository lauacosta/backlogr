use core::fmt;
use std::collections::HashMap;

use clap::ValueEnum;
use color_eyre::owo_colors::OwoColorize;
use eyre::Result;
use facet::Facet;

pub const TAIGA_API_URL: &str = "https://api.taiga.io/api/v1";

pub enum TaigaAPIError {
    Authentication(String),
    StoryNotFound(String),
    ProjectNotFound(String),
    ApiError(String),
    InternalError(minreq::Error),
    DeserializationError(String),
}

impl fmt::Display for TaigaAPIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaigaAPIError::Authentication(msg) => write!(f, "Authentication failed: {}", msg),
            TaigaAPIError::StoryNotFound(msg) => write!(f, "User story not found: {}", msg),
            TaigaAPIError::ProjectNotFound(msg) => write!(f, "Project not found: {}", msg),
            TaigaAPIError::ApiError(msg) => write!(f, "API error: {}", msg),
            TaigaAPIError::InternalError(err) => write!(f, "Network error: {}", err),
            TaigaAPIError::DeserializationError(err) => {
                write!(f, "Failed to parse response: {}", err)
            }
        }
    }
}

impl From<facet_json::DeserErrorKind<'_>> for TaigaAPIError {
    fn from(err: facet_json::DeserErrorKind) -> Self {
        TaigaAPIError::DeserializationError(format!("{:?}", err))
    }
}

impl From<minreq::Error> for TaigaAPIError {
    fn from(err: minreq::Error) -> Self {
        TaigaAPIError::InternalError(err)
    }
}

impl TaigaAPIError {
    pub fn print_tip(&self) {
        match self {
            TaigaAPIError::Authentication(_) => {
                eprintln!("💡 Troubleshooting authentication:");
                eprintln!("   • Set environment variables:");
                eprintln!("     export USERNAME=your_taiga_username");
                eprintln!("     export PASSWORD=your_taiga_password");
                eprintln!("   • Verify credentials by logging into Taiga web interface");
                eprintln!("   • Check if your account is active and not locked");
            }
            TaigaAPIError::StoryNotFound(story) => {
                eprintln!("💡 Story '{}' not found. Try:", story);
                eprintln!("   • backlogr list           # See all available stories");
                eprintln!("   • backlogr create         # Create a new story");
                eprintln!("   • Check for typos in the story title");
                eprintln!("   • Ensure you're in the correct project");
            }
            TaigaAPIError::ProjectNotFound(project) => {
                eprintln!("💡 Project '{}' not found. Check:", project);
                eprintln!("   • Project name spelling (case-sensitive)");
                eprintln!("   • Your permissions to access this project");
                eprintln!("   • If the project exists in your Taiga instance");
                eprintln!("   • Set correct PROJECT_NAME environment variable");
            }
            TaigaAPIError::ApiError(msg) => {
                eprintln!("💡 API error occurred:");
                if msg.contains("500") || msg.contains("502") || msg.contains("503") {
                    eprintln!("   • Taiga server appears to be experiencing issues");
                    eprintln!("   • Try again in a few minutes");
                    eprintln!("   • Contact your Taiga administrator if this persists");
                } else if msg.contains("401") {
                    eprintln!("   • This looks like an authentication issue");
                    eprintln!("   • Check your username and password");
                } else if msg.contains("403") {
                    eprintln!("   • Permission denied - you may not have access to this resource");
                    eprintln!("   • Contact your project administrator");
                } else if msg.contains("404") {
                    eprintln!("   • Resource not found - check project/story names");
                } else {
                    eprintln!("   • Check your network connection");
                    eprintln!("   • Verify your Taiga instance URL is correct");
                    eprintln!("   • Try the operation again");
                }
            }
            TaigaAPIError::InternalError(error) => {
                eprintln!("💡 Network/connection error:");
                let error_msg = error.to_string().to_lowercase();
                if error_msg.contains("connection") || error_msg.contains("timeout") {
                    eprintln!("   • Check your internet connection");
                    eprintln!("   • Verify Taiga instance URL is accessible");
                    eprintln!("   • Try again - this might be a temporary issue");
                } else if error_msg.contains("dns") || error_msg.contains("resolve") {
                    eprintln!("   • DNS resolution failed");
                    eprintln!("   • Check if the Taiga hostname is correct");
                    eprintln!("   • Try using an IP address instead of hostname");
                } else if error_msg.contains("ssl") || error_msg.contains("tls") {
                    eprintln!("   • SSL/TLS certificate issue");
                    eprintln!("   • Check if your Taiga instance uses valid certificates");
                } else {
                    eprintln!("   • Network error: {}", error);
                    eprintln!("   • Check your connection and try again");
                }
            }
            TaigaAPIError::DeserializationError(error) => {
                eprintln!("💡 Data parsing error:");
                eprintln!("   • Taiga API response format may have changed");
                eprintln!("   • This might indicate a version compatibility issue");
                eprintln!("   • Error details: {}", error);
                eprintln!("   • Try updating backlogr to the latest version");
                eprintln!("   • Report this issue if it persists");
            }
        }
    }
    pub fn exit_code(&self) -> i32 {
        match self {
            TaigaAPIError::Authentication(_) => 1,
            TaigaAPIError::InternalError(_) => 1,
            TaigaAPIError::StoryNotFound(_) => 2,
            TaigaAPIError::ProjectNotFound(_) => 3,
            TaigaAPIError::ApiError(_) => 4,
            TaigaAPIError::DeserializationError(_) => 5,
        }
    }

    pub fn exit_with_tips(self) -> ! {
        eprintln!("❌ {}", self);
        self.print_tip();
        std::process::exit(self.exit_code());
    }
}

pub struct TaigaAPI {
    pub auth_token: String,
    pub api_url: String,
}

impl TaigaAPI {
    /// Authenticates a user with the Taiga API using a username and password.
    ///
    /// On success, returns a new instance of the API client with a valid auth token.
    ///
    /// # Errors
    /// Returns `TaigaAPIError::ApiError` if the credentials are invalid
    /// or if there is a problem communicating with the API.
    pub fn authenticate(username: &str, password: &str) -> Result<Self, TaigaAPIError> {
        eprintln!("🔐 Authenticating with Taiga API...");
        #[derive(Facet)]
        struct AuthPayload<'a> {
            #[facet(rename = "type")]
            user_type: &'a str,
            username: &'a str,
            password: &'a str,
        }

        let payload = AuthPayload {
            user_type: "Normal",
            username,
            password,
        };

        let payload_str = facet_json::to_string(&payload);

        let response = minreq::post(format!("{TAIGA_API_URL}/auth"))
            .with_header("Content-Type", "application/json")
            .with_body(payload_str)
            .send()?;

        if response.status_code != 200 {
            let body = response.as_str()?;
            return Err(TaigaAPIError::Authentication(format!(
                "HTTP {}: {}",
                response.status_code, body
            )));
        }

        let response_txt = response.as_str()?;
        dbg!("{:?}", response_txt);
        let user_auth_detail: UserAuthenticationDetail =
            facet_json::from_str(response_txt).unwrap();
        let auth_token = user_auth_detail.auth_token;

        Ok(Self {
            auth_token,
            api_url: TAIGA_API_URL.to_owned(),
        })
    }

    /// Lists all user stories for the given project ID.
    ///
    /// This fetches user stories the authenticated user has access to in the specified project.
    ///
    /// # Errors
    /// Returns `TaigaAPIError::ApiError` if the request fails or the API response is invalid.
    pub fn list_all_stories(&self, project_id: usize) -> Result<Vec<UserStory>, TaigaAPIError> {
        let mut all_stories = Vec::new();
        let mut page = 1;
        let page_size = 100;

        loop {
            let (stories, has_more) = self.list_stories_page(project_id, page, page_size)?;
            all_stories.extend(stories);

            if !has_more {
                break;
            }

            page += 1;
        }

        Ok(all_stories)
    }

    fn list_stories_page(
        &self,
        project_id: usize,
        page: usize,
        page_size: usize,
    ) -> Result<(Vec<UserStory>, bool), TaigaAPIError> {
        let auth_token = self.auth_token.clone();
        let api_url = self.api_url.clone();

        let response = minreq::get(format!(
            "{api_url}/userstories?project={project_id}&page={page}&page_size={page_size}"
        ))
        .with_header("Authorization", format!("Bearer {auth_token}"))
        .send()?;

        if response.status_code != 200 {
            let body = response.as_str()?;
            return Err(TaigaAPIError::ApiError(format!(
                "Fetching the list of stories failed. HTTP {}: {}",
                response.status_code, body
            )));
        }

        let response_str = response.as_str()?;
        let stories: Vec<UserStory> = facet_json::from_str(response_str).unwrap();
        let current_page_count = response
            .headers
            .get("x-pagination-count")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        let page_size_header = response
            .headers
            .get("x-paginated-by")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(page_size);

        let is_paginated = response
            .headers
            .get("x-paginated")
            .map(|s| s == "true")
            .unwrap_or(false);

        let has_more = is_paginated && current_page_count == page_size_header;

        Ok((stories, has_more))
    }

    /// Retrieves the project ID for a given project name where the current user is a member.
    ///
    /// # Errors
    /// Returns `TaigaAPIError::ApiError` if the user or project list cannot be fetched,
    /// or if the project name is not found among the user’s projects.
    pub fn get_project_id(&self, project_name: &str) -> Result<usize, TaigaAPIError> {
        let user_id = {
            let response = minreq::get(format!("{TAIGA_API_URL}/users/me"))
                .with_header("Authorization", format!("Bearer {}", self.auth_token))
                .send()?;

            if response.status_code != 200 {
                let body = response.as_str()?;
                return Err(TaigaAPIError::ApiError(format!(
                    "HTTP {}: {}",
                    response.status_code, body
                )));
            }

            let response_str = response.as_str()?;
            let user_detail: UserDetail = facet_json::from_str(response_str).unwrap();

            user_detail.id
        };

        eprintln!("🔗 Connected to Taiga (User ID: {})", user_id.bold().cyan());

        let Some(project_id) = ({
            let response = minreq::get(format!("{TAIGA_API_URL}/projects?member={user_id}"))
                .with_header("Authorization", format!("Bearer {}", self.auth_token))
                .send()?;

            if response.status_code != 200 {
                let body = response.as_str()?;
                return Err(TaigaAPIError::ApiError(format!(
                    "HTTP {}: {}",
                    response.status_code, body
                )));
            }

            let response_str = response.as_str()?;
            let projects_entry: Vec<ProjectListEntry> = facet_json::from_str(response_str).unwrap();

            projects_entry
                .iter()
                .find(|v| v.name == project_name)
                .map(|v| v.id)
        }) else {
            return Err(TaigaAPIError::ProjectNotFound(format!(
                "Could not find a project named {project_name}. Please check the project name."
            )));
        };

        println!(
            "📂 Project: {} (ID: {})",
            project_name.bright_green().bold(),
            project_id.bright_green().bold()
        );

        Ok(project_id)
    }

    /// Creates a new user story in the specified project with the given subject and status.
    ///
    /// # Errors
    /// Returns `TaigaAPIError::ApiError` if the request fails, status cannot be found,
    /// or the API response is invalid.
    pub fn create_story(
        &self,
        project_id: usize,
        subject: &str,
        description: &str,
        status: &Status,
    ) -> Result<usize, TaigaAPIError> {
        let auth_token = self.auth_token.clone();

        let status_id = self.get_status_id(project_id, status)?;

        #[derive(Facet)]
        struct StoryPayload<'a> {
            project_id: usize,
            subject: &'a str,
            description: &'a str,
            #[facet(rename = "status")]
            status_id: usize,
        }

        let payload = StoryPayload {
            project_id,
            subject,
            description,
            status_id,
        };

        let payload_str = facet_json::to_string(&payload);

        let response = minreq::post(format!("{TAIGA_API_URL}/userstories"))
            .with_headers([
                ("Authorization", format!("Bearer {auth_token}")),
                ("Content-Type", "application/json".to_owned()),
            ])
            .with_body(payload_str)
            .send()?;

        if response.status_code != 201 {
            let body = response.as_str()?;
            return Err(TaigaAPIError::ApiError(format!(
                "Creating new story failed. HTTP {}: {}",
                response.status_code, body
            )));
        }

        let response_txt = response.as_str()?;
        let story_detail: UserStoryDetail = facet_json::from_str(response_txt).unwrap();

        Ok(story_detail.reference)
    }

    /// Finds the internal user story ID from a reference number within a given project.
    ///
    /// # Errors
    /// Returns `TaigaAPIError::ApiError` if the story reference is not found.
    pub fn get_story_id(&self, project_id: usize, story_id: usize) -> Result<usize, TaigaAPIError> {
        eprintln!("🔍 Looking up user story with ref #{story_id} in project...");

        let user_story_list = self.list_all_stories(project_id)?;

        user_story_list
            .iter()
            .find(|v| v.reference == story_id)
            .map(|v| v.id)
            .ok_or(TaigaAPIError::StoryNotFound(format!(
                "User story with ref #{story_id} not found."
            )))
    }

    /// Updates the status of an existing user story in the specified project.
    ///
    /// This function fetches the current version of the story and updates its status
    /// to the one provided.
    ///
    /// # Errors
    /// Returns `TaigaAPIError::ApiError` if the status or story cannot be retrieved or updated.
    pub fn update_story_status(
        &self,
        project_id: usize,
        story_id: usize,
        user_story_id: usize,
        status: &Status,
    ) -> Result<(), TaigaAPIError> {
        let auth_token = self.auth_token.clone();
        let api_url = self.api_url.clone();

        eprintln!("✅ Found user story ID: {}", user_story_id.bold().cyan());
        eprintln!("🔍 Fetching '{status}' status ID for the project...");

        let status_id = self.get_status_id(project_id, status)?;

        eprintln!("✅ '{status}' status ID is: {}", status_id.bold().green());

        eprintln!("🔍 Retrieving current version of user story #{story_id}...");

        let user_story_current_version = self.retrieve_current_version(user_story_id)?;

        eprintln!("✅ Current version of user story #{story_id} is {user_story_current_version}");

        eprintln!("🔄 Updating user story status to '{status}'...");

        #[derive(Facet)]
        struct StatusPayload {
            status: usize,
            version: usize,
        }

        let payload = StatusPayload {
            status: status_id,
            version: user_story_current_version,
        };

        let payload_str = facet_json::to_string(&payload);

        let response = minreq::patch(format!("{api_url}/userstories/{user_story_id}"))
            .with_headers([
                ("Authorization", format!("Bearer {auth_token}")),
                ("Content-Type", "application/json".to_owned()),
            ])
            .with_body(payload_str)
            .send()?;

        if response.status_code != 200 {
            let body = response.as_str()?;
            return Err(TaigaAPIError::ApiError(format!(
                "Failed to update {story_id} to '{status}'. HTTP {}: {}",
                response.status_code, body
            )));
        }

        eprintln!(
            "✅ Successfully updated user story  {story_id} to '{status}' (version {user_story_current_version})"
        );

        Ok(())
    }

    /// Deletes a user story with the given internal ID from the Taiga project.
    ///
    /// # Errors
    /// Returns `TaigaAPIError::ApiError` if the deletion fails.
    pub fn delete_story(&self, story_id: usize) -> Result<(), TaigaAPIError> {
        let auth_token = self.auth_token.clone();

        let response = minreq::delete(format!("{TAIGA_API_URL}/userstories/{story_id}"))
            .with_header("Authorization", format!("Bearer {auth_token}"))
            .send()?;

        if response.status_code != 204 {
            let body = response.as_str()?;
            return Err(TaigaAPIError::ApiError(format!(
                "Failed to delete the story failed. HTTP {}: {}",
                response.status_code, body
            )));
        }

        Ok(())
    }

    /// Retrieves the current version number of the specified user story.
    ///
    /// This is required when updating a story to avoid version conflicts.
    ///
    /// # Errors
    /// Returns `TaigaAPIError::ApiError` if the story details cannot be fetched.
    fn retrieve_current_version(&self, user_story_id: usize) -> Result<usize, TaigaAPIError> {
        let api_url = self.api_url.clone();
        let auth_token = self.auth_token.clone();

        let response = minreq::get(format!("{api_url}/userstories/{user_story_id}"))
            .with_header("Authorization", format!("Bearer {auth_token}"))
            .send()?;

        let response_str = response.as_str()?;
        let user_story_detail: UserStoryDetail = facet_json::from_str(response_str).unwrap();

        Ok(user_story_detail.version)
    }

    /// Fetches the status ID corresponding to a `Status` enum variant for a given project.
    ///
    /// # Errors
    /// Returns `TaigaAPIError::ApiError` if the status cannot be found or the request fails.
    fn get_status_id(&self, project_id: usize, status: &Status) -> Result<usize, TaigaAPIError> {
        let auth_token = self.auth_token.clone();
        let api_url = self.api_url.clone();

        let status = match status {
            Status::Done => "Done",
            Status::Wip => "In progress",
            Status::New => "New",
        };

        let response = minreq::get(format!("{api_url}/userstory-statuses?project={project_id}"))
            .with_header("Authorization", format!("Bearer {auth_token}"))
            .send()?;

        if response.status_code != 200 {
            let body = response.as_str()?;
            return Err(TaigaAPIError::ApiError(format!(
                "Unable to retrieve data for {project_id}. HTTP {}: {}",
                response.status_code, body
            )));
        }

        let response_str = response.as_str()?;

        let statuses_list: Vec<UserStoryStatusDetail> = facet_json::from_str(response_str).unwrap();
        statuses_list
            .iter()
            .find(|v| v.name == status)
            .map(|v| v.id)
            .ok_or(TaigaAPIError::ApiError(format!(
                "Could not find '{status}' status for project"
            )))
    }
}

#[repr(u8)]
#[derive(ValueEnum, Debug, Clone, Facet)]
pub enum Status {
    Done,
    Wip,
    New,
}

macro_rules! enum_all {
    ($enum_name:ident { $($variant:ident),* $(,)? }) => {
        impl $enum_name {
            pub fn all() -> Vec<$enum_name> {
                vec![$($enum_name::$variant),*]
            }
        }
    };
}

enum_all!(Status { New, Wip, Done });

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::New => write!(f, "New"),
            Status::Wip => write!(f, "In Progress"),
            Status::Done => write!(f, "Done"),
        }
    }
}

#[derive(Facet)]
/// <https://docs.taiga.io/api.html#object-userstory-status-detail>
struct UserStoryStatusDetail {
    id: usize,
    name: String,
}

#[derive(Facet)]
/// <https://docs.taiga.io/api.html#object-userstory-detail-list>
pub struct UserStory {
    id: usize,
    #[facet(rename = "ref")]
    reference: usize,
    subject: String,
    status: usize,
    created_date: String,
    status_extra_info: StatusInfo,
}

#[derive(Facet)]
struct StatusInfo {
    color: String,
    is_closed: bool,
    name: String,
}

impl fmt::Display for UserStory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = match self.status_extra_info.name.as_str() {
            "Done" => self.reference.bright_green().bold().to_string(),
            "In progress" => self.reference.bright_yellow().bold().to_string(),
            "New" => self.reference.bright_blue().bold().to_string(),
            _ => self.reference.bright_white().bold().to_string(),
        };

        write!(f, "#{:>2} {:<40}", id, self.subject)
    }
}

pub struct UserStories {
    pub new: Vec<UserStory>,
    pub wip: Vec<UserStory>,
    pub done: Vec<UserStory>,
    pub other: HashMap<String, Vec<UserStory>>,
}

impl UserStories {
    pub fn new(stories: Vec<UserStory>) -> Self {
        let mut new = Vec::new();
        let mut wip = Vec::new();
        let mut done = Vec::new();
        let mut other: HashMap<String, Vec<UserStory>> = HashMap::new();

        for story in stories {
            match story.status_extra_info.name.as_str() {
                "New" => new.push(story),
                "In progress" | "WIP" => wip.push(story),
                "Done" | "Ready" => done.push(story),
                status => other.entry(status.to_string()).or_default().push(story),
            }
        }

        Self {
            new,
            wip,
            done,
            other,
        }
    }

    pub fn total_count(&self) -> usize {
        self.new.len()
            + self.wip.len()
            + self.done.len()
            + self.other.values().map(|v| v.len()).sum::<usize>()
    }
}

impl fmt::Display for UserStories {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "📋 Total user stories: ({})\n", self.total_count())?;

        if !self.new.is_empty() {
            writeln!(f, "🆕 New ({})", self.new.len())?;
            for story in &self.new {
                writeln!(f, "  {story}")?;
            }
            writeln!(f)?;
        }

        if !self.wip.is_empty() {
            writeln!(f, "🔄 Work in Progress ({})", self.wip.len())?;
            for story in &self.wip {
                writeln!(f, "  {story}")?;
            }
            writeln!(f)?;
        }

        if !self.done.is_empty() {
            writeln!(f, "✅ Done ({})", self.done.len())?;
            for story in &self.done {
                writeln!(f, "  {story}")?;
            }
            writeln!(f)?;
        }

        for (status, stories) in &self.other {
            writeln!(f, "📌 {} ({})", status, stories.len())?;
            for story in stories {
                writeln!(f, "  {story}")?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Facet)]
/// <https://docs.taiga.io/api.html#object-userstory-detail-get>
struct UserStoryDetail {
    id: usize,
    #[facet(rename = "ref")]
    reference: usize,
    version: usize,
}

#[derive(Facet)]
/// <https://docs.taiga.io/api.html#object-userstory-status-detail>
struct UserAuthenticationDetail {
    auth_token: String,
    email: String,
    id: usize,
    refresh: String,
    username: String,
}

#[repr(u8)]
#[derive(Facet)]
enum Roles {
    Front,
    UX,
    Back,
    Design,
    #[facet(rename = "Product Owner")]
    ProductOwner,
}

#[derive(Facet)]
struct UserDetail {
    id: usize,
    username: String,
}

#[derive(Facet)]
/// <https://docs.taiga.io/api.html#object-project-list-entry>
struct ProjectListEntry {
    id: usize,
    name: String,
}

// TODO: Move this to a separate crate
pub trait ExitOnError<T> {
    fn or_exit(self) -> T;
}

impl<T> ExitOnError<T> for Result<T, TaigaAPIError> {
    fn or_exit(self) -> T {
        self.unwrap_or_else(|err| err.exit_with_tips())
    }
}
