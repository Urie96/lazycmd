use std::ffi::OsString;

pub use app::App;
pub use events::Event;
pub use keymap::*;
pub use mode::*;
pub use page::*;
pub use state::*;
pub use state::{ConfirmButton, ConfirmDialog, SelectDialog, SelectOption};
use tokio::task;
pub use widgets::InputDialogState;
pub use widgets::InputState;

mod app;
mod confirm_handler;
mod errors;
mod events;
mod input_handler;
mod keymap;
mod log;
mod mode;
mod path_codec;
mod page;
mod plugin;
mod select_handler;
mod state;
mod term;
mod widgets;

fn parse_initial_path(args: impl IntoIterator<Item = OsString>) -> anyhow::Result<Vec<String>> {
    let mut args = args.into_iter();
    let _program = args.next();

    let Some(raw_path) = args.next() else {
        return Ok(Vec::new());
    };

    if args.next().is_some() {
        anyhow::bail!("Usage: lazycmd [initial-path]");
    }

    let raw_path = raw_path
        .into_string()
        .map_err(|_| anyhow::anyhow!("initial path must be valid UTF-8"))?;
    let trimmed = raw_path.trim_matches('/');

    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    Ok(trimmed
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(path_codec::decode_path_segment_input)
        .collect::<anyhow::Result<Vec<_>>>()?)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    log::Logs::start()?;
    errors::install_hooks();
    let initial_path = parse_initial_path(std::env::args_os())?;
    let local = task::LocalSet::new();

    // Run the local task set.
    local
        .run_until(async move {
            // Initialize terminal first (required for crossterm event stream)
            let term = term::init()?;

            let events = events::Events::new();

            let mut app = App::new(events.sender(), term, initial_path);

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

#[cfg(test)]
mod tests {
    use super::parse_initial_path;
    use std::ffi::OsString;

    fn os_args(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn parse_initial_path_defaults_to_root() {
        assert_eq!(
            parse_initial_path(os_args(&["lazycmd"])).unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn parse_initial_path_splits_segments() {
        assert_eq!(
            parse_initial_path(os_args(&["lazycmd", "/docker/container"])).unwrap(),
            vec!["docker".to_string(), "container".to_string()]
        );
    }

    #[test]
    fn parse_initial_path_normalizes_repeated_slashes() {
        assert_eq!(
            parse_initial_path(os_args(&["lazycmd", "docker//container/"])).unwrap(),
            vec!["docker".to_string(), "container".to_string()]
        );
    }

    #[test]
    fn parse_initial_path_rejects_extra_args() {
        assert!(parse_initial_path(os_args(&["lazycmd", "/docker", "/extra"])).is_err());
    }

    #[test]
    fn parse_initial_path_decodes_percent_encoded_segments() {
        assert_eq!(
            parse_initial_path(os_args(&["lazycmd", "/github/repo/tpope/vim-abolish/tags/feature%2Ftest"]))
                .unwrap(),
            vec![
                "github".to_string(),
                "repo".to_string(),
                "tpope".to_string(),
                "vim-abolish".to_string(),
                "tags".to_string(),
                "feature/test".to_string(),
            ]
        );
    }
}
