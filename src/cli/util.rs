use nom_supreme::{
    error::ErrorTree,
    final_parser::{ByteOffset, ExtractContext},
};
use signum::nom::{Finish, IResult};

pub(super) fn to_err_tree<'a>(
    original_input: &'a [u8],
) -> impl FnOnce(ErrorTree<&'a [u8]>) -> ErrorTree<usize> {
    move |t| {
        let t2: ErrorTree<ByteOffset> = t.extract_context(original_input);
        let t3: ErrorTree<usize> = t2.map_locations(|o| o.0);
        t3
    }
}

pub(super) fn load<'a, F, T>(fun: F, input: &'a [u8]) -> Result<T, ErrorTree<usize>>
where
    F: FnOnce(&'a [u8]) -> IResult<&'a [u8], T, ErrorTree<&'a [u8]>>,
{
    let (_, result) = fun(input).finish().map_err(to_err_tree(input))?;
    Ok(result)
}
