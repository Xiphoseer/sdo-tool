pub mod bimc;
pub mod font;
pub mod opt;
pub mod sdoc;
mod util;

/// Set up CLI
pub fn init<T: clap::Parser>() -> color_eyre::Result<T> {
    color_eyre::install()?;
    let mut builder = pretty_env_logger::formatted_builder();
    builder.filter_level(log::LevelFilter::Info);
    if let Ok(s) = ::std::env::var("SDO_TOOL_LOG") {
        builder.parse_filters(&s);
    }
    builder.init();
    let args = T::try_parse()?;
    Ok(args)
}
