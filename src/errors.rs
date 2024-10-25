use anyhow::Result;

use crate::term;

pub fn install_hooks() -> Result<()> {
    install_better_panic();

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        term::restore().unwrap();
        let backtrace = std::backtrace::Backtrace::capture();
        tracing::info!("My backtrace: {:#?}", backtrace);
        hook(info);
    }));

    Ok(())
}

fn install_better_panic() {
    better_panic::Settings::auto()
        .most_recent_first(false)
        .verbosity(better_panic::Verbosity::Full)
        .install()
}
