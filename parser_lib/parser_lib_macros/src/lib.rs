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
            let body = parse_struct(&data.fields, quote! { Self }, input.ident.to_string());
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
            if data.variants.is_empty() {
                return syn::Error::new(input.span(), "Empty enums are not supported")
                    .to_compile_error()
                    .into();
            }
            let mut variant_parser_names = Vec::new();
            let mut variant_parsers = Vec::new();
            for variant in &data.variants {
                let ident = &variant.ident;
                let body = parse_struct(
                    &variant.fields,
                    quote! { Self::#ident },
                    format!("{}::{}", input.ident, ident),
                );
                let function_name = syn::Ident::new(
                    &format!("parse_{}", ident.to_string().to_lowercase()),
                    ident.span(),
                );
                variant_parser_names.push(function_name.clone());
                variant_parsers.push(quote! {
                    #[inline(always)]
                    fn #function_name(mut words: parser_lib::VecWindow<parser_lib::Word>) -> parser_lib::ParseResult<Self> #body
                });
            }
            let name = &input.ident;
            let type_name = name.to_string();
            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
            let output = quote! {
                impl #impl_generics parser_lib::Parser<#name> for #name #ty_generics #where_clause {
                    fn parse(words: parser_lib::VecWindow<parser_lib::Word>) -> parser_lib::ParseResult<Self> {
                        let mut errors = Vec::new();
                        #(
                            let parser_lib::ParseResult(res, new_words, new_errors) = Self::#variant_parser_names(words.clone());
                            if let Some(res) = res {
                                parser_lib::log::info!("> {}", #type_name);
                                return parser_lib::ParseResult(Some(res), new_words, new_errors);
                            }
                            errors.extend(new_errors);
                        )*
                        parser_lib::log::debug!("! {} - NoMatch", #type_name);
                        parser_lib::ParseResult(None, words, errors)
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

fn parse_struct(
    fields: &syn::Fields,
    resulting_type: TokenStream2,
    type_name: String,
) -> TokenStream2 {
    match fields {
        syn::Fields::Named(fields) => {
            let mut parse_fields = Vec::new();
            let mut set_fields = Vec::new();
            for (i, field) in fields.named.iter().enumerate() {
                let parse = parse_field(field, type_name.clone());
                let res_name = syn::Ident::new(&format!("res{}", i), field.span());
                let field_name = field
                    .ident
                    .as_ref()
                    .map_or("".to_string(), |i| i.to_string());
                parse_fields.push(quote! {
                    let parser_lib::ParseResult(res, mut words, errors) = #parse;
                    let Some(#res_name) = res else {
                        parser_lib::log::debug!("! {}.{} !! None", #type_name, #field_name);
                        return parser_lib::ParseResult(None, words, [prev_errors, errors].concat());
                    };
                    prev_errors = errors;
                });
                let ident = field.ident.clone().unwrap();
                set_fields.push(quote! {
                    #ident: #res_name,
                });
            }
            quote! {
                {
                    let mut prev_errors = Vec::new();
                    #(#parse_fields)*
                    parser_lib::log::info!("> {}", #type_name);
                    parser_lib::ParseResult(Some(#resulting_type {
                        #(#set_fields)*
                    }), words, prev_errors)
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let mut parse_fields = Vec::new();
            let mut set_fields = Vec::new();
            for (i, field) in fields.unnamed.iter().enumerate() {
                let parse = parse_field(field, type_name.clone());
                let res_name = syn::Ident::new(&format!("res{}", i), field.span());
                parse_fields.push(quote! {
                    let parser_lib::ParseResult(res, mut words, errors) = #parse;
                    let Some(#res_name) = res else {
                        parser_lib::log::debug!("! {}.{} !! None", #type_name, #i);
                        return parser_lib::ParseResult(None, words, [prev_errors, errors].concat());
                    };
                    prev_errors = errors;
                });
                set_fields.push(quote! { #res_name, });
            }
            quote! {
                {
                    let mut prev_errors = Vec::new();
                    #(#parse_fields)*
                    parser_lib::log::info!("> {}", #type_name);
                    parser_lib::ParseResult(Some(#resulting_type (
                        #(#set_fields)*
                    )), words, prev_errors)
                }
            }
        }
        syn::Fields::Unit => quote! {
            parser_lib::log::info!("> {}", #type_name);
            parser_lib::ParseResult(Some(#resulting_type), words, Vec::new())
        },
    }
}

fn parse_field(field: &syn::Field, type_name: String) -> TokenStream2 {
    let parse_attrs = field
        .attrs
        .iter()
        .filter_map(|attr| {
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
            Some(quote! {
                if words.first().and_then(|w| w.get_word()) == Some(&#val.to_string()) {
                    words.pop_first();
                    parser_lib::log::info!("\"{}\"", #val);
                } else {
                    let first = words.first().cloned();
                    parser_lib::log::debug!("! {} - \"{}\" !! \"{}\"", #type_name, #val, first.as_ref().map_or("EOF".to_string(), |x| x.display_text()));
                    return parser_lib::ParseResult(None, words, vec![parser_lib::ParseError {
                        expected: #val.to_string(),
                        got: first,
                    }]);
                }
            })
        })
        .collect::<Vec<_>>();
    let ty = &field.ty;
    let log_field_name = field.ident.as_ref().map(|i| i.to_string()).map_or(
        quote! {},
        |ident| quote! { parser_lib::log::debug!("{}:", #ident) },
    );
    quote! {
        {
            #(#parse_attrs)*
            #log_field_name;
            parser_lib::parse_to_type::<#ty>(words)
        }
    }
}
