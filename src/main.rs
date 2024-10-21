use std::time::Duration;

pub use app::App;
pub use events::Event;
pub use keymap::*;
pub use mode::*;
pub use page::*;
pub use state::*;
use tokio::{task, time::sleep};

mod action;
mod app;
mod errors;
mod events;
mod keymap;
mod log;
mod mode;
mod page;
mod plugin;
mod ro_cell;
mod state;
mod term;
mod widgets;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let local = task::LocalSet::new();

    // Run the local task set.
    local
        .run_until(async move {
            log::Logs::start()?;
            errors::install_hooks()?;
            state::init();
            plugin::init()?;

            let events = events::Events::new();
            App::new().run(events).await?;

            term::restore()?;
            sleep(Duration::from_millis(3000)).await;
            Ok::<_, anyhow::Error>(())
        })
        .await?;

    // errors::install_hooks()?;
    // state::init();
    // plugin::init()?;
    //
    // let term = term::init()?;
    // let events = events::Events::new();
    // App::new().run(term, events).await?;
    // //
    // term::restore()?;
    // sleep(Duration::from_millis(3000)).await;
    Ok(())
}
