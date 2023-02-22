use std::{fmt, num::NonZeroU64};

use bstr::BStr;
use nom::error::VerboseError;

use super::{SectionType, Symbol, SymbolIterator};
use crate::{combinators::*, Endianness, Input};

/// The type of `Data`.
#[derive(Debug, PartialEq, Eq)]
pub enum DataType {
    /// `Data` represents a string table.
    StringTable,
    /// `Data` represents a symbol table.
    SymbolTable,
    /// `Data` has unspecified data.
    Unspecified,
}

impl From<SectionType> for DataType {
    fn from(value: SectionType) -> Self {
        match value {
            SectionType::StringTable => Self::StringTable,
            SectionType::SymbolTable => Self::SymbolTable,
            _ => Self::Unspecified,
        }
    }
}

/// `Data` is a wrapper around `&[u8]`.
///
/// It represents the data owned by a [`Program`][super::Program] or a
/// [`Section`][super::Section].
#[derive(PartialEq)]
pub struct Data<'a> {
    /// Inner bytes.
    pub(crate) inner: &'a [u8],
    /// The type of the data represented by the bytes.
    pub(crate) r#type: DataType,
    /// The endianness of the data.
    endianness: Endianness,
    /// The size, in bytes, of each “entry”, if the data represents fixed-sized
    /// entries.
    entity_size: Option<NonZeroU64>,
}

impl<'a> Data<'a> {
    /// Create a new `Data` type, wrapping some bytes.
    pub(crate) fn new(
        inner: &'a [u8],
        r#type: DataType,
        endianness: Endianness,
        entity_size: Option<NonZeroU64>,
    ) -> Self {
        Self { inner, r#type, endianness, entity_size }
    }

    /// Get the string at a specific offset, if and only if (i) the data type
    /// is [`DataType::StringTable`], (ii) the string is null-terminated, and
    /// (iii) if the offset exists.
    ///
    /// The string is not guaranteed to be valid UTF-8. It is a bytes slice,
    /// `&[u8]`.
    pub fn string_at_offset(&self, offset: usize) -> Option<&'a BStr> {
        if self.r#type != DataType::StringTable {
            return None;
        }

        if offset >= self.inner.len() {
            return None;
        }

        let name = &self.inner[offset..];

        if let Some(name_end) = name.iter().position(|c| *c == 0x00) {
            Some(BStr::new(&name[..name_end]))
        } else {
            None
        }
    }

    /// Get an iterator over symbols, if and only if the data type is
    /// [`DataType::SymbolTable`].
    pub fn symbols<E>(&self) -> Option<impl Iterator<Item = Result<Symbol<'a>, Err<E>>>>
    where
        E: ParseError<Input<'a>>,
    {
        if self.r#type != DataType::SymbolTable {
            return None;
        }

        Some(SymbolIterator::new(self.inner, self.endianness, self.entity_size))
    }
}

impl<'a> fmt::Debug for Data<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.r#type {
            DataType::Unspecified => {
                let len = self.inner.len();

                if len > 10 {
                    formatter.write_fmt(format_args!(
                        "{:?} Data({:0<2x?} ... truncated)",
                        self.r#type,
                        &self.inner[..10],
                    ))
                } else {
                    formatter.write_fmt(format_args!(
                        "{:?} Data({:0<2x?})",
                        self.r#type,
                        &self.inner[..len],
                    ))
                }
            }

            DataType::StringTable => formatter.write_fmt(format_args!(
                "{:?} Data(..), interpreted: {:#?}",
                self.r#type,
                self.inner.split(|c| *c == 0x00).map(BStr::new).collect::<Vec<_>>()
            )),

            DataType::SymbolTable => formatter.write_fmt(format_args!(
                "{:?} Data(..), interpreted: {:#?}",
                self.r#type,
                self.symbols::<VerboseError<Input>>().unwrap().collect::<Vec<_>>()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_at_offset() {
        let data = Data::new(
            &[0x0, 0x61, 0x62, 0x63, 0x0, 0x64, 0x65, 0x0, 0x66],
            DataType::StringTable,
            Endianness::Little,
            None,
        );

        assert_eq!(data.string_at_offset(0), Some(BStr::new("")));
        assert_eq!(data.string_at_offset(1), Some(BStr::new("abc")));
        assert_eq!(data.string_at_offset(2), Some(BStr::new("bc")));
        assert_eq!(data.string_at_offset(3), Some(BStr::new("c")));
        assert_eq!(data.string_at_offset(4), Some(BStr::new("")));
        assert_eq!(data.string_at_offset(5), Some(BStr::new("de")));
        assert_eq!(data.string_at_offset(6), Some(BStr::new("e")));
        assert_eq!(data.string_at_offset(7), Some(BStr::new("")));
        assert_eq!(data.string_at_offset(8), None);
        assert_eq!(data.string_at_offset(9), None);
        assert_eq!(data.string_at_offset(10), None);
    }
}