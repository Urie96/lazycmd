use mlua::prelude::*;

#[derive(Debug)]
struct State(i32);

fn main() -> LuaResult<()> {
    let mut s = State(5);

    let lua = Lua::new();
    lua.scope(|sco| {
        lua.set_named_registry_value("state", sco.create_any_userdata_ref_mut(&mut s)?)?;
        let state: LuaAnyUserData = lua.named_registry_value("state")?;
        state.borrow_mut_scoped::<State, _>(|state| state.0 = 1)?;
        state.borrow_scoped::<State, _>(|state| {
            dbg!(state.0);
        })?;
        // state.borrow_scoped(|state:|)
        Ok(())
    })?;

    let state: LuaAnyUserData = lua.named_registry_value("state")?;
    state.borrow_mut_scoped::<State, _>(|state| state.0 = 1)?;
    state.borrow_scoped::<State, _>(|state| {
        dbg!(state.0);
    })?;

    println!("{:?}", s.0);

    // lua.load("print(state:get())").exec()?;

    Ok(())
}
