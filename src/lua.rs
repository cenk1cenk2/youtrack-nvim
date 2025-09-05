use mlua::prelude::*;
use mlua::{FromLua, IntoLua};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NoData;

impl FromLua for NoData {
    fn from_lua(_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        Ok(NoData)
    }
}

impl IntoLua for NoData {
    fn into_lua(self, _lua: &Lua) -> LuaResult<LuaValue> {
        Ok(LuaValue::Nil)
    }
}
