use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, Attribute, Data, DataEnum, DeriveInput, Generics, Ident};

#[proc_macro_derive(EnumParse)]
pub fn derive_enum_parse(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = parse(input).unwrap();

    match derive_input.data {
        Data::Enum(ref enum_data) => derive_enum_parse_impl(
            &derive_input.ident,
            enum_data,
            &derive_input.generics,
            fetch_repr(&derive_input.attrs),
        ),
        Data::Struct(_) | Data::Union(_) => {
            panic!("`EnumParse` cannot be derived onto `struct` or `union`")
        }
    }
}

fn derive_enum_parse_impl(
    enum_name: &Ident,
    data: &DataEnum,
    generics: &Generics,
    repr: Option<Ident>,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let parser_combinator = proc_macro2::Ident::new(
        match repr
            .expect("A `#repr(…)` attribute must be present")
            .to_string()
            .as_str()
        {
            "u8" => "le_u8",
            repr => panic!("`EnumParse` does not handle the `{repr}` representation yet"),
        },
        proc_macro2::Span::call_site(),
    );

    let parser_logic: Vec<_> = data
        .variants
        .iter()
        .map(|variant| {
            let name = &variant.ident;
            let discriminant = match &variant.discriminant {
                Some((
                    _,
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(int),
                        ..
                    }),
                )) => int,
                _ => panic!(
                    "All variants must have a discriminant, and it must reprenset an integer"
                ),
            };

            quote! {
                #discriminant => Self::#name
            }
        })
        .collect();

    quote! {
        impl #impl_generics #enum_name #ty_generics
        #where_clause
        {
            pub fn parse<'a, E>(input: crate::Input<'a>) -> crate::Result<Self, E>
            where
                E: ::nom::error::ParseError<crate::Input<'a>>,
            {
                let (input, discriminant) = ::nom::number::complete::#parser_combinator(input)?;

                Ok((
                    input,
                    match discriminant {
                        #( #parser_logic, )*
                        _ => return Err(::nom::Err::Error(E::from_error_kind(input, ::nom::error::ErrorKind::Digit))),
                    }
                ))
            }
        }
    }
    .into()
}

fn fetch_repr(attrs: &[Attribute]) -> Option<Ident> {
    attrs
        .iter()
        .find_map(|attr| {
            attr.parse_meta()
                .map(|meta| match meta {
                    syn::Meta::List(ref meta_list) if meta_list.path.is_ident("repr") => meta_list
                        .nested
                        .first()
                        .map(|nested_meta| match nested_meta {
                            syn::NestedMeta::Meta(syn::Meta::Path(repr_value)) => {
                                repr_value.get_ident().cloned()
                            }
                            _ => panic!("`repr` seems to have an invalid value"),
                        }),
                    _ => None,
                })
                .ok()
                .flatten()
        })
        .flatten()
}
