use std::time::Duration;

use app::App;
use tokio::time::sleep;

mod action;
mod app;
mod errors;
mod events;
mod plugin;
mod ro_cell;
mod state;
mod term;
mod widgets;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    errors::install_hooks()?;
    crate::plugin::init_lua();

    // let term = term::init()?;
    // let events = events::Events::new();
    // App::new().run(term, events).await?;
    //
    // term::restore()?;
    sleep(Duration::from_millis(3000)).await;
    Ok(())
}
