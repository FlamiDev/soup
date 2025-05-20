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
            let (keywword, body) =
                parse_struct(&data.fields, quote! { Self }, input.ident.to_string());
            let name = &input.ident;
            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
            let output = quote! {
                impl #impl_generics parser_lib::Parser<#name #ty_generics> for #name #ty_generics #where_clause {
                    fn parse(mut words: parser_lib::VecWindow<parser_lib::Word>) -> parser_lib::ParseResult<Self> #body
                    fn starting_keywords() -> Vec<&'static str> {
                        vec![#keywword]
                    }
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
            let name = &input.ident;
            let type_name = name.to_string();
            let mut variant_parser_calls = Vec::new();
            let mut variant_parsers = Vec::new();
            let mut keywords = Vec::new();
            for variant in &data.variants {
                let ident = &variant.ident;
                let (keyword, body) = parse_struct(
                    &variant.fields,
                    quote! { Self::#ident },
                    format!("{}::{}", input.ident, ident),
                );
                let keyword_check = if let Some(keyword) = keyword {
                    keywords.push(keyword.clone());
                    quote! { words.first().and_then(|w| w.get_word()).is_some_and(|w| w == #keyword) }
                } else {
                    quote! { true }
                };
                let function_name = syn::Ident::new(
                    &format!("parse_{}", ident.to_string().to_lowercase()),
                    ident.span(),
                );
                variant_parser_calls.push(quote! {
                    if #keyword_check {
                        let parser_lib::ParseResult(res, new_words, new_errors) = Self::#function_name(words.clone());
                        if let Some(res) = res {
                            parser_lib::log_end(#type_name);
                            return parser_lib::ParseResult(Some(res), new_words, new_errors);
                        }
                        errors.push(new_errors);
                    }
                });
                variant_parsers.push(quote! {
                    #[inline(always)]
                    fn #function_name(mut words: parser_lib::VecWindow<parser_lib::Word>) -> parser_lib::ParseResult<Self> #body
                });
            }
            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
            let output = quote! {
                impl #impl_generics parser_lib::Parser<#name #ty_generics> for #name #ty_generics #where_clause {
                    fn parse(words: parser_lib::VecWindow<parser_lib::Word>) -> parser_lib::ParseResult<Self> {
                        parser_lib::log_start(#type_name);
                        let first_word = words.first().cloned();
                        let mut errors = Vec::new();
                        #(
                            #variant_parser_calls
                        )*
                        let errors = parser_lib::flatten_branched_errors(errors);
                        parser_lib::log_error(#type_name, &first_word);
                        let error_count_message = format!("errors.len() = {}", errors.len());
                        parser_lib::log_message(#type_name, &error_count_message);
                        parser_lib::ParseResult(None, words, errors)
                    }
                    fn starting_keywords() -> Vec<&'static str> {
                        let mut keywords = Vec::new();
                        #(
                            keywords.push(#keywords);
                        )*
                        keywords
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
) -> (Option<String>, TokenStream2) {
    match fields {
        syn::Fields::Named(fields) => {
            let mut parse_fields = Vec::new();
            let mut set_fields = Vec::new();
            let mut first_attr = None;
            for (i, field) in fields.named.iter().enumerate() {
                let (attr, parse) = parse_field(field, type_name.clone());
                if i == 0 {
                    first_attr = attr;
                }
                let res_name = syn::Ident::new(&format!("res{}", i), field.span());
                let field_name = field
                    .ident
                    .as_ref()
                    .map_or("".to_string(), |i| i.to_string());
                parse_fields.push(quote! {
                    let first_word = words.first().cloned();
                    let parser_lib::ParseResult(res, mut words, new_errors) = #parse;
                    errors.extend(new_errors);
                    let Some(#res_name) = res else {
                        let type_field_name = format!("{}.{}", #type_name, #field_name);
                        parser_lib::log_error(&type_field_name, &first_word);
                        let error_count_message = format!("errors.len() = {}", errors.len());
                        parser_lib::log_message(&type_field_name, &error_count_message);
                        return parser_lib::ParseResult(None, words, errors);
                    };
                });
                let ident = field.ident.clone().unwrap();
                set_fields.push(quote! {
                    #ident: #res_name,
                });
            }
            let res = quote! {
                {
                    parser_lib::log_start(#type_name);
                    let mut errors = Vec::new();
                    #(#parse_fields)*
                    parser_lib::log_end(#type_name);
                    parser_lib::ParseResult(Some(#resulting_type {
                        #(#set_fields)*
                    }), words, errors)
                }
            };
            (first_attr, res)
        }
        syn::Fields::Unnamed(fields) => {
            let mut parse_fields = Vec::new();
            let mut set_fields = Vec::new();
            let mut first_attr = None;
            for (i, field) in fields.unnamed.iter().enumerate() {
                let (attr, parse) = parse_field(field, type_name.clone());
                if i == 0 {
                    first_attr = attr;
                }
                let res_name = syn::Ident::new(&format!("res{}", i), field.span());
                parse_fields.push(quote! {
                    let first_word = words.first().cloned();
                    let parser_lib::ParseResult(res, mut words, new_errors) = #parse;
                    errors.extend(new_errors);
                    let Some(#res_name) = res else {
                        let type_field_name = format!("{}.{}", #type_name, #i);
                        parser_lib::log_error(&type_field_name, &first_word);
                        let error_count_message = format!("errors.len() = {}", errors.len());
                        parser_lib::log_message(&type_field_name, &error_count_message);
                        return parser_lib::ParseResult(None, words, errors);
                    };
                });
                set_fields.push(quote! { #res_name, });
            }
            let res = quote! {
                {
                    parser_lib::log_start(#type_name);
                    let mut errors = Vec::new();
                    #(#parse_fields)*
                    parser_lib::log_end(#type_name);
                    parser_lib::ParseResult(Some(#resulting_type (
                        #(#set_fields)*
                    )), words, errors)
                }
            };
            (first_attr, res)
        }
        syn::Fields::Unit => (
            None,
            quote! {
                {
                    parser_lib::log_end(#type_name);
                    parser_lib::ParseResult(Some(#resulting_type), words, Vec::new())
                }
            },
        ),
    }
}

fn parse_field(field: &syn::Field, type_name: String) -> (Option<String>, TokenStream2) {
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
            let res = quote! {
                if words.first().and_then(|w| w.get_word()) == Some(&#val.to_string()) {
                    words.pop_first();
                    parser_lib::log_parsed(#type_name, &#val);
                } else {
                    let first = words.first().cloned();
                    let display_text = first.as_ref().map_or("EOF".to_string(), |x| x.display_text());
                    parser_lib::log_error(#type_name, &display_text);
                    let error_count_message = format!("errors.len() = {}", errors.len());
                    parser_lib::log_message(#type_name, &error_count_message);
                    return parser_lib::ParseResult(None, words, vec![parser_lib::ParseError {
                        expected: #val.to_string(),
                        got: first,
                        unlikely: false,
                    }]);
                }
            };
            Some((val, res))
        })
        .collect::<Vec<_>>();
    let (attr_names, parse_attrs): (Vec<_>, Vec<_>) = parse_attrs.iter().cloned().unzip();
    let ty = &field.ty;
    let log_field_name = field.ident.as_ref().map(|i| i.to_string()).map_or(
        quote! {},
        |ident| quote! { parser_lib::log_message(#type_name, #ident); },
    );
    let res = quote! {
        {
            #(#parse_attrs)*
            #log_field_name
            parser_lib::parse_to_type::<#ty>(words)
        }
    };
    (attr_names.first().cloned(), res)
}
