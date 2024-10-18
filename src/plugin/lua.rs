use mlua::{chunk, prelude::*, Lua, Table};
use tokio::time::{sleep, Duration};

use crate::preset;
use crate::ro_cell::RoCell;

pub static LUA: RoCell<Lua> = RoCell::new();

pub fn init_lua() -> mlua::Result<()> {
    let lua = Lua::new();
    let event_on = LuaFunction::wrap_raw(event_on);
    let defer_fn = lua.create_function(|_, (f, ms): (LuaFunction, u64)| {
        tokio::spawn(async move {
            sleep(Duration::from_millis(ms)).await;
            f.call::<()>(()).expect("asd");
        });
        Ok(())
    })?;
    lua.load(chunk!(
        lc = {
            event = {
               on = $event_on,
            },
            defer_fn = $defer_fn,
        }
    ))
    .exec()?;

    let m: Table = lua.load(preset!("init")).call(())?;
    let items: Vec<String> = m.call_method("list", ["a", "b", "c"])?;

    println!("{:?}", items);

    LUA.init(lua);
    Ok(())
}

fn event_on() {
    println!("1");
}
