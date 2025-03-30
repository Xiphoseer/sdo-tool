use color_eyre::eyre;
use signum::chsets::v2::parse_chset_v2;

use crate::cli::{opt::Options, util};

pub fn process_cset_v2(input: &[u8], _opt: Options) -> eyre::Result<()> {
    let (_, cset) = util::load_partial(parse_chset_v2, input)?;
    log::info!("{:#?}", cset.chset001);
    log::info!("{:#?}", cset.fdeskr01);
    log::info!("{:#?}", cset.lgtab001);
    log::info!("{:#?}", cset.chars001);
    log::info!("{:#?}", cset.kerntab1);
    Ok(())
}
