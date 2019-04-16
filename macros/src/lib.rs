// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![recursion_limit = "128"]

extern crate proc_macro;

use quote::quote;
use syn::{parse::Error as ParseError, spanned::Spanned};

fn snake_case(ident: &syn::Ident) -> syn::Ident {
    let mut snake = String::new();
    for (i, ch) in ident.to_string().char_indices() {
        if i > 0 && ch.is_uppercase() {
            snake.push('_');
        }
        snake.push(ch.to_ascii_lowercase());
    }
    syn::Ident::new(&snake, ident.span())
}

fn pascal_case(ident: &syn::Ident) -> syn::Ident {
    let mut pascal = String::new();
    let mut capitalize = true;
    for ch in ident.to_string().chars() {
        if ch == '_' {
            capitalize = true;
        } else if capitalize {
            pascal.push(ch.to_ascii_uppercase());
            capitalize = false;
        } else {
            pascal.push(ch);
        }
    }
    syn::Ident::new(&pascal, ident.span())
}

struct JsonRpcApiDef {
    pub(crate) name: syn::LitStr,
    pub(crate) func: syn::Ident,
    pub(crate) prefix: syn::Ident,
    pub(crate) inputs: Vec<syn::Type>,
    pub(crate) output: syn::Type,
}

struct JsonRpcClientDef {
    pub(crate) name: syn::Ident,
    pub(crate) vis: syn::Visibility,
    pub(crate) apis: Vec<JsonRpcApiDef>,
}

impl syn::parse::Parse for JsonRpcClientDef {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let item_trait = {
            // ExprClosure
            let _oror: syn::Token![||] = input.parse()?;
            let content;
            let _braces = syn::braced!(content in input);
            content.parse()?
        };
        Self::parse_item_trait(item_trait)
    }
}

impl JsonRpcClientDef {
    fn parse_item_trait(it: syn::ItemTrait) -> syn::parse::Result<Self> {
        if let Some(tk) = it.unsafety {
            return Err(ParseError::new(tk.span, "don't support `unsafe`"));
        }
        if let Some(tk) = it.auto_token {
            return Err(ParseError::new(tk.span, "don't support `auto`"));
        }
        if !it.generics.params.is_empty() {
            let sp = it.generics.span();
            return Err(ParseError::new(sp, "don't support generics"));
        }
        if !it.supertraits.is_empty() {
            let sp = it.supertraits.span();
            return Err(ParseError::new(sp, "don't support trait bound"));
        }
        let name = it.ident;
        let vis = it.vis;
        let mut apis = Vec::new();
        for ti in it.items.into_iter() {
            let api = Self::parse_trait_item(ti)?;
            apis.push(api);
        }
        Ok(Self { name, vis, apis })
    }

    fn parse_trait_item(ti: syn::TraitItem) -> syn::parse::Result<JsonRpcApiDef> {
        if let syn::TraitItem::Method(tim) = ti {
            let syn::TraitItemMethod {
                attrs,
                sig,
                default,
                ..
            } = tim;
            if !attrs.is_empty() {
                return Err(ParseError::new(attrs[0].span(), "don't support attributes"));
            }
            if default.is_some() {
                return Err(ParseError::new(
                    default.span(),
                    "don't support default implementation",
                ));
            }
            let syn::MethodSig {
                constness,
                unsafety,
                asyncness,
                abi,
                ident,
                decl,
            } = sig;
            if let Some(tk) = constness {
                return Err(ParseError::new(tk.span, "don't support `const`"));
            }
            if let Some(tk) = unsafety {
                return Err(ParseError::new(tk.span, "don't support `unsafe`"));
            }
            if let Some(tk) = asyncness {
                return Err(ParseError::new(tk.span, "don't support `async`"));
            }
            if let Some(tk) = abi {
                return Err(ParseError::new(
                    tk.span(),
                    "don't support  binary interface",
                ));
            }
            if !decl.generics.params.is_empty() {
                let sp = decl.generics.span();
                return Err(ParseError::new(sp, "don't support generics"));
            }
            if let Some(ref tk) = decl.variadic {
                return Err(ParseError::new(tk.span(), "don't support variadic"));
            }
            let name = syn::LitStr::new(&ident.to_string(), ident.span());
            let func = snake_case(&ident);
            let prefix = pascal_case(&ident);
            let mut inputs = Vec::new();
            for input in decl.inputs.into_iter() {
                if let syn::FnArg::Ignored(ty) = input {
                    inputs.push(ty);
                } else {
                    return Err(ParseError::new(
                        input.span(),
                        "only support types not bound to any pattern",
                    ));
                }
            }
            let output = if let syn::ReturnType::Type(_, bt) = decl.output {
                *bt
            } else {
                syn::Type::Tuple(syn::TypeTuple {
                    paren_token: syn::token::Paren::default(),
                    elems: syn::punctuated::Punctuated::new(),
                })
            };
            let api = JsonRpcApiDef {
                name,
                func,
                prefix,
                inputs,
                output,
            };
            Ok(api)
        } else {
            Err(ParseError::new(ti.span(), "only support methods"))
        }
    }
}

fn construct_idents(ident: &syn::Ident) -> (syn::Ident, syn::Ident) {
    let ident_str = ident.to_string();
    let request_str = ident_str.clone() + "JsonRpcRequest";
    let request = syn::Ident::new(&request_str, proc_macro2::Span::call_site());
    let response_str = ident_str + "JsonRpcResponse";
    let response = syn::Ident::new(&response_str, proc_macro2::Span::call_site());
    (request, response)
}

fn construct_jsonrpc_api(
    api: JsonRpcApiDef,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let (request, response) = construct_idents(&api.prefix);
    let common_part = construct_jsonrpc_common_part(&api, &request, &response);
    let JsonRpcApiDef { func, inputs, .. } = api;
    if inputs.is_empty() {
        let define_part = quote!(
            #[derive(Debug)]
            pub struct #request {}
            impl ::std::convert::TryFrom<#request> for jsonrpc_core::Params {
                type Error = serde_json::Error;
                fn try_from(_req: #request) -> serde_json::Result<Self> {
                    Ok(jsonrpc_core::Params::None)
                }
            }
            #common_part
        );
        let impl_part = quote!(
            pub fn #func() -> #request {
                #request {}
            }
        );
        (define_part, impl_part)
    } else {
        let inputs_len = inputs.len() as u64;
        let len = syn::LitInt::new(
            inputs_len,
            syn::IntSuffix::None,
            proc_macro2::Span::call_site(),
        );
        let idx = &(0..inputs_len)
            .map(|i| syn::LitInt::new(i, syn::IntSuffix::None, proc_macro2::Span::call_site()))
            .collect::<Vec<_>>();
        let arg = (0..inputs_len)
            .map(|i| format!("v{}", i))
            .map(|ref i| syn::Ident::new(i, proc_macro2::Span::call_site()))
            .collect::<Vec<_>>();
        let arg1 = &arg;
        let arg2 = &arg;
        let input = &inputs;
        let define_part = quote!(
            #[derive(Debug)]
            pub struct #request (#(#input,)*);
            impl ::std::convert::TryFrom<#request> for jsonrpc_core::Params {
                type Error = serde_json::Error;
                fn try_from(req: #request) -> serde_json::Result<Self> {
                    let mut values = Vec::with_capacity(#len);
                    #(values.push(serde_json::to_value(req.#idx)?);)*
                    Ok(jsonrpc_core::Params::Array(values))
                }
            }
            #common_part
        );
        let impl_part = quote!(
            pub fn #func(#(#arg1: #input,)*) -> #request {
                #request (#(#arg2,)*)
            }
        );
        (define_part, impl_part)
    }
}

fn construct_jsonrpc_common_part(
    api: &JsonRpcApiDef,
    request: &syn::Ident,
    response: &syn::Ident,
) -> proc_macro2::TokenStream {
    let JsonRpcApiDef {
        ref name,
        ref output,
        ..
    } = api;
    quote!(
        pub struct #response(#output);

        impl ::std::convert::From<#response> for #output {
            fn from(r: #response) -> #output {
                r.0
            }
        }

        impl ::std::convert::From<#output> for #response {
            fn from(o: #output) -> #response {
                #response(o)
            }
        }

        impl ::std::convert::TryFrom<serde_json::Value> for #response {
            type Error = serde_json::Error;
            fn try_from(val: serde_json::Value) -> serde_json::Result<Self> {
                serde_json::from_value(val).map(#response)
            }
        }

        impl JsonRpcRequest for #request {
            type Output = #response;

            #[inline]
            fn method() -> &'static str {
                #name
            }
        }
    )
}

#[proc_macro]
pub fn jsonrpc_client(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inputs = syn::parse_macro_input!(input as JsonRpcClientDef);
    let expanded = {
        let JsonRpcClientDef { name, vis, apis } = inputs;
        let mut defs = quote!();
        let mut impls = quote!();
        for api in apis.into_iter() {
            let (define_part, impl_part) = construct_jsonrpc_api(api);
            defs = quote!(#defs #define_part);
            impls = quote!(#impls #impl_part);
        }
        quote!(
            #defs
            #vis struct #name {}
            impl #name {
                #impls
            }
        )
    };
    expanded.into()
}
