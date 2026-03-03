pub use app::App;
pub use events::Event;
pub use keymap::*;
pub use mode::*;
pub use page::*;
pub use state::*;
pub use widgets::InputState;
pub use state::{ConfirmButton, ConfirmDialog};
use tokio::task;

mod app;
mod confirm_handler;
mod errors;
mod events;
mod input_handler;
mod keymap;
mod log;
mod mode;
mod page;
mod plugin;
mod state;
mod term;
mod widgets;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    log::Logs::start()?;
    errors::install_hooks();
    let local = task::LocalSet::new();

    // Run the local task set.
    local
        .run_until(async move {
            // Initialize terminal first (required for crossterm event stream)
            let term = term::init()?;

            let events = events::Events::new();

            // Get plugin name from command line argument
            let args: Vec<String> = std::env::args().collect();
            let plugin_name = if args.len() >= 2 { Some(args[1].clone()) } else { None };

            let mut app = App::new(events.sender(), term, plugin_name);

            if let Err(e) = app.run(events).await {
                term::restore();
                eprintln!("Error: {}", e);
                return Err(e);
            }

            term::restore();
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
