extern crate proc_macro;

use proc_macro2::Span;
use quote::quote;
use syn;

use super::common_methods as structs_methods;
use crate::derives::interactive_clap::common_methods;

/// returns the whole result `TokenStream` of derive logic of containing module
pub fn token_stream(ast: &syn::DeriveInput, fields: &syn::Fields) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let vec_fn_input_arg = vec_fn_input_arg(ast, &fields);
    quote! {
        impl #name {
            #(#vec_fn_input_arg)*
        }
    }
}

fn vec_fn_input_arg(ast: &syn::DeriveInput, fields: &syn::Fields) -> Vec<proc_macro2::TokenStream> {
    let interactive_clap_attrs_context =
        common_methods::interactive_clap_attrs_context::InteractiveClapAttrsContext::new(ast);
    let vec_fn_input_arg = fields
        .iter()
        .filter(|field| !structs_methods::is_field_with_subcommand::predicate(field))
        .filter(|field| {
            !common_methods::fields_with_skip_default_input_arg::is_field_with_skip_default_input_arg(
                field,
            )
        })
        .map(|field| {
            let ident_field = &field.clone().ident.expect("this field does not exist");
            let ty = &field.ty;

            let input_context_dir = interactive_clap_attrs_context
                .clone()
                .get_input_context_dir();

            let fn_input_arg =
                syn::Ident::new(&format!("input_{}", &ident_field), Span::call_site());

            if field.attrs.is_empty() {
                let promt = &syn::LitStr::new(&ident_field.to_string(), Span::call_site());
                return quote! {
                    fn #fn_input_arg(
                        _context: &#input_context_dir,
                    ) -> color_eyre::eyre::Result<Option<#ty>> {
                        match inquire::CustomType::new(#promt).prompt() {
                            Ok(value) => Ok(Some(value)),
                            Err(inquire::error::InquireError::OperationCanceled | inquire::error::InquireError::OperationInterrupted) => Ok(None),
                            Err(err) => Err(err.into()),
                        }
                    }
                };
            }

            if common_methods::skip_interactive_input::is_skip_interactive_input(field) {
                return quote! {};
            }

            let doc_attrs = field
                .attrs
                .iter()
                .filter(|attr| attr.path.is_ident("doc"))
                .filter_map(|attr| {
                    for attr_token in attr.tokens.clone() {
                        if let proc_macro2::TokenTree::Literal(literal) = attr_token {
                            return Some(literal);
                        }
                    }
                    None
                });

            quote! {
                fn #fn_input_arg(
                    _context: &#input_context_dir,
                ) -> color_eyre::eyre::Result<Option<#ty>> {
                    match inquire::CustomType::new(concat!(#( #doc_attrs, )*).trim()).prompt() {
                        Ok(value) => Ok(Some(value)),
                        Err(inquire::error::InquireError::OperationCanceled | inquire::error::InquireError::OperationInterrupted) => Ok(None),
                        Err(err) => Err(err.into()),
                    }
                }
            }
        })
        .filter(|token_stream| !token_stream.is_empty())
        .collect::<Vec<proc_macro2::TokenStream>>();
    vec_fn_input_arg
}
