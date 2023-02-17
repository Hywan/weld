use std::{marker::PhantomData, num::NonZeroU64, result::Result as StdResult};

use bstr::BStr;
use nom::Offset;

use super::{Address, SectionIndex};
use crate::{combinators::*, BigEndian, Endianness, Input, LittleEndian, NumberParser, Result};

/// A symbol.
#[derive(Debug, PartialEq, Eq)]
pub struct Symbol<'a> {
    // Name of the symbol, if any.
    pub name: Option<&'a BStr>,
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

impl<'a> Symbol<'a> {
    pub fn parse<N, E>(input: Input<'a>) -> Result<'a, Self, E>
    where
        N: NumberParser<'a, E>,
        E: ParseError<Input<'a>>,
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
            Address::parse_u32::<N, _>,
            SymbolBinding::parse::<N, _>,
            SymbolType::parse::<N, _>,
            tag(&[0x00]),
            SectionIndex::parse_u16::<N, _>,
            Address::parse::<N, _>,
            N::u64,
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

impl SymbolBinding {
    pub fn parse<'a, N, E>(input: Input<'a>) -> Result<Self, E>
    where
        N: NumberParser<'a, E>,
        E: ParseError<Input<'a>>,
    {
        let (_, binding) = N::u8(input)?;

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

impl SymbolType {
    pub fn parse<'a, N, E>(input: Input<'a>) -> Result<Self, E>
    where
        N: NumberParser<'a, E>,
        E: ParseError<Input<'a>>,
    {
        let (input, r#type) = N::u8(input)?;

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
    ) -> Self {
        Self { input, endianness, entity_size, _phantom: PhantomData }
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

        let parsed = match self.endianness {
            Endianness::Big => Symbol::parse::<BigEndian, E>(self.input),
            Endianness::Little => Symbol::parse::<LittleEndian, E>(self.input),
        };

        match parsed {
            Ok((next_input, symbol)) => {
                // Ensure we have parsed the correct amount of bytes.
                if let Some(entity_size) = self.entity_size {
                    let offset = self.input.offset(next_input);
                    let entity_size: usize = u64::from(entity_size)
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

        assert_eq!(
            Symbol::parse::<BigEndian, ()>(input),
            Ok((
                &[] as &[u8],
                Symbol {
                    name: None,
                    name_offset: Address(1),
                    binding: SymbolBinding::Global,
                    r#type: SymbolType::Function,
                    section_index_where_symbol_is_defined: SectionIndex::Ok(2),
                    value: Address(7),
                    size: 1,
                }
            ))
        );
    }

    #[test]
    fn test_symbol_binding() {
        macro_rules! test {
            ($input:expr => $result:expr) => {{
                let input: u8 = $input << 4;
                assert_eq!(
                    SymbolBinding::parse::<crate::BigEndian, ()>(&[input]),
                    Ok((&[input] as &[u8], $result))
                    //    ^~~~~ doesn't consume the input!
                );
            }};

            ( $( $input:expr => $result:expr ),* $(,)? ) => {
                $( test!($input => $result); )*
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
            ($input:expr => $result:expr) => {{
                let input: u8 = $input & 0x0f;
                assert_eq!(
                    SymbolType::parse::<crate::BigEndian, ()>(&[input]),
                    Ok((&[] as &[u8], $result))
                );
            }};

            ( $( $input:expr => $result:expr ),* $(,)? ) => {
                $( test!($input => $result); )*
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

        let mut iterator = SymbolIterator::<()>::new(input, Endianness::Big, None);

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
