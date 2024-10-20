use std::{pin::Pin, time::Duration};

use crossterm::event::{Event as CrosstermEvent, *};
use futures::{Stream, StreamExt};
use tokio::{sync::mpsc, time::interval};
use tokio_stream::{wrappers::IntervalStream, StreamMap};

use crate::{ro_cell::RoCell, Keymap};

static TX: RoCell<mpsc::UnboundedSender<Event>> = RoCell::new();

#[derive(Default)]
pub struct Events {
    streams: StreamMap<StreamName, Pin<Box<dyn Stream<Item = Event>>>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum StreamName {
    Render,
    Crossterm,
    Text,
}

pub enum Event {
    Init,
    Quit,
    Error,
    Closed,
    Tick,
    Render,
    Command(String),
    Crossterm(CrosstermEvent),
}

pub fn emit(e: Event) {
    TX.send(e).unwrap();
}

impl Events {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();
        TX.init(tx);

        let stream = async_stream::stream! {
            while let Some(item) = rx.recv().await {
                yield item;
            }
        };
        let stream = Box::pin(stream);

        Self {
            streams: StreamMap::from_iter([
                (StreamName::Render, render_stream()),
                (StreamName::Crossterm, crossterm_stream()),
                (StreamName::Text, stream),
            ]),
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.streams.next().await.map(|(_name, event)| event)
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
