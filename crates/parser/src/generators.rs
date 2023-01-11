use crate::Input;
pub use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    error::{ErrorKind, ParseError},
    sequence::tuple,
    Err,
};
use nom::{IResult, InputIter, ToUsize};

/// Like `take` but it “skips” the parsed value.
pub fn skip<'a, C, E>(count: C) -> impl Fn(Input<'a>) -> IResult<Input<'a>, Input<'a>, E>
where
    C: ToUsize,
    E: ParseError<Input<'a>>,
{
    let count = count.to_usize();

    move |input: Input| match input.slice_index(count) {
        Err(_needed) => Err(Err::Error(E::from_error_kind(input, ErrorKind::Eof))),
        Ok(index) => Ok((&input[index..], &[])),
    }
}
