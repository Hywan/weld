use std::{borrow::Cow, fmt, num::NonZeroU64, ops::Deref};

use bstr::BStr;
use nom::error::VerboseError;

use super::{Section, SectionType, Symbol, SymbolIterator};
use crate::{combinators::*, Endianness, Input};

/// The type of `Data`.
#[derive(Debug, PartialEq, Eq)]
pub enum DataType {
    /// `Data` represents a string table.
    StringTable,
    /// `Data` represents a symbol table.
    SymbolTable,
    /// `Data` represents program data.
    ProgramData,
    /// `Data` has unspecified data.
    Unspecified,
}

impl From<SectionType> for DataType {
    fn from(value: SectionType) -> Self {
        match value {
            SectionType::StringTable => Self::StringTable,
            SectionType::SymbolTable => Self::SymbolTable,
            SectionType::ProgramData => Self::ProgramData,
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
    pub(crate) inner: Cow<'a, [u8]>,
    /// The type of the data represented by the bytes.
    pub(crate) r#type: DataType,
    /// The endianness of the data.
    endianness: Endianness,
    /// The size, in bytes, of each “entry”, if the data represents fixed-sized
    /// entries.
    entity_size: Option<NonZeroU64>,
}

impl<'a> Deref for Data<'a> {
    type Target = Cow<'a, [u8]>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> Data<'a> {
    /// Create a new `Data` type, wrapping some bytes.
    pub fn new(
        inner: Cow<'a, [u8]>,
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
    pub fn string_at_offset(&self, offset: usize) -> Option<Cow<BStr>> {
        if self.r#type != DataType::StringTable {
            return None;
        }

        if offset >= self.inner.len() {
            return None;
        }

        let name = &self.inner[offset..];

        name.iter()
            .position(|c| *c == 0x00)
            .map(|name_end| Cow::Borrowed(BStr::new(&name[..name_end])))
    }

    /// Get an iterator over symbols, if and only if the data type is
    /// [`DataType::SymbolTable`].
    ///
    /// The optional `strings_section` argument is supposed to contain the
    /// `.strtab` section, see [`File::strings_section`] to get it.
    ///
    /// [`File::strings_section`]: super::File::strings_section
    pub fn symbols<E>(
        &'a self,
        strings_section: Option<&'a Section<'a>>,
    ) -> Option<impl Iterator<Item = Result<Symbol<'a>, Err<E>>>>
    where
        E: ParseError<Input<'a>>,
    {
        if self.r#type != DataType::SymbolTable {
            return None;
        }

        Some(SymbolIterator::new(
            self.inner.as_ref(),
            self.endianness,
            self.entity_size,
            strings_section,
        ))
    }
}

impl<'a> fmt::Debug for Data<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.r#type {
            DataType::StringTable => formatter.write_fmt(format_args!(
                "{:?} Data(..), interpreted: {:#?}",
                self.r#type,
                self.inner.split(|c| *c == 0x00).map(BStr::new).collect::<Vec<_>>()
            )),

            DataType::SymbolTable => formatter.write_fmt(format_args!(
                "{:?} Data(..), interpreted: {:#?}",
                self.r#type,
                self.symbols::<VerboseError<Input>>(None).unwrap().collect::<Vec<_>>()
            )),

            #[cfg(feature = "debug")]
            DataType::ProgramData => {
                #[cfg(feature = "debug-x86")]
                {
                    use iced_x86::{Decoder, DecoderOptions, FastFormatter, Instruction};

                    formatter
                        .write_fmt(format_args!("{:?} Data(..), interpreted:", self.r#type))?;

                    let mut decoder = Decoder::new(64, &self.inner, DecoderOptions::NONE);
                    let mut x86_formatter = FastFormatter::new();

                    {
                        let options = x86_formatter.options_mut();
                        options.set_space_after_operand_separator(true);
                        options.set_rip_relative_addresses(true);
                        options.set_show_symbol_address(true);
                        options.set_uppercase_hex(false);
                        options.set_use_hex_prefix(true);
                    }

                    let mut output = String::new();
                    let mut instruction = Instruction::default();

                    while decoder.can_decode() {
                        decoder.decode_out(&mut instruction);
                        output.clear();
                        x86_formatter.format(&instruction, &mut output);
                        formatter.write_fmt(format_args!("\n{:016x} ", instruction.ip()))?;

                        let start_index = instruction.ip() as usize;
                        let instr_bytes = &self.inner[start_index..start_index + instruction.len()];

                        for bytes in instr_bytes.iter() {
                            formatter.write_fmt(format_args!("{bytes:02x}"))?;
                        }

                        if instr_bytes.len() < 10 {
                            for _ in 0..10 - instr_bytes.len() {
                                formatter.write_fmt(format_args!("  "))?;
                            }
                        }

                        formatter.write_fmt(format_args!(" {output}"))?;
                    }
                }

                #[cfg(not(feature = "debug-x86"))]
                {
                    formatter.write_fmt(format_args!(
                        "{:?} Data(..), cannot interpret them",
                        self.r#type,
                    ))?;
                }

                Ok(())
            }

            #[cfg_attr(feature = "debug", allow(unreachable_patterns))]
            DataType::ProgramData | DataType::Unspecified => {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_at_offset() {
        let data = Data::new(
            Cow::Borrowed(&[0x0, 0x61, 0x62, 0x63, 0x0, 0x64, 0x65, 0x0, 0x66]),
            DataType::StringTable,
            Endianness::Little,
            None,
        );

        assert_eq!(data.string_at_offset(0), Some(Cow::Borrowed(BStr::new(""))));
        assert_eq!(data.string_at_offset(1), Some(Cow::Borrowed(BStr::new("abc"))));
        assert_eq!(data.string_at_offset(2), Some(Cow::Borrowed(BStr::new("bc"))));
        assert_eq!(data.string_at_offset(3), Some(Cow::Borrowed(BStr::new("c"))));
        assert_eq!(data.string_at_offset(4), Some(Cow::Borrowed(BStr::new(""))));
        assert_eq!(data.string_at_offset(5), Some(Cow::Borrowed(BStr::new("de"))));
        assert_eq!(data.string_at_offset(6), Some(Cow::Borrowed(BStr::new("e"))));
        assert_eq!(data.string_at_offset(7), Some(Cow::Borrowed(BStr::new(""))));
        assert_eq!(data.string_at_offset(8), None);
        assert_eq!(data.string_at_offset(9), None);
        assert_eq!(data.string_at_offset(10), None);
    }
}
