use crate::term;

pub fn install_hooks() {
    better_panic::install();
    // better_panic::Settings::auto()
    //     .most_recent_first(false)
    //     .verbosity(better_panic::Verbosity::Full)
    //     .install()

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        term::restore();
        let backtrace = std::backtrace::Backtrace::capture();
        tracing::info!("My backtrace: {:#?}", backtrace);
        hook(info);
    }));
}
