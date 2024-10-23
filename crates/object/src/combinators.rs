pub use nom::{
    bytes::complete::tag,
    error::{ErrorKind, ParseError},
    sequence::tuple,
    Err,
};
use nom::{InputIter, ToUsize};

use crate::{Input, Result};

/// Like `take` but it “skips” the parsed value.
pub fn skip<'a, C, E>(count: C) -> impl Fn(Input<'a>) -> Result<'a, Input<'a>, E>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip() {
        let input: &[u8] = &[1, 2, 3, 4, 5];

        assert_eq!(skip::<_, ()>(2usize)(input), Ok((&[3, 4, 5][..], &[] as &[u8])));
    }
}
