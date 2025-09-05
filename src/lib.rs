use crate::config::Config;
use client::*;
use error::Error;
use lua::NoData;
use macros::export_async_fn;
use mlua::prelude::*;
use reqwest::header::{self, HeaderMap};
use structured_logger::Builder;
use tokio::runtime::Runtime;
use url::Url;
use writer::LuaWriter;

mod client;
mod config;
mod error;
mod lua;
mod macros;
mod writer;

#[derive(Clone)]
struct Module {
    pub config: Config,
    pub client: reqwest::Client,
    pub api_url: Url,
}

impl Module {
    fn setup(lua: &Lua, config: Config) -> Result<NoData, Error> {
        let _ = Builder::with_level(log::Level::Trace.as_str())
            .with_target_writer("*", LuaWriter::new(lua, "youtrack.log")?.get())
            .try_init()
            .map(|_| {
                log::debug!("Setup the logger for the library.");
            });

        let mut headers = HeaderMap::new();

        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(format!("Bearer {}", config.token).as_str()).map_err(
                |err| {
                    log::error!("Failed to create the authorization header: {}", err);
                    error::Error::NoSetup
                },
            )?,
        );

        let api_url = Url::parse(config.url.as_str())?.join("/api")?;

        let client = reqwest::Client::builder()
            .user_agent("youtrack-nvim")
            .default_headers(headers)
            .build()?;
        log::debug!("Setup the client with url: {}", api_url);

        let guard = RUNTIME.enter();
        lua.set_app_data(guard);

        lua.set_app_data(Self {
            config,
            client,
            api_url,
        });

        Ok(NoData {})
    }
}

static RUNTIME: once_cell::sync::Lazy<Runtime> = once_cell::sync::Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create runtime.")
});

#[mlua::lua_module(skip_memory_check)]
pub fn youtrack_lib(lua: &Lua) -> mlua::Result<LuaTable> {
    let exports = lua.create_table()?;

    exports.set(
        "setup",
        lua.create_function(|lua: &Lua, args| {
            Module::setup(lua, args).map_err(|err| err.into_lua_err())
        })?,
    )?;

    export_async_fn!(
        lua.clone(),
        exports,
        None,
        get_saved_queries,
        GetSavedQueriesArgs
    )?;
    export_async_fn!(lua, exports, None, get_issues, GetIssuesArgs)?;
    export_async_fn!(lua, exports, None, get_issue, GetIssueArgs)?;
    export_async_fn!(lua, exports, None, update_issue, UpdateIssueArgs)?;
    export_async_fn!(
        lua,
        exports,
        None,
        apply_issue_command,
        ApplyIssueCommandArgs
    )?;
    export_async_fn!(lua, exports, None, add_issue_comment, AddIssueCommentArgs)?;
    export_async_fn!(lua, exports, None, get_projects, GetProjectsArgs)?;
    export_async_fn!(lua, exports, None, create_issue, CreateIssueArgs)?;
    export_async_fn!(lua, exports, None, get_agiles, GetAgilesArgs)?;

    Ok(exports)
}
