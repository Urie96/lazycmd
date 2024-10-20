use mlua::{prelude::*, Lua};
use tokio::time::{sleep, Duration};

pub fn new(lua: &Lua) -> mlua::Result<LuaFunction> {
    lua.create_function(|_, (f, ms): (LuaFunction, u64)| {
        tokio::task::spawn_local(async move {
            sleep(Duration::from_millis(ms)).await;
            f.call::<()>(()).expect("asd");
        });
        Ok(())
    })
}
