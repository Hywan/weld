use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, Attribute, Data, DataEnum, DeriveInput, Generics, Ident};

#[proc_macro_derive(Read)]
pub fn derive_enum_read(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = parse(input).unwrap();

    match derive_input.data {
        Data::Enum(ref enum_data) => derive_enum_read_impl(
            &derive_input.ident,
            enum_data,
            &derive_input.generics,
            fetch_repr(&derive_input.attrs),
        ),
        Data::Struct(_) | Data::Union(_) => {
            panic!("`Read` cannot be derived onto `struct` or `union`")
        }
    }
}

fn derive_enum_read_impl(
    enum_name: &Ident,
    data: &DataEnum,
    generics: &Generics,
    repr: Option<Ident>,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let repr = repr.expect("A `#repr(…)` attribute must be present");
    let parser_combinator = proc_macro2::Ident::new(
        match repr.to_string().as_str() {
            "u8" => "read_u8",
            "u16" => "read_u16",
            "u32" => "read_u32",
            repr => panic!("`Read` does not handle the `{repr}` representation yet"),
        },
        proc_macro2::Span::call_site(),
    );

    let (parser_logic, variants): (Vec<_>, Vec<_>) = data
        .variants
        .iter()
        .map(|variant| {
            let name = &variant.ident;
            let discriminant = match &variant.discriminant {
                Some((_, syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(int), .. }))) => int,
                _ => panic!(
                    "All variants must have a discriminant, and it must represent an integer"
                ),
            };

            (
                quote! {
                    #discriminant => Self::#name
                },
                quote! {
                    #name
                },
            )
        })
        .unzip();

    let test_name = proc_macro2::Ident::new(
        &format!("test_{}", enum_name.to_string().to_lowercase()),
        proc_macro2::Span::call_site(),
    );

    quote! {
        impl #impl_generics #enum_name #ty_generics
        #where_clause
        {
            pub fn read<'a, N, E>(input: crate::Input<'a>) -> crate::Result<Self, E>
            where
                N: crate::Number,
                E: ::nom::error::ParseError<crate::Input<'a>>,
            {
                let (input, discriminant) = N::#parser_combinator::<E>(input)?;

                Ok((
                    input,
                    match discriminant {
                        #( #parser_logic, )*
                        _ => return Err(::nom::Err::Error(E::from_error_kind(input, ::nom::error::ErrorKind::Alt))),
                    }
                ))
            }
        }

        #[test]
        fn #test_name() {
            #(
                {
                    let input: #repr = #enum_name::#variants as _;

                    assert_eq!(
                        #enum_name::read::<crate::LittleEndian, ()>(&input.to_le_bytes()[..]),
                        Ok((&[] as &[u8], #enum_name::#variants))
                    );
                    assert_eq!(
                        #enum_name::read::<crate::BigEndian, ()>(&input.to_be_bytes()[..]),
                        Ok((&[] as &[u8], #enum_name::#variants))
                    );
                }
            )*
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
                    syn::Meta::List(ref meta_list) if meta_list.path.is_ident("repr") => {
                        meta_list.nested.first().map(|nested_meta| match nested_meta {
                            syn::NestedMeta::Meta(syn::Meta::Path(repr_value)) => {
                                repr_value.get_ident().cloned()
                            }
                            _ => panic!("`repr` seems to have an invalid value"),
                        })
                    }
                    _ => None,
                })
                .ok()
                .flatten()
        })
        .flatten()
}
