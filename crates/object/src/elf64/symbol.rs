use std::{borrow::Cow, marker::PhantomData, num::NonZeroU64, result::Result as StdResult};

use bstr::BStr;
use nom::Offset;

use super::{Address, Section, SectionIndex};
use crate::{
    combinators::*, BigEndian, Endianness, Input, LittleEndian, Number, Read, Result, Write,
};

/// A symbol.
#[derive(Debug, PartialEq, Eq)]
pub struct Symbol<'a> {
    // Name of the symbol, if any.
    pub name: Option<Cow<'a, BStr>>,
    /// An offset, in bytes, to the symbol name, relative to the start
    /// of the symbol string table. If this field contains zero, the symbol has
    /// no name.
    pub name_offset: Address,
    /// The symbol type.
    pub r#type: SymbolType,
    /// The symbol binding attribute, i.e. its scope.
    pub binding: SymbolBinding,
    /// The section index of the section in which the symbol is “defined”.
    pub section_index_where_symbol_is_defined: SectionIndex,
    /// The value of the symbol. This may be an absolute value or a relocatable
    /// address.
    ///
    /// In relocatable files, this field contains the alignment constraint for
    /// common symbols, and a section-relative offset for defined relocatable
    /// symbols.
    ///
    /// In executable of shared object files, this field contains a virtual
    /// address for defined relocatable symbols.
    pub value: Address,
    /// The size of the value associated with the symbol. If a symbol does not
    /// have an associated size, or the size is unknown, this field contains
    /// zero.
    pub size: u64,
}

impl<'a> Read for Symbol<'a> {
    fn read<'r, N, E>(input: Input<'r>) -> Result<'r, Self, E>
    where
        N: Number,
        E: ParseError<Input<'r>>,
    {
        let (
            input,
            (
                name_offset,
                binding,
                r#type,
                _other,
                section_index_where_symbol_is_defined,
                value,
                size,
            ),
        ) = tuple((
            <Address as Read<u32>>::read::<N, _>,
            SymbolBinding::read::<N, _>,
            SymbolType::read::<N, _>,
            tag(&[0x00]),
            <SectionIndex as Read<u16>>::read::<N, _>,
            <Address as Read<u64>>::read::<N, _>,
            N::read_u64,
        ))(input)?;

        Ok((
            input,
            Self {
                name: None,
                name_offset,
                r#type,
                binding,
                section_index_where_symbol_is_defined,
                value,
                size,
            },
        ))
    }
}

impl<'a> Write for Symbol<'a> {
    fn write<N, B>(&self, buffer: &mut B) -> std::io::Result<()>
    where
        N: Number,
        B: std::io::Write,
    {
        <Address as Write<u32>>::write::<N, _>(&self.name_offset, buffer)?;

        let binding: u8 = match self.binding {
            SymbolBinding::Local => 0x00,
            SymbolBinding::Global => 0x01,
            SymbolBinding::Weak => 0x02,
            SymbolBinding::LowEnvironmentSpecific => 0x0a,
            SymbolBinding::HighEnvironmentSpecific => 0x0c,
            SymbolBinding::LowProcessorSpecific => 0x0d,
            SymbolBinding::HighProcessorSpecific => 0x0f,
        };

        let r#type: u8 = match self.r#type {
            SymbolType::NoType => 0x00,
            SymbolType::Object => 0x01,
            SymbolType::Function => 0x02,
            SymbolType::Section => 0x03,
            SymbolType::File => 0x04,
            SymbolType::LowEnvironmentSpecific => 0x0a,
            SymbolType::HighEnvironmentSpecific => 0x0c,
            SymbolType::LowProcessorSpecific => 0x0d,
            SymbolType::HighProcessorSpecific => 0x0f,
        };

        let binding_and_type = (binding << 4) | (r#type & 0x0f);

        buffer.write_all(&N::write_u8(binding_and_type))?;
        buffer.write_all(&N::write_u8(0))?;
        <SectionIndex as Write<u16>>::write::<N, _>(
            &self.section_index_where_symbol_is_defined,
            buffer,
        )?;
        <Address as Write<u64>>::write::<N, _>(&self.value, buffer)?;
        buffer.write_all(&N::write_u64(self.size))
    }
}

/// A symbol binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolBinding {
    /// The symbol is not visible outside the object file.
    Local = 0x00,
    /// Global symbol, visible to all object files.
    Global = 0x01,
    /// Global scope, but with lower precedence than global symbols.
    Weak = 0x02,
    /// Low environment-specific use.
    LowEnvironmentSpecific = 0x0a,
    /// High environment-specific use.
    HighEnvironmentSpecific = 0x0c,
    /// Low processor-specific use.
    LowProcessorSpecific = 0x0d,
    /// High processor-specific use.
    HighProcessorSpecific = 0x0f,
}

impl Read for SymbolBinding {
    fn read<'a, N, E>(input: Input<'a>) -> Result<'a, Self, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (_, binding) = N::read_u8(input)?;

        Ok((
            input,
            match binding >> 4 {
                0x00 => Self::Local,
                0x01 => Self::Global,
                0x02 => Self::Weak,
                0x0a => Self::LowEnvironmentSpecific,
                0x0c => Self::HighProcessorSpecific,
                0x0d => Self::LowEnvironmentSpecific,
                0x0f => Self::HighProcessorSpecific,
                _ => return Err(Err::Error(E::from_error_kind(input, ErrorKind::Alt))),
            },
        ))
    }
}

/// A symbol type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    /// No type specified (e.g., an absolute symbol).
    NoType = 0x00,
    /// Data object.
    Object = 0x01,
    /// Function entry point.
    Function = 0x02,
    /// The symbol is associated with a section.
    Section = 0x03,
    /// Source file associated with the object file.
    File = 0x04,
    /// Low environment-specific use.
    LowEnvironmentSpecific = 0x0a,
    /// High environment-specific use.
    HighEnvironmentSpecific = 0x0c,
    /// Low processor-specific use.
    LowProcessorSpecific = 0x0d,
    /// High processor-specific use.
    HighProcessorSpecific = 0x0f,
}

impl Read for SymbolType {
    fn read<'a, N, E>(input: Input<'a>) -> Result<'a, Self, E>
    where
        N: Number,
        E: ParseError<Input<'a>>,
    {
        let (input, r#type) = N::read_u8(input)?;

        Ok((
            input,
            match r#type & 0x0f {
                0x00 => Self::NoType,
                0x01 => Self::Object,
                0x02 => Self::Function,
                0x03 => Self::Section,
                0x04 => Self::File,
                0x0a => Self::LowEnvironmentSpecific,
                0x0c => Self::HighProcessorSpecific,
                0x0d => Self::LowEnvironmentSpecific,
                0x0f => Self::HighProcessorSpecific,
                _ => return Err(Err::Error(E::from_error_kind(input, ErrorKind::Alt))),
            },
        ))
    }
}

/// An iterator producing [`Symbol`]s.
pub struct SymbolIterator<'a, E>
where
    E: ParseError<Input<'a>>,
{
    input: Input<'a>,
    endianness: Endianness,
    entity_size: Option<NonZeroU64>,
    strings_section: Option<&'a Section<'a>>,
    _phantom: PhantomData<E>,
}

impl<'a, E> SymbolIterator<'a, E>
where
    E: ParseError<Input<'a>>,
{
    pub(super) fn new(
        input: Input<'a>,
        endianness: Endianness,
        entity_size: Option<NonZeroU64>,
        strings_section: Option<&'a Section<'a>>,
    ) -> Self {
        Self { input, endianness, entity_size, strings_section, _phantom: PhantomData }
    }
}

impl<'a, E> Iterator for SymbolIterator<'a, E>
where
    E: ParseError<Input<'a>>,
{
    type Item = StdResult<Symbol<'a>, Err<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            return None;
        }

        let read = match self.endianness {
            Endianness::Big => Symbol::read::<BigEndian, E>(self.input),
            Endianness::Little => Symbol::read::<LittleEndian, E>(self.input),
        };

        match read {
            Ok((next_input, mut symbol)) => {
                // Ensure we have read the correct amount of bytes.
                if let Some(entity_size) = self.entity_size {
                    let offset = self.input.offset(next_input);
                    let entity_size: usize = entity_size
                        .get()
                        .try_into()
                        .expect("Failed to cast the entity size from `u64` to `usize`");

                    if offset != entity_size {
                        return Some(Err(Err::Error(E::from_error_kind(
                            self.input,
                            ErrorKind::LengthValue,
                        ))));
                    }
                }

                self.input = next_input;

                if let Some(strings_section) = &self.strings_section {
                    symbol.name = strings_section.data.string_at_offset(symbol.name_offset.into());
                }

                Some(Ok(symbol))
            }

            Err(err) => Some(Err(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol() {
        #[rustfmt::skip]
        let input: &[u8] = &[
            // Name offset.
            0x00, 0x00, 0x00, 0x01,
            // Binding + type.
            0x12,
            // (other).
            0x00,
            // Section index.
            0x00, 0x02,
            // Value.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,
            // Size.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];

        let symbol = Symbol {
            name: None,
            name_offset: Address(1),
            binding: SymbolBinding::Global,
            r#type: SymbolType::Function,
            section_index_where_symbol_is_defined: SectionIndex::Ok(2),
            value: Address(7),
            size: 1,
        };

        assert_read_write!(
            Symbol: Read<()> + Write<()> {
                bytes_value(big_endian) = input,
                rust_value = symbol,
            }
        );
    }

    #[test]
    fn test_symbol_binding() {
        macro_rules! test {
            ( $( $input:expr => $result:expr ),* $(,)* ) => {
                $(
                    let input: u8 = $input << 4;
                    assert_eq!(
                        SymbolBinding::read::<crate::BigEndian, ()>(&[input]),
                        Ok((&[input] as &[u8], $result))
                        //    ^~~~~ doesn't consume the input!
                    );
                )*
            };
        }

        test!(
            0x00 => SymbolBinding::Local,
            0x01 => SymbolBinding::Global,
            0x02 => SymbolBinding::Weak,
            0x0a => SymbolBinding::LowEnvironmentSpecific,
            0x0c => SymbolBinding::HighProcessorSpecific,
            0x0d => SymbolBinding::LowEnvironmentSpecific,
            0x0f => SymbolBinding::HighProcessorSpecific,
        );
    }

    #[test]
    fn test_symbol_type() {
        macro_rules! test {
            ( $( $input:expr => $result:expr ),* $(,)* ) => {
                $(
                    let input: u8 = $input & 0x0f;
                    assert_eq!(
                        SymbolType::read::<crate::BigEndian, ()>(&[input]),
                        Ok((&[] as &[u8], $result))
                    );
                )*
            };
        }

        test!(
            0x00 => SymbolType::NoType,
            0x01 => SymbolType::Object,
            0x02 => SymbolType::Function,
            0x03 => SymbolType::Section,
            0x04 => SymbolType::File,
            0x0a => SymbolType::LowEnvironmentSpecific,
            0x0c => SymbolType::HighProcessorSpecific,
            0x0d => SymbolType::LowEnvironmentSpecific,
            0x0f => SymbolType::HighProcessorSpecific,
        );
    }

    #[test]
    fn test_symbol_iterator() {
        #[rustfmt::skip]
        let input: &[u8] = &[
            // Symbol 1.

            // Name offset.
            0x00, 0x00, 0x00, 0x01,
            // Binding + type.
            0x12,
            // (other).
            0x00,
            // Section index.
            0x00, 0x02,
            // Value.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,
            // Size.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,

            // Symbol 2.

            // Name offset.
            0x00, 0x00, 0x00, 0x03,
            // Binding + type.
            0x23,
            // (other).
            0x00,
            // Section index.
            0x00, 0x01,
            // Value.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05,
            // Size.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        ];

        let mut iterator = SymbolIterator::<()>::new(input, Endianness::Big, None, None);

        {
            let symbol = iterator.next();

            assert_eq!(
                symbol,
                Some(Ok(Symbol {
                    name: None,
                    name_offset: Address(1),
                    binding: SymbolBinding::Global,
                    r#type: SymbolType::Function,
                    section_index_where_symbol_is_defined: SectionIndex::Ok(2),
                    value: Address(7),
                    size: 1,
                }))
            )
        }

        {
            let symbol = iterator.next();

            assert_eq!(
                symbol,
                Some(Ok(Symbol {
                    name: None,
                    name_offset: Address(3),
                    binding: SymbolBinding::Weak,
                    r#type: SymbolType::Section,
                    section_index_where_symbol_is_defined: SectionIndex::Ok(1),
                    value: Address(5),
                    size: 2,
                }))
            )
        }

        {
            assert_eq!(iterator.next(), None);
        }
    }
}
