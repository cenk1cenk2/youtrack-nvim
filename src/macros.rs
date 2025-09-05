macro_rules! into_lua {
    ($structname: ident) => {
        impl mlua::IntoLua for $structname {
            fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
                lua.to_value(&self)
            }
        }
    };
}

macro_rules! from_lua {
    ($structname: ident) => {
        impl mlua::FromLua for $structname {
            fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
                lua.from_value(value)
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! export_fn {
    ($lua:expr, $exports:expr, $name:expr, $fn:expr) => {
        $exports.set(
            $name.unwrap_or(stringify!($fn)),
            $lua.create_function(move |lua: &Lua, args| {
                let m = lua.app_data_ref::<Module>().ok_or_else(|| Error::NoSetup)?;

                $fn(lua, m, args).map_err(|err| err.into_lua_err())
            })?,
        )
    };
}

macro_rules! export_async_fn {
    ($lua:expr, $exports:expr, $name: expr, $fn:expr, $args: ty) => {
        $exports.set(
            $name.unwrap_or(stringify!($fn)),
            $lua.create_function(move |lua: &Lua, args: $args| {
                let _ = lua
                    .app_data_ref::<Module>()
                    .ok_or_else(|| Error::NoSetup.into_lua_err())?;

                let f = lua
                    .create_async_function(|lua: Lua, args: $args| async move {
                        // TODO: this is validated outside, can not pass this due to borrowing reasons, have to figure that out again
                        let m = lua.app_data_ref::<Module>().unwrap();
                        $fn(&lua, m, args).await.map_err(|err| err.into_lua_err())?;

                        Ok(LuaValue::Nil)
                    })?
                    .bind(args)?;

                lua.load(mlua::chunk! {
                    local coroutine = coroutine.wrap($f)
                    local step = function() end
                    step = function()
                        if coroutine() ~= nil then
                            vim.schedule(step)
                        end
                    end
                    step()

                })
                .exec()?;

                Ok(())
            })?,
        )
    };
}

pub(crate) use export_async_fn;
pub(crate) use from_lua;
pub(crate) use into_lua;
