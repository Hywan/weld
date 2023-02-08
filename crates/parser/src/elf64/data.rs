use std::fmt;

use bstr::BStr;

use super::{SectionType, Symbol, SymbolIterator};
use crate::{combinators::*, Endianness, Input};

/// `Data` is a wrapper around `&[u8]`.
///
/// It represents the data owned by a [`Program`][super::Program] or a
/// [`Section`][super::Section].
pub struct Data<'a> {
    pub(crate) inner: &'a [u8],
    pub(crate) r#type: DataType,
    endianness: Endianness,
}

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

impl<'a> Data<'a> {
    /// Create a new `Data` type, wrapping some bytes.
    pub(crate) fn new(inner: &'a [u8], r#type: DataType, endianness: Endianness) -> Self {
        Self { inner, r#type, endianness }
    }

    /// Get the string at a specific offset, if and only if (i) the data type
    /// is [`DataType::StringTable`] and (ii) the string is null-terminated.
    ///
    /// The string is not guaranteed to be valid UTF-8. It is a bytes slice,
    /// `&[u8]`.
    pub fn get_string_at_offset(&self, offset: usize) -> Option<&'a BStr> {
        if self.r#type != DataType::StringTable {
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
    pub fn iter_symbols<E>(&self) -> Option<impl Iterator<Item = Result<Symbol<'a>, Err<E>>>>
    where
        E: ParseError<Input<'a>>,
    {
        if self.r#type != DataType::SymbolTable {
            return None;
        }

        Some(SymbolIterator::new(self.inner, self.endianness))
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

            DataType::SymbolTable => formatter
                .write_fmt(format_args!("{:?} Data(..), interpreted: {:#?}", self.r#type, "")),
        }
    }
}
