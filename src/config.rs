use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::macros::{from_lua, into_lua};

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct Config {
    #[validate(url(message = "URL should be a full url of your Youtrack instance."))]
    pub url: String,

    pub token: String,

    pub issues: ConfigIssuesIssues,
    pub issue: ConfigIssuesIssue,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigIssuesIssues {
    pub fields: Vec<String>,
}

from_lua!(ConfigIssuesIssues);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigIssuesIssue {
    pub fields: Vec<String>,
}

from_lua!(ConfigIssuesIssue);

impl Config {}

impl FromLua for Config {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let c: Config = lua.from_value(value)?;

        match c.validate() {
            Ok(_) => Ok(c),
            Err(err) => Err(LuaError::external(err)),
        }
    }
}

into_lua!(Config);
