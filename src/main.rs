use app::App;

mod action;
mod app;
mod errors;
mod events;
mod lua;
mod ro_cell;
mod state;
mod term;
mod widgets;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    errors::install_hooks()?;

    let term = term::init()?;
    let events = events::Events::new();
    App::new().run(term, events).await?;

    term::restore()?;
    Ok(())
}
