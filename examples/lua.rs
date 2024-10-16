use mlua::{prelude::*, Lua};
use std::{sync::LazyLock, time::Duration};

static LUA: LazyLock<Lua> = LazyLock::new(Lua::new);

#[tokio::main]
async fn main() -> mlua::Result<()> {
    // let lua = Lua::new();
    let set_timeout = LUA.create_function(|_, (callback, time): (LuaFunction, u64)| {
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(time)).await;
            let _ = callback.call::<()>(());
        });
        Ok(())
    })?;

    LUA.globals().set("set_timeout", set_timeout)?;

    LUA.load(
        r#"
local a=1
set_timeout(function() print("foo",a) end, 1000)
set_timeout(function() a=2 end, 500)
print("global end")
"#,
    )
    .exec()?;
    dbg!("asdf");
    tokio::time::sleep(Duration::from_millis(2000)).await;

    Ok(())
}
