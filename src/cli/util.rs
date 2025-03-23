use nom_supreme::{
    error::ErrorTree,
    final_parser::{ByteOffset, ExtractContext},
};
use signum::{
    docs::Chunk,
    nom::{Finish, IResult},
    util::Buf,
};

pub(super) fn to_err_tree<'a>(
    original_input: &'a [u8],
) -> impl FnOnce(ErrorTree<&'a [u8]>) -> ErrorTree<usize> {
    move |t| {
        let t2: ErrorTree<ByteOffset> = t.extract_context(original_input);
        let t3: ErrorTree<usize> = t2.map_locations(|o| o.0);
        t3
    }
}

pub(super) fn load_partial<'a, F, T>(
    fun: F,
    input: &'a [u8],
) -> Result<(&'a [u8], T), ErrorTree<usize>>
where
    F: FnOnce(&'a [u8]) -> IResult<&'a [u8], T, ErrorTree<&'a [u8]>>,
{
    fun(input).finish().map_err(to_err_tree(input))
}

pub(super) fn load<'a, F, T>(fun: F, input: &'a [u8]) -> Result<T, ErrorTree<usize>>
where
    F: FnOnce(&'a [u8]) -> IResult<&'a [u8], T, ErrorTree<&'a [u8]>>,
{
    let (rest, result) = fun(input).finish().map_err(to_err_tree(input))?;
    if !rest.is_empty() {
        log::warn!("Unparsed rest: {:#?}", Buf(rest));
    }
    Ok(result)
}

pub(super) fn load_chunk<'a, T: Chunk<'a>>(input: &'a [u8]) -> Result<T, ErrorTree<usize>> {
    let (rest, result) = T::parse::<ErrorTree<&'a [u8]>>(input)
        .finish()
        .map_err(to_err_tree(input))?;
    if !rest.is_empty() {
        log::warn!("Unparsed rest after {}: {:#?}", T::TAG, Buf(rest));
    }
    Ok(result)
}
