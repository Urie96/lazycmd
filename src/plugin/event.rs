use mlua::UserData;

struct Event;

impl UserData for Event {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {}

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {}

    fn register(registry: &mut mlua::UserDataRegistry<Self>) {
        Self::add_fields(registry);
        Self::add_methods(registry);
    }
}
