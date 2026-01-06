use env_logger::Env;
use log::LevelFilter;

pub mod bimc;
pub mod font;
pub mod opt;
pub mod sdoc;
mod util;

/// Set up CLI
pub fn init<T: clap::Parser>() -> color_eyre::Result<T> {
    color_eyre::install()?;
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .format_timestamp(None)
        .parse_env(Env::new().filter("SDO_TOOL_LOG"))
        .init();
    let args = T::parse();
    Ok(args)
}
