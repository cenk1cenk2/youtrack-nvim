use std::{
    fmt::{self, Display},
    io::{self},
    sync::Arc,
};

use log::SetLoggerError;
use mlua::prelude::LuaError;

#[derive(Debug)]
pub enum Error {
    NoSetup,
    Str(String),
    Std(Box<dyn std::error::Error + Send + Sync>),
    Validation(validator::ValidationErrors),
    HttpClient(reqwest::Error),
    Api,
    Url(url::ParseError),
    Lua(mlua::Error),
    Logger(SetLoggerError),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            NoSetup => write!(
                f,
                "Library did not get setup correctly. Did you call setup?"
            ),
            Str(ref err) => write!(f, "{}", err),
            Std(ref err) => <dyn std::error::Error as fmt::Display>::fmt(&**err, f),
            Validation(ref err) => <validator::ValidationErrors as fmt::Display>::fmt(err, f),
            HttpClient(ref err) => <reqwest::Error as fmt::Display>::fmt(err, f),
            Api => write!(f, "API returned an unexpected result."),
            Url(ref err) => <url::ParseError as fmt::Display>::fmt(err, f),
            Lua(ref err) => <LuaError as fmt::Display>::fmt(err, f),
            Logger(ref err) => <SetLoggerError as fmt::Display>::fmt(err, f),
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Std(err)
    }
}

impl From<validator::ValidationErrors> for Error {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::Validation(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_status() {
            return Self::Str(
                err.status()
                    .unwrap()
                    .canonical_reason()
                    .unwrap()
                    .to_string(),
            );
        }

        Self::HttpClient(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Self::Url(err)
    }
}

impl From<SetLoggerError> for Error {
    fn from(err: SetLoggerError) -> Self {
        Self::Logger(err)
    }
}

impl From<LuaError> for Error {
    fn from(err: LuaError) -> Self {
        Self::Lua(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Std(Box::new(err))
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Std(boxed_err) => {
                if let Some(io_err) = boxed_err.downcast_ref::<io::Error>() {
                    io::Error::new(io_err.kind(), format!("{}", io_err))
                } else {
                    io::Error::new(io::ErrorKind::Other, boxed_err)
                }
            }
            Error::Lua(lua_err) => {
                io::Error::new(io::ErrorKind::Other, format!("Lua error: {}", lua_err))
            }
            other => io::Error::new(io::ErrorKind::Other, format!("{}", other)),
        }
    }
}

impl From<Error> for mlua::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Lua(err) => err,
            err => LuaError::ExternalError(Arc::new(err)),
        }
    }
}
