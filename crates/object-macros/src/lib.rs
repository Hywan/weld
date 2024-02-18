use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, Attribute, Data, DataEnum, DeriveInput, Generics, Ident};

#[proc_macro_derive(ReadWrite)]
pub fn derive_enum_read_write(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = parse(input).unwrap();

    match derive_input.data {
        Data::Enum(ref enum_data) => derive_enum_read_write_impl(
            &derive_input.ident,
            enum_data,
            &derive_input.generics,
            fetch_repr(&derive_input.attrs),
        ),
        Data::Struct(_) | Data::Union(_) => {
            panic!("`ReadWrite` cannot be derived onto `struct` or `union`")
        }
    }
}

fn derive_enum_read_write_impl(
    enum_name: &Ident,
    data: &DataEnum,
    generics: &Generics,
    repr: Option<Ident>,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let repr = repr.expect("A `#repr(…)` attribute must be present");
    let (reader, writer) = match repr.to_string().as_str() {
        "u8" => ("read_u8", "write_u8"),
        "u16" => ("read_u16", "write_u16"),
        "u32" => ("read_u32", "write_u32"),
        repr => panic!("`Read` does not handle the `{repr}` representation yet"),
    };

    let reader = proc_macro2::Ident::new(reader, proc_macro2::Span::call_site());
    let writer = proc_macro2::Ident::new(writer, proc_macro2::Span::call_site());

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
        &format!("autotest_{}", enum_name.to_string().to_lowercase()),
        proc_macro2::Span::call_site(),
    );

    quote! {
        impl #impl_generics crate::Read for #enum_name #ty_generics
        #where_clause
        {
            fn read<'r, N, E>(input: crate::Input<'r>) -> crate::Result<'r, Self, E>
            where
                N: crate::Number,
                E: ::nom::error::ParseError<crate::Input<'r>>,
            {
                let (input, discriminant) = N::#reader::<E>(input)?;

                Ok((
                    input,
                    match discriminant {
                        #( #parser_logic, )*
                        _ => return Err(::nom::Err::Error(E::from_error_kind(input, ::nom::error::ErrorKind::Alt))),
                    }
                ))
            }
        }

        impl #impl_generics crate::Write for #enum_name #ty_generics
        #where_clause
        {
            fn write<N, B>(&self, buffer: &mut B) -> ::std::io::Result<()>
            where
                N: crate::Number,
                B: ::std::io::Write,
            {
                buffer.write_all(&N::#writer(*self as _))
            }
        }

        #[test]
        fn #test_name() {
            use crate::Write;

            #(
                {
                    let input = #enum_name::#variants as #repr;

                    // Read as big endian.
                    {
                        assert_eq!(
                            #enum_name::read::<crate::BigEndian, ()>(&input.to_be_bytes()),
                            Ok((&[] as &[u8], #enum_name::#variants)),
                            "read as big endian",
                        );
                    }

                    // Read as little endian.
                    {
                        assert_eq!(
                            #enum_name::read::<crate::LittleEndian, ()>(&input.to_le_bytes()),
                            Ok((&[] as &[u8], #enum_name::#variants)),
                            "read as little endian",
                        );
                    }

                    // Write as big endian.
                    {
                        let mut buffer = Vec::new();

                        #enum_name::#variants.write::<crate::BigEndian, _>(&mut buffer).unwrap();

                        assert_eq!(buffer, input.to_be_bytes(), "write as big endian");
                    }

                    // Write as little endian.
                    {
                        let mut buffer = Vec::new();

                        #enum_name::#variants.write::<crate::LittleEndian, _>(&mut buffer).unwrap();

                        assert_eq!(buffer, input.to_le_bytes(), "write as little endian");
                    }
                }
            )*
        }
    }
    .into()
}

fn fetch_repr(attrs: &[Attribute]) -> Option<Ident> {
    attrs.iter().find_map(|attr| {
        let mut ident = None;

        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                if let Some(path_ident) = meta.path.get_ident() {
                    ident = Some(path_ident.clone());

                    Ok(())
                } else {
                    Err(meta.error("Unknown `repr`"))
                }
            })
            .ok();
        }

        ident
    })
}
