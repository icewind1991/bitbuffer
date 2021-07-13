use crate::discriminant::Discriminant;
use crate::size;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, parse_str, Attribute, Data, DataStruct, DeriveInput, Expr,
    Fields, GenericParam, Ident, Index, Lit, Member, Path, Type,
};
use syn_util::get_attribute_value;

pub fn derive_bitwrite_trait(
    input: proc_macro::TokenStream,
    trait_name: String,
    write_method_name: String,
    extra_param: Option<TokenStream>,
) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let endianness = get_attribute_value(&input.attrs, &["endianness"]);
    let mut trait_generics = input.generics.clone();
    // we need these separate generics to only add out Endianness param to the 'impl'
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    let lifetime: Option<&GenericParam> = trait_generics
        .params
        .iter()
        .find(|param| matches!(param, GenericParam::Lifetime(_)));
    let _lifetime = match lifetime {
        Some(GenericParam::Lifetime(lifetime)) => lifetime.lifetime.clone(),
        _ => {
            // trait_generics.params.push(parse_quote!('a));
            parse_quote!('a)
        }
    };
    if endianness.is_none() {
        trait_generics
            .params
            .push(parse_quote!(_E: ::bitbuffer::Endianness));
    }
    let (impl_generics, _, _) = trait_generics.split_for_impl();
    let span = input.span();

    let _size = size(
        input.data.clone(),
        &name,
        &input.attrs,
        extra_param.is_some(),
    );
    let parsed = write(input.data.clone(), &name, &input.attrs);
    let _parsed_unchecked = write(input.data.clone(), &name, &input.attrs);

    let endianness_placeholder = endianness.unwrap_or_else(|| "_E".to_owned());
    let trait_def_str = format!("::bitbuffer::{}<{}>", trait_name, &endianness_placeholder);
    let trait_def = parse_str::<Path>(&trait_def_str).expect("trait");

    let endianness_ident = Ident::new(&endianness_placeholder, span);

    let _size_extra_param = if extra_param.is_some() {
        Some(quote!(input_size: usize))
    } else {
        None
    };

    let _extra_param_call = if extra_param.is_some() {
        Some(quote!(input_size,))
    } else {
        None
    };

    let write_method = Ident::new(&write_method_name, span);

    let expanded = quote! {
        impl #impl_generics #trait_def for #name #ty_generics #where_clause {
            fn #write_method(&self, __target__stream: &mut ::bitbuffer::BitWriteStream<#endianness_ident>#extra_param) -> ::bitbuffer::Result<()> {
                #parsed
            }
        }
    };

    // panic!("{}", TokenStream::to_string(&expanded));

    proc_macro::TokenStream::from(expanded)
}

fn write(data: Data, struct_name: &Ident, attrs: &[Attribute]) -> TokenStream {
    let span = struct_name.span();

    match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let expand = fields.iter().enumerate().map(|(i, field)| {
                let name = field
                    .ident
                    .clone()
                    .unwrap_or_else(|| Ident::new(&format!("__{}", i), span));
                let member = field.ident.clone().map(Member::Named).unwrap_or_else(|| {
                    Member::Unnamed(Index {
                        index: i as u32,
                        span: field.span(),
                    })
                });
                // extract int fields to be used in size expressions
                if type_is_int(&field.ty) {
                    quote_spanned! { field.span() =>
                        #[allow(unused_variables)]
                        let #name = self.#member;
                    }
                } else {
                    quote! {}
                }
            });

            let writes = fields.iter().enumerate().map(|(i, f)| {
                // Get attributes `#[..]` on each field
                let size = get_field_size(&f.attrs, f.span());
                let span = f.span();
                let member = f.ident.clone().map(Member::Named).unwrap_or_else(|| {
                    Member::Unnamed(Index {
                        index: i as u32,
                        span,
                    })
                });
                match size {
                    Some(size) => {
                        quote_spanned! { span =>
                            {
                                let _size: usize = #size;
                                __target__stream.write_sized(&self.#member, _size)?;
                            }
                        }
                    }
                    None => {
                        quote_spanned! { span => {
                            __target__stream.write(&self.#member)?;
                        }}
                    }
                }
            });

            quote_spanned! {span=>
                #(#expand)*
                #(#writes)*
                Ok(())
            }
        }
        Data::Enum(data) => {
            let discriminant_bits: u64 = match get_attribute_value(attrs, &["discriminant_bits"]) {
                Some(attr) => attr,
                None => {
                    return quote! {span=>
                        compile_error!("'discriminant_bits' attribute is required when deriving `BinWrite` for enums");
                    }
                }
            };

            let mut last_discriminant = -1;

            let max_discriminant = data
                .variants
                .iter()
                .map(|variant| match Discriminant::from(variant) {
                    Discriminant::Int(discriminant) => {
                        last_discriminant = discriminant as isize;
                        discriminant
                    }
                    Discriminant::Wildcard => 0,
                    Discriminant::Default => {
                        let new_discriminant = (last_discriminant + 1) as usize;
                        last_discriminant += 1;
                        new_discriminant
                    }
                })
                .max()
                .unwrap_or(0);

            let mut last_discriminant = -1;

            let discriminant_value = data.variants.iter().map(|variant| {
                let span = variant.span();
                let variant_name = &variant.ident;

                let discriminant_token: TokenStream = match Discriminant::from(variant) {
                    Discriminant::Int(discriminant) => {
                        last_discriminant = discriminant as isize;
                        quote_spanned! { span => #discriminant }
                    }
                    Discriminant::Wildcard => {
                        let free_discriminant = max_discriminant + 1;
                        quote_spanned! { span => #free_discriminant }
                    }
                    Discriminant::Default => {
                        let new_discriminant = (last_discriminant + 1) as usize;
                        last_discriminant += 1;
                        quote_spanned! { span => #new_discriminant }
                    }
                };

                match &variant.fields {
                    Fields::Unit => quote_spanned! {span =>
                        #struct_name::#variant_name => #discriminant_token
                    },
                    Fields::Unnamed(_f) => {
                        quote_spanned! { span =>
                            #struct_name::#variant_name(_) => #discriminant_token
                        }
                    }
                    _ => unimplemented!(),
                }
            });

            let write_inner = data.variants.iter().map(|variant| {
                let span = variant.span();
                let variant_name = &variant.ident;

                match &variant.fields {
                    Fields::Unit => quote_spanned! {span =>
                        #struct_name::#variant_name => {},
                    },
                    Fields::Unnamed(f) => {
                        let size = get_field_size(&variant.attrs, f.span());
                        match size {
                            Some(size) => {
                                quote_spanned! { span =>
                                    #struct_name::#variant_name(inner) => {
                                        let size:usize = #size;
                                        __target__stream.write_sized(inner, size)?;
                                    }
                                }
                            }
                            None => {
                                quote_spanned! { span =>
                                    #struct_name::#variant_name(inner) => { __target__stream.write(inner)?; }
                                }
                            }
                        }
                    }
                    _ => unimplemented!(),
                }
            });

            let span = data.enum_token.span();

            quote_spanned! {span=>
                let discriminant = match &self {
                    #(#discriminant_value),*
                };
                __target__stream.write_int(discriminant as usize, #discriminant_bits as usize)?;
                match &self {
                    #(#write_inner)*
                }
                Ok(())
            }
        }
        _ => unimplemented!(),
    }
}

fn get_field_size(attrs: &[Attribute], span: Span) -> Option<TokenStream> {
    get_attribute_value(attrs, &["size"])
        .map(|size_lit| match size_lit {
            Lit::Int(size) => {
                quote_spanned! {span =>
                    #size
                }
            }
            Lit::Str(size_field) => {
                let size = parse_str::<Expr>(&size_field.value()).expect("size");
                quote_spanned! {span =>
                    (#size) as usize
                }
            }
            _ => panic!("Unsupported value for size attribute"),
        })
        .or_else(|| {
            get_attribute_value::<Lit>(attrs, &["size_bits"]).map(|_| {
                quote_spanned! {span =>
                    compile_error!("#[size_bits] is not supported when deriving BitWrite or BitWriteSized")
                }
            })
        })
}

fn type_is_int(ty: &Type) -> bool {
    if let Type::Path(path) = ty {
        if let Some(ident) = path.path.get_ident() {
            let name = ident.to_string();
            matches!(
                name.as_str(),
                "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize"
            )
        } else {
            false
        }
    } else {
        false
    }
}
