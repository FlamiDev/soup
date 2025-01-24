use proc_macro::TokenStream;
use quote::quote;
use syn::__private::TokenStream2;
use syn::parse_macro_input;
use syn::spanned::Spanned;

#[proc_macro_derive(Parser, attributes(text))]
pub fn parser_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    match input.data {
        syn::Data::Struct(ref data) => {
            let body = parse_struct(&data.fields, quote! { Self });
            let name = &input.ident;
            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
            let output = quote! {
                impl #impl_generics parser_lib::Parser<#name> for #name #ty_generics #where_clause {
                    fn parse(mut words: parser_lib::VecWindow<parser_lib::Word>) -> parser_lib::ParseResult<Self> #body
                }
            };
            output.into()
        }
        syn::Data::Enum(ref data) => {
            let mut variant_parser_names = Vec::new();
            let mut variant_parsers = Vec::new();
            for variant in &data.variants {
                let ident = &variant.ident;
                let body = parse_struct(&variant.fields, quote! { Self::#ident });
                let function_name = syn::Ident::new(&format!("parse_{}", ident.to_string().to_lowercase()), ident.span());
                variant_parser_names.push(function_name.clone());
                variant_parsers.push(quote! {
                    fn #function_name(mut words: parser_lib::VecWindow<parser_lib::Word>) -> parser_lib::ParseResult<Self> #body
                });
            }
            let name = &input.ident;
            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
            let output = quote! {
                impl #impl_generics parser_lib::Parser<#name> for #name #ty_generics #where_clause {
                    fn parse(words: parser_lib::VecWindow<parser_lib::Word>) -> parser_lib::ParseResult<Self> {
                        let mut errors = Vec::new();
                        let depth = 0;
                        #(
                            let parser_lib::ParseResult(res, new_words, new_errors, depth) = Self::#variant_parser_names(words.clone());
                            if let Some(res) = res {
                                return parser_lib::ParseResult(Some(res), new_words, new_errors, depth);
                            }
                            errors.extend(new_errors);
                            if depth > 0 {
                                return parser_lib::ParseResult(None, new_words, errors, depth);
                            }
                        )*
                        parser_lib::ParseResult(None, words, errors, depth)
                    }
                }
                impl #impl_generics #name #ty_generics #where_clause {
                    #(#variant_parsers)*
                }
            };
            output.into()
        }
        _ => panic!("Only structs and enums are supported"),
    }
}

fn parse_struct(fields: &syn::Fields, resulting_type: TokenStream2) -> TokenStream2 {
    match fields {
        syn::Fields::Named(fields) => {
            let mut parse_fields = Vec::new();
            let mut set_fields = Vec::new();
            let mut depth = 0;
            for (i, field) in fields.named.iter().enumerate() {
                let (new_depth, parse) = parse_field(field, depth);
                depth = new_depth;
                let res_name = syn::Ident::new(&format!("res{}", i), field.span());
                parse_fields.push(quote! {
                    let parser_lib::ParseResult(res, mut words, new_errors, depth) = #parse;
                    errors.extend(new_errors);
                    let Some(#res_name) = res else {
                        return parser_lib::ParseResult(None, words, errors, depth + #depth);
                    };
                });
                let ident = field.ident.clone().unwrap();
                set_fields.push(
                    quote! {
                        #ident: #res_name,
                    }
                );
                depth += 1;
            }
            quote! {
                {
                    let mut errors = Vec::new();
                    #(#parse_fields)*
                    parser_lib::ParseResult(Some(#resulting_type {
                        #(#set_fields)*
                    }), words, errors, #depth)
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let mut parse_fields = Vec::new();
            let mut set_fields = Vec::new();
            let mut depth = 0;
            for (i, field) in fields.unnamed.iter().enumerate() {
                let (new_depth, parse) = parse_field(field, depth);
                depth = new_depth;
                let res_name = syn::Ident::new(&format!("res{}", i), field.span());
                parse_fields.push(quote! {
                    let parser_lib::ParseResult(res, mut words, new_errors, depth) = #parse;
                    errors.extend(new_errors);
                    let Some(#res_name) = res else {
                        return parser_lib::ParseResult(None, words, errors, depth + #depth);
                    };
                });
                set_fields.push(quote! { #res_name, });
                depth += 1;
            }
            quote! {
                {
                    let mut errors = Vec::new();
                    #(#parse_fields)*
                    parser_lib::ParseResult(Some(#resulting_type (
                        #(#set_fields)*
                    )), words, errors, #depth)
                }
            }
        }
        syn::Fields::Unit => quote! {
            parser_lib::ParseResult(Some(#resulting_type), words, Vec::new(), 0)
        },
    }
}

fn parse_field(field: &syn::Field, depth: usize) -> (usize, TokenStream2) {
    let parse_attrs = field
        .attrs
        .iter()
        .enumerate()
        .filter_map(|(i, attr)| {
            let syn::Meta::NameValue(meta) = attr.parse_meta().ok()? else {
                return None;
            };
            if !meta.path.is_ident("text") {
                return None;
            }
            let syn::Lit::Str(lit) = meta.lit else {
                return None;
            };
            let val = lit.value();
            let attr_depth = depth + i;
            Some(quote! {
                if words.first().map(|w| &w.text) == Some(&#val.to_string()) {
                    words.pop_first();
                } else {
                    let first = words.first().cloned();
                    return parser_lib::ParseResult(None, words, vec![parser_lib::ParseError {
                        expected: #val.to_string(),
                        got: first,
                    }], #attr_depth);
                }
            })
        })
        .collect::<Vec<_>>();
    let ty = &field.ty;
    (
        depth + parse_attrs.len(),
        quote! {
            {
                #(#parse_attrs)*
                parser_lib::parse_to_type::<#ty>(words)
            }
        },
    )
}
