use mlua::{chunk, prelude::*, Lua};

pub fn main() -> mlua::Result<()> {
    let lua = Lua::new();
    set_global(&lua)?;

    let res: i64 = lua
        .load(chunk! {
        print(lc.event.on("a"))
        return 1
        })
        .call(())?;

    println!("{res}");

    Ok(())
}

fn set_global(lua: &Lua) -> mlua::Result<()> {
    let lua_event_on = LuaFunction::wrap_raw(event_on);

    lua.load(chunk!(
    lc = {
        event = {
            on = $lua_event_on
        }
    }
    ))
    .exec()?;
    Ok(())
}

fn event_on(s: String) -> String {
    s.to_string()
}
