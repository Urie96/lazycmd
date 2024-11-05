use mlua::prelude::*;
use std::{hash::Hash, pin::Pin, time::Duration};

use crossterm::event::{Event as CrosstermEvent, *};
use futures::{Stream, StreamExt};
use tokio::{
    sync::mpsc::{self},
    time::interval,
};
use tokio_stream::{wrappers::IntervalStream, StreamMap};

use crate::{Keymap, PageEntry};

pub struct Events {
    tx: EventSender,
    streams: StreamMap<StreamName, Pin<Box<dyn Stream<Item = Event>>>>,
}

pub type EventSender = mpsc::UnboundedSender<Event>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum StreamName {
    Render,
    Crossterm,
    Text,
}

pub enum Event {
    Quit,
    Render,
    Enter(Vec<String>),
    Command(String),
    Crossterm(CrosstermEvent),
    AddKeymap(Keymap),
    PageSetEntries(Vec<PageEntry>),
    AddEventHook(EventName, LuaFunction),
    LuaCallback(Box<dyn FnOnce()>),
}

#[derive(PartialEq, Eq, Hash)]
pub enum EventName {
    Quit,
    Render,
    Enter,
    Command,
    Crossterm,
    AddKeymap,
    PageSetEntries,
    AddEventHook,
    LuaCallback,
}

impl FromLua for EventName {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        Ok(match String::from_lua(value, lua)?.as_str() {
            "enter" => EventName::Enter,
            other => Err(format!("Unable to cast string '{other}' into EventName").into_lua_err())?,
        })
    }
}

impl From<&Event> for EventName {
    fn from(value: &Event) -> Self {
        match value {
            Event::Quit => EventName::Quit,
            Event::Enter(_) => EventName::Enter,
            Event::Render => EventName::Render,
            Event::Command(_) => EventName::Command,
            Event::Crossterm(_) => EventName::Crossterm,
            Event::AddKeymap(_) => EventName::AddKeymap,
            Event::PageSetEntries(_) => EventName::PageSetEntries,
            Event::AddEventHook(_, _) => EventName::AddEventHook,
            Event::LuaCallback(_) => EventName::LuaCallback,
        }
    }
}

impl Events {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();

        let stream = async_stream::stream! {
            while let Some(item) = rx.recv().await {
                yield item;
            }
        };
        let stream = Box::pin(stream);

        Self {
            tx,
            streams: StreamMap::from_iter([
                (StreamName::Render, render_stream()),
                (StreamName::Crossterm, crossterm_stream()),
                (StreamName::Text, stream),
            ]),
        }
    }

    pub fn send(&self, event: Event) {
        self.tx.send(event).unwrap()
    }

    pub fn sender(&self) -> EventSender {
        self.tx.clone()
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.streams.next().await.map(|(_name, event)| event)
    }
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

fn render_stream() -> Pin<Box<dyn Stream<Item = Event>>> {
    let render_delay = Duration::from_secs_f64(20.0);
    let render_interval = interval(render_delay);
    Box::pin(IntervalStream::new(render_interval).map(|_| Event::Render))
}

fn crossterm_stream() -> Pin<Box<dyn Stream<Item = Event>>> {
    Box::pin(EventStream::new().fuse().filter_map(|event| async move {
        match event {
            // Ignore key release / repeat events
            Ok(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Release => None,
            Ok(event) => Some(Event::Crossterm(event)),
            Err(e) => panic!("{}", e),
        }
    }))
}
