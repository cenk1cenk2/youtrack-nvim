use std::fmt::Debug;

use chrono::prelude::*;
use mlua::AppDataRef;
use mlua::Lua;
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Module;
use crate::error::Error;
use crate::lua::NoData;
use crate::macros::{from_lua, into_lua};
use serde_json::{Value as JsonValue, json};

static SAVED_QUERY_FIELDS: &str = "id,name,query";
static ISSUES_FIELDS: &str = "id,idReadable,summary,description,project(id,name,shortName),customFields(id,name,presentation,value(id,name,presentation,color(background,foreground))),tags(id,color(background,foreground),name)";
static ISSUE_FIELDS: &str = "id,idReadable,summary,description,project(id,name,shortName),customFields(id,name,presentation,value(id,name,presentation,color(background,foreground))),tags(id,color(background,foreground),name),comments(author(fullName),text,created)";
static PROJECT_FIELDS: &str = "id,name,shortName";
static AGILES_FIELDS: &str = "id,name,projects(id,name,shortName),sprints(id,name,isDefault)";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pagination {
    pub skip: Option<i32>,
    pub take: Option<i32>,
}

impl Default for Pagination {
    fn default() -> Self {
        Pagination {
            skip: Some(0),
            take: Some(-1),
        }
    }
}

from_lua!(Pagination);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SavedQuery {
    pub id: String,

    pub name: String,

    pub query: String,
}

into_lua!(SavedQuery);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Issue {
    pub id: String,

    pub text: String,

    pub summary: String,

    pub description: Option<String>,

    pub project: Project,

    pub fields: Vec<Field>,

    pub tags: Vec<Tag>,

    pub comments: Option<Vec<Comment>>,
}

from_lua!(Issue);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,

    pub name: String,

    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agile {
    pub id: String,

    pub name: String,

    pub projects: Vec<Project>,

    pub sprints: Vec<Sprint>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sprint {
    pub id: String,

    pub name: String,

    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: String,

    pub name: String,

    pub color: JsonValue,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    pub author: String,

    pub text: String,

    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Field {
    pub id: String,

    pub name: String,

    pub text: String,

    pub value: Option<JsonValue>,

    pub values: Option<Vec<JsonValue>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSavedQueries {
    pub page: Option<Pagination>,
}

impl Default for GetSavedQueries {
    fn default() -> Self {
        GetSavedQueries {
            page: Some(Pagination::default()),
        }
    }
}

into_lua!(GetSavedQueries);
from_lua!(GetSavedQueries);

pub type GetSavedQueriesArgs = (Option<GetSavedQueries>, LuaFunction);

#[allow(unused_variables)]
pub async fn get_saved_queries(
    lua: &Lua,
    m: AppDataRef<'_, Module>,
    (options, callback): GetSavedQueriesArgs,
) -> Result<NoData, Error> {
    let mut url = m.api_url.clone();

    url.path_segments_mut().unwrap().push("savedQueries");

    let query: Vec<(&str, JsonValue)> = vec![
        ("fields", JsonValue::String(SAVED_QUERY_FIELDS.into())),
        (
            "$top",
            JsonValue::Number(
                options
                    .clone()
                    .unwrap_or_default()
                    .page
                    .unwrap_or_default()
                    .take
                    .unwrap_or_default()
                    .into(),
            ),
        ),
        (
            "$skip",
            JsonValue::Number(
                options
                    .clone()
                    .unwrap_or_default()
                    .page
                    .unwrap_or_default()
                    .skip
                    .unwrap_or_default()
                    .into(),
            ),
        ),
    ];

    let req = m.client.get(url).query(&query);

    log::debug!("Youtrack issue add comment request: {:?}", req);

    let res = req.send().await?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let json: JsonValue = res.json().await?;
            let result = json
                .as_array()
                .unwrap()
                .iter()
                .map(|query| process_saved_query(query.clone()))
                .collect::<Result<Vec<SavedQuery>, Error>>()?;

            log::debug!(
                "Youtrack saved queries fetched: {:?} -> {:#?}",
                options,
                result
            );
            callback.call::<()>((LuaNil, lua.to_value(&result)))?;
        }
        _ => {
            let text = res.text().await?;

            log::debug!(
                "Youtrack saved queries can not be fetched: {:?} -> {:#?}",
                options,
                text,
            );
            callback.call::<()>((
                format!("Youtrack saved queries can not be fetched: {:#?}", text),
                LuaNil,
            ))?;
        }
    }

    Ok(NoData)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetIssues {
    pub query: Option<String>,
    pub page: Option<Pagination>,
}

impl Default for GetIssues {
    fn default() -> Self {
        GetIssues {
            query: Some("for: me #Unresolved".to_string()),
            page: Some(Pagination::default()),
        }
    }
}

into_lua!(GetIssues);
from_lua!(GetIssues);

pub type GetIssuesArgs = (Option<GetIssues>, LuaFunction);

#[allow(unused_variables)]
pub async fn get_issues(
    lua: &Lua,
    m: AppDataRef<'_, Module>,
    (options, callback): GetIssuesArgs,
) -> Result<NoData, Error> {
    let mut url = m.api_url.clone();

    url.path_segments_mut().unwrap().push("issues");

    let mut query: Vec<(&str, JsonValue)> = vec![
        ("fields", JsonValue::String(ISSUES_FIELDS.into())),
        (
            "query",
            JsonValue::String(
                options
                    .clone()
                    .unwrap_or_default()
                    .query
                    .unwrap_or_default(),
            ),
        ),
        (
            "$top",
            JsonValue::Number(
                options
                    .clone()
                    .unwrap_or_default()
                    .page
                    .unwrap_or_default()
                    .take
                    .unwrap_or_default()
                    .into(),
            ),
        ),
        (
            "$skip",
            JsonValue::Number(
                options
                    .clone()
                    .unwrap_or_default()
                    .page
                    .unwrap_or_default()
                    .skip
                    .unwrap_or_default()
                    .into(),
            ),
        ),
    ];

    m.config.clone().issues.fields.iter().for_each(|field| {
        query.push(("customFields", JsonValue::String(field.clone())));
    });

    let req = m.client.get(url).query(&query);

    log::debug!("Youtrack issues request: {:?}", req);

    let res = req.send().await?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let json: JsonValue = res.json().await?;
            let processed = json
                .as_array()
                .unwrap()
                .iter()
                .map(|issue| process_issue(issue.clone()))
                .collect::<Result<Vec<Issue>, Error>>()?;

            log::debug!(
                "Youtrack issues matching: {:?} -> {:#?}",
                options.unwrap_or_default(),
                processed
            );
            callback.call::<()>((LuaNil, lua.to_value(&processed)))?;
        }
        _ => {
            let text = res.text().await?;

            log::debug!(
                "Youtrack issues can not be fetched: {:?} -> {:#?}",
                options.unwrap_or_default(),
                text
            );
            callback.call::<()>((
                format!("Youtrack issues can not be fetched: {:#?}", text),
                LuaNil,
            ))?;
        }
    }

    Ok(NoData)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetIssue {
    pub id: String,
}

into_lua!(GetIssue);
from_lua!(GetIssue);

pub type GetIssueArgs = (GetIssue, LuaFunction);

#[allow(unused_variables)]
pub async fn get_issue(
    lua: &Lua,
    m: AppDataRef<'_, Module>,
    (options, callback): GetIssueArgs,
) -> Result<NoData, Error> {
    let mut url = m.api_url.clone();

    url.path_segments_mut()
        .unwrap()
        .push("issues")
        .push(options.clone().id.as_str());

    let query: Vec<(&str, JsonValue)> = vec![("fields", JsonValue::String(ISSUE_FIELDS.into()))];

    let req = m.client.get(url).query(&query);

    log::debug!("Youtrack issue detail request: {:?}", req);

    let res = req.send().await?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let json: JsonValue = res.json().await?;
            let processed = process_issue(json.clone())?;

            log::debug!("Youtrack issue details: {:?} -> {:#?}", options, processed);
            callback.call::<()>((LuaNil, lua.to_value(&processed)))?;
        }
        _ => {
            let text = res.text().await?;

            log::debug!(
                "Youtrack issue details can not be fetched: {:?} -> {:#?}",
                options,
                text
            );
            callback.call::<()>((
                format!("Youtrack issue details can not be fetched: {:#?}", text),
                LuaNil,
            ))?;
        }
    }

    Ok(NoData)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateIssue {
    pub project: String,
    pub summary: String,
    pub description: Option<String>,
}

into_lua!(CreateIssue);
from_lua!(CreateIssue);

pub type CreateIssueArgs = (CreateIssue, LuaFunction);

#[allow(unused_variables)]
pub async fn create_issue(
    lua: &Lua,
    m: AppDataRef<'_, Module>,
    (options, callback): CreateIssueArgs,
) -> Result<NoData, Error> {
    let mut url = m.api_url.clone();

    url.path_segments_mut().unwrap().push("issues");

    let query: Vec<(&str, JsonValue)> = vec![("fields", JsonValue::String(ISSUE_FIELDS.into()))];

    let req = m.client.post(url).query(&query).json(&json!({
        "project": { "id": options.project },
        "summary": options.summary,
        "description": options.description
    }));

    log::debug!("Youtrack issue create request: {:?}", req);

    let res = req.send().await?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let json: JsonValue = res.json().await?;
            let processed = process_issue(json)?;
            log::debug!("Youtrack issue created: {:?} -> {:#?}", options, processed);
            callback.call::<()>((LuaNil, lua.to_value(&processed)))?;
        }
        _ => {
            log::debug!(
                "Youtrack issue can not be created: {:?} -> {:#?}",
                options,
                res.text().await?
            );
            callback.call::<()>(("Youtrack issue issue can not be created.", LuaNil))?;
        }
    }

    Ok(NoData)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateIssue {
    pub id: String,
    pub description: Option<String>,
    pub summary: Option<String>,
}

into_lua!(UpdateIssue);
from_lua!(UpdateIssue);

pub type UpdateIssueArgs = (UpdateIssue, LuaFunction);

#[allow(unused_variables)]
pub async fn update_issue(
    lua: &Lua,
    m: AppDataRef<'_, Module>,
    (options, callback): UpdateIssueArgs,
) -> Result<NoData, Error> {
    let mut url = m.api_url.clone();

    url.path_segments_mut()
        .unwrap()
        .push("issues")
        .push(options.clone().id.as_str());

    let query: Vec<(&str, JsonValue)> = vec![("fields", JsonValue::String(ISSUE_FIELDS.into()))];

    let req = m.client.post(url).query(&query).json(&json!({
        "summary": options.summary,
        "description": options.description
    }));

    log::debug!("Youtrack issue update request: {:?}", req);

    let res = req.send().await?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let json: JsonValue = res.json().await?;
            log::debug!("Youtrack issue updated: {:?} -> {:#?}", options, json);
            callback.call::<()>((LuaNil, lua.to_value(&json)))?;
        }
        _ => {
            log::debug!(
                "Youtrack issue can not be updated: {:?} -> {:#?}",
                options,
                res.text().await?
            );
            callback.call::<()>((
                format!("Youtrack issue issue can not be updated: {}", options.id),
                LuaNil,
            ))?;
        }
    }

    Ok(NoData)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApplyIssueCommand {
    pub id: String,
    pub query: String,
}

into_lua!(ApplyIssueCommand);
from_lua!(ApplyIssueCommand);

pub type ApplyIssueCommandArgs = (ApplyIssueCommand, LuaFunction);

#[allow(unused_variables)]
pub async fn apply_issue_command(
    lua: &Lua,
    m: AppDataRef<'_, Module>,
    (options, callback): ApplyIssueCommandArgs,
) -> Result<NoData, Error> {
    let mut url = m.api_url.clone();

    url.path_segments_mut().unwrap().push("commands");

    let query: Vec<(&str, JsonValue)> = vec![];

    let req = m.client.post(url).query(&query).json(&json!({
        "issues": [{ "id": options.id }],
        "query": options.query
    }));

    log::debug!("Youtrack issue apply command request: {:?}", req);

    let res = req.send().await?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let json: JsonValue = res.json().await?;
            log::debug!(
                "Youtrack issue command applied: {:?} -> {:#?}",
                options,
                json
            );
            callback.call::<()>((LuaNil, lua.to_value(&json)))?;
        }
        _ => {
            log::debug!(
                "Youtrack issue command can not be applied: {:?} -> {:#?}",
                options,
                res.text().await?
            );
            callback.call::<()>((
                format!("Youtrack issue command can not be applied: {}", options.id),
                LuaNil,
            ))?;
        }
    }

    Ok(NoData)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddIssueComment {
    pub id: String,
    pub comment: String,
}

into_lua!(AddIssueComment);
from_lua!(AddIssueComment);

pub type AddIssueCommentArgs = (AddIssueComment, LuaFunction);

#[allow(unused_variables)]
pub async fn add_issue_comment(
    lua: &Lua,
    m: AppDataRef<'_, Module>,
    (options, callback): AddIssueCommentArgs,
) -> Result<NoData, Error> {
    let mut url = m.api_url.clone();

    url.path_segments_mut()
        .unwrap()
        .push("issues")
        .push(options.clone().id.as_str())
        .push("comments");

    let query: Vec<(&str, JsonValue)> = vec![];

    let req = m.client.post(url).query(&query).json(&json!({
        "text": options.comment
    }));

    log::debug!("Youtrack issue add comment request: {:?}", req);

    let res = req.send().await?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let json: JsonValue = res.json().await?;
            log::debug!("Youtrack issue comment added: {:?} -> {:#?}", options, json);
            callback.call::<()>((LuaNil, lua.to_value(&json)))?;
        }
        _ => {
            log::debug!(
                "Youtrack issue comment can not be added: {:?} -> {:#?}",
                options,
                res.text().await?
            );
            callback.call::<()>((
                format!("Youtrack issue comment can not be added: {}", options.id),
                LuaNil,
            ))?;
        }
    }

    Ok(NoData)
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetProjects {}

into_lua!(GetProjects);
from_lua!(GetProjects);

pub type GetProjectsArgs = (Option<GetProjects>, LuaFunction);

#[allow(unused_variables)]
pub async fn get_projects(
    lua: &Lua,
    m: AppDataRef<'_, Module>,
    (options, callback): GetProjectsArgs,
) -> Result<NoData, Error> {
    let mut url = m.api_url.clone();

    url.path_segments_mut()
        .unwrap()
        .push("admin")
        .push("projects");

    let query: Vec<(&str, JsonValue)> = vec![
        ("fields", JsonValue::String(PROJECT_FIELDS.into())),
        ("archived", JsonValue::Bool(false)),
    ];

    let req = m.client.get(url).query(&query);

    log::debug!("Youtrack projects request: {:?}", req);

    let res = req.send().await?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let json: JsonValue = res.json().await?;
            let processed = json
                .as_array()
                .unwrap()
                .iter()
                .map(|project| process_project(project.clone()))
                .collect::<Result<Vec<Project>, Error>>()?;

            log::debug!(
                "Youtrack projects matching: {:?} -> {:#?}",
                options.unwrap_or_default(),
                processed
            );
            callback.call::<()>((LuaNil, lua.to_value(&processed)))?;
        }
        _ => {
            let text = res.text().await?;

            log::debug!(
                "Youtrack projects can not be fetched: {:?} -> {:#?}",
                options.unwrap_or_default(),
                text
            );
            callback.call::<()>((
                format!("Youtrack projects can not be fetched: {:#?}", text),
                LuaNil,
            ))?;
        }
    }

    Ok(NoData)
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetAgiles {}

into_lua!(GetAgiles);
from_lua!(GetAgiles);

pub type GetAgilesArgs = (Option<GetAgiles>, LuaFunction);

#[allow(unused_variables)]
pub async fn get_agiles(
    lua: &Lua,
    m: AppDataRef<'_, Module>,
    (options, callback): GetAgilesArgs,
) -> Result<NoData, Error> {
    let mut url = m.api_url.clone();

    url.path_segments_mut().unwrap().push("agiles");

    let query: Vec<(&str, JsonValue)> = vec![("fields", JsonValue::String(AGILES_FIELDS.into()))];

    let req = m.client.get(url).query(&query);

    log::debug!("Youtrack agiles request: {:?}", req);

    let res = req.send().await?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let json: JsonValue = res.json().await?;
            let processed = json
                .as_array()
                .unwrap()
                .iter()
                .map(|agile| process_agile(agile.clone()))
                .collect::<Result<Vec<Agile>, Error>>()?;
            log::debug!(
                "Youtrack agiles matching: {:?} -> {:#?}",
                options.unwrap_or_default(),
                processed
            );
            callback.call::<()>((LuaNil, lua.to_value(&processed)))?;
        }
        _ => {
            let text = res.text().await?;

            log::debug!(
                "Youtrack agiles can not be fetched: {:?} -> {:#?}",
                options.unwrap_or_default(),
                text
            );
            callback.call::<()>((
                format!("Youtrack agiles can not be fetched: {:#?}", text),
                LuaNil,
            ))?;
        }
    }

    Ok(NoData)
}

fn process_saved_query(query: JsonValue) -> Result<SavedQuery, Error> {
    Ok(SavedQuery {
        id: query.get("id").unwrap().as_str().unwrap().to_string(),
        name: query.get("name").unwrap().as_str().unwrap().to_string(),
        query: query.get("query").unwrap().as_str().unwrap().to_string(),
    })
}

fn process_issue(issue: JsonValue) -> Result<Issue, Error> {
    let project = Project {
        id: issue
            .get("project")
            .unwrap()
            .get("id")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string(),
        name: issue
            .get("project")
            .unwrap()
            .get("shortName")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string(),
        text: issue
            .get("project")
            .unwrap()
            .get("name")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string(),
    };

    let fields = process_fields(
        issue
            .get("customFields")
            .unwrap()
            .as_array()
            .unwrap()
            .to_vec(),
    )?;

    let tags = issue
        .get("tags")
        .map(|tags| {
            tags.as_array()
                .unwrap()
                .iter()
                .map(|tag| {
                    Ok(Tag {
                        id: tag.get("id").unwrap().as_str().unwrap().to_string(),
                        name: tag.get("name").unwrap().as_str().unwrap().to_string(),
                        color: tag.get("color").unwrap().clone(),
                    })
                })
                .collect::<Result<Vec<Tag>, Error>>()
        })
        .transpose()?
        .unwrap_or(vec![]);

    let mut result = Issue {
        id: issue.get("id").unwrap().as_str().unwrap().to_string(),
        text: issue
            .get("idReadable")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string(),
        summary: issue.get("summary").unwrap().as_str().unwrap().to_string(),
        description: None,
        project,
        fields,
        tags,
        comments: None,
    };

    if let Some(field) = issue.get("comments") {
        if let Some(comments) = field.as_array() {
            result.comments = comments
                .iter()
                .rev()
                .map(|comment| {
                    comment.as_object().map(|comment| {
                        let date = DateTime::from_timestamp_millis(
                            comment.get("created").unwrap().as_i64().unwrap(),
                        )
                        .unwrap()
                        .with_timezone(&Local);

                        Comment {
                            author: comment
                                .get("author")
                                .unwrap()
                                .get("fullName")
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_string(),
                            text: comment
                                .get("text")
                                .unwrap()
                                .as_str()
                                .unwrap_or("[No text]")
                                .to_string(),
                            created_at: date.format("%FT%T").to_string(),
                        }
                    })
                })
                .collect::<Option<Vec<Comment>>>();
        }
    }

    if let Some(description) = issue.get("description").unwrap().as_str() {
        result.description = Some(description.to_string());
    }

    Ok(result)
}

fn process_fields(fields: Vec<JsonValue>) -> Result<Vec<Field>, Error> {
    let mut result = vec![];

    fields.iter().for_each(|field| {
        if field["value"].is_null() {
            result.push(Field {
                id: field.get("id").unwrap().as_str().unwrap().to_string(),
                name: field.get("name").unwrap().as_str().unwrap().to_string(),
                text: "None".to_string(),
                value: None,
                values: None,
            });

            return;
        } else if field["value"].is_array() && field["value"].as_array().unwrap().is_empty() {
            result.push(Field {
                id: field.get("id").unwrap().as_str().unwrap().to_string(),
                name: field.get("name").unwrap().as_str().unwrap().to_string(),
                text: "[None]".to_string(),
                value: None,
                values: None,
            });

            return;
        }

        match field.get("$type").unwrap().as_str().unwrap() {
            "SimpleIssueCustomField" => {
                let value = field.get("value").unwrap();

                result.push(Field {
                    id: field.get("id").unwrap().as_str().unwrap().to_string(),
                    name: field.get("name").unwrap().as_str().unwrap().to_string(),
                    text: value.to_string(),
                    value: None,
                    values: None,
                })
            }
            "DateIssueCustomField" => {
                let value = field.get("value").unwrap().as_i64().unwrap();

                let date = DateTime::from_timestamp_millis(value).unwrap();

                result.push(Field {
                    id: field.get("id").unwrap().as_str().unwrap().to_string(),
                    name: field.get("name").unwrap().as_str().unwrap().to_string(),
                    text: date.format("%F").to_string(),
                    value: Some(JsonValue::Number(value.into())),
                    values: None,
                })
            }
            "PeriodIssueCustomField" => {
                let value = field.get("value").unwrap().as_object().unwrap();

                result.push(Field {
                    id: field.get("id").unwrap().as_str().unwrap().to_string(),
                    name: field.get("name").unwrap().as_str().unwrap().to_string(),
                    text: value
                        .clone()
                        .get("presentation")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                    value: Some(JsonValue::Object(value.clone())),
                    values: None,
                })
            }
            _ => {
                let value = field.get("value").unwrap();
                if value.is_array() {
                    let values = value.as_array().unwrap();

                    result.push(Field {
                        id: field.get("id").unwrap().as_str().unwrap().to_string(),
                        name: field.get("name").unwrap().as_str().unwrap().to_string(),
                        text: values
                            .iter()
                            .map(|v| v.get("name").unwrap().as_str().unwrap())
                            .collect::<Vec<&str>>()
                            .join(", "),
                        value: None,
                        values: Some(values.clone()),
                    })
                } else {
                    result.push(Field {
                        id: field.get("id").unwrap().as_str().unwrap().to_string(),
                        name: field.get("name").unwrap().as_str().unwrap().to_string(),
                        text: value.get("name").unwrap().as_str().unwrap().to_string(),
                        value: Some(value.clone()),
                        values: None,
                    })
                }
            }
        }
    });

    Ok(result)
}

fn process_project(project: JsonValue) -> Result<Project, Error> {
    Ok(Project {
        id: project.get("id").unwrap().as_str().unwrap().to_string(),
        name: project
            .get("shortName")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string(),
        text: project.get("name").unwrap().as_str().unwrap().to_string(),
    })
}

fn process_agile(agile: JsonValue) -> Result<Agile, Error> {
    Ok(Agile {
        id: agile.get("id").unwrap().as_str().unwrap().to_string(),
        name: agile.get("name").unwrap().as_str().unwrap().to_string(),
        projects: agile
            .get("projects")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|project| process_project(project.clone()))
            .collect::<Result<Vec<Project>, Error>>()?,
        sprints: agile
            .get("sprints")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|sprint| {
                Ok(Sprint {
                    id: sprint.get("id").unwrap().as_str().unwrap().to_string(),
                    name: sprint.get("name").unwrap().as_str().unwrap().to_string(),
                    is_default: sprint.get("isDefault").unwrap().as_bool().unwrap(),
                })
            })
            .collect::<Result<Vec<Sprint>, Error>>()?,
    })
}
