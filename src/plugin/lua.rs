use std::{cell::RefCell, rc::Rc};

use mlua::Lua;
use tokio::sync::mpsc;

use super::lc;
use crate::{Event, State};

pub struct PluginRunner {
    pub(super) lua: Lua,

    pub(super) event_sender: mpsc::UnboundedSender<Event>,

    pub(super) state: Rc<RefCell<State>>,
}

impl PluginRunner {
    pub async fn new(
        event_sender: mpsc::UnboundedSender<Event>,
        state: Rc<RefCell<State>>,
    ) -> Self {
        let lua = Lua::new();
        // lc::set_lua_global(&lua).unwrap();

        let s = Self {
            lua,
            event_sender,
            state,
        };
        lc::register(&s).unwrap();

        macro_rules! preset {
            ($name:literal) => {{
                #[cfg(debug_assertions)]
                {
                    std::fs::read(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/preset/",
                        $name,
                        ".lua"
                    ))
                    .expect(concat!(
                        "Failed to read 'yazi-plugin/preset/",
                        $name,
                        ".lua'"
                    ))
                }
                #[cfg(not(debug_assertions))]
                {
                    &include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/preset/",
                        $name,
                        ".lua"
                    ))[..]
                }
            }};
        }

        s.lua
            .load(preset!("init"))
            .call_async::<()>(())
            .await
            .unwrap();
        s
    }
}
