use std::{hash::Hash, pin::Pin, time::Duration};

use crossterm::event::{Event as CrosstermEvent, *};
use futures::{Stream, StreamExt};
use tokio::{sync::mpsc, time::interval};
use tokio_stream::{wrappers::IntervalStream, StreamMap};

use crate::{Keymap, PageEntry, TapKeyAsyncCallback};

pub struct Events {
    tx: mpsc::UnboundedSender<Event>,
    streams: StreamMap<StreamName, Pin<Box<dyn Stream<Item = Event>>>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum StreamName {
    Render,
    Crossterm,
    Text,
}

pub enum Event {
    Quit,
    Error,
    Render,
    Enter(Vec<String>),
    Command(String),
    Crossterm(CrosstermEvent),
    AddKeymap(Keymap),
    PageSetEntries(Vec<PageEntry>),
    AddEventHook(EventName, TapKeyAsyncCallback),
}

#[derive(PartialEq, Eq, Hash)]
pub enum EventName {
    Quit,
    Error,
    Render,
    Enter,
    Command,
    Crossterm,
    AddKeymap,
    PageSetEntries,
    AddEventHook,
}

impl From<&Event> for EventName {
    fn from(value: &Event) -> Self {
        match value {
            Event::Quit => EventName::Quit,
            Event::Enter(_) => EventName::Enter,
            Event::Error => EventName::Error,
            Event::Render => EventName::Render,
            Event::Command(_) => EventName::Command,
            Event::Crossterm(_) => EventName::Crossterm,
            Event::AddKeymap(_) => EventName::AddKeymap,
            Event::PageSetEntries(_) => EventName::PageSetEntries,
            Event::AddEventHook(_, _) => EventName::AddEventHook,
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

    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
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
    let render_delay = Duration::from_secs_f64(1.0);
    let render_interval = interval(render_delay);
    Box::pin(IntervalStream::new(render_interval).map(|_| Event::Render))
}

fn crossterm_stream() -> Pin<Box<dyn Stream<Item = Event>>> {
    Box::pin(EventStream::new().fuse().filter_map(|event| async move {
        match event {
            // Ignore key release / repeat events
            Ok(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Release => None,
            Ok(event) => Some(Event::Crossterm(event)),
            Err(_) => Some(Event::Error),
        }
    }))
}
