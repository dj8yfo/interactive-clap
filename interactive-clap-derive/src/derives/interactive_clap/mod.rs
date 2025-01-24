extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort_call_site;
use quote::{quote, ToTokens};
use syn;

pub fn impl_interactive_clap(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let cli_name = {
        let cli_name_string = format!("Cli{}", name);
        &syn::Ident::new(&cli_name_string, Span::call_site())
    };
    match &ast.data {
        syn::Data::Struct(data_struct) => {
            let fields = data_struct.fields.clone();

            self::structs::token_stream(name, cli_name, ast, &fields)
        }
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let enum_variants = variants.iter().map(|variant| {
                let ident = &variant.ident;
                let mut attrs: Vec<proc_macro2::TokenStream> = Vec::new();
                if !&variant.attrs.is_empty() {
                    for attr in &variant.attrs {
                        if attr.path.is_ident("doc") {
                            attrs.push(attr.into_token_stream());
                        };
                        if attr.path.is_ident("cfg") {
                            for attr_token in attr.tokens.clone() {
                                match attr_token {
                                    proc_macro2::TokenTree::Group(group) => {
                                        if group.stream().to_string().contains("feature") {
                                            attrs.push(attr.into_token_stream());
                                        } else {
                                            continue;
                                        };
                                    }
                                    _ => {
                                        abort_call_site!("Only option `TokenTree::Group` is needed")
                                    }
                                }
                            }
                        };
                    }
                    match &variant.fields {
                        syn::Fields::Unnamed(fields) => {
                            let ty = &fields.unnamed[0].ty;
                            if attrs.is_empty() {
                                quote! {#ident(<#ty as interactive_clap::ToCli>::CliVariant)}
                            } else {
                                quote! {
                                    #(#attrs)*
                                    #ident(<#ty as interactive_clap::ToCli>::CliVariant)
                                }
                            }
                        }
                        syn::Fields::Unit => {
                            if attrs.is_empty() {
                                quote! {#ident}
                            } else {
                                quote! {
                                    #(#attrs)*
                                    #ident
                                }
                            }
                        }
                        _ => abort_call_site!(
                            "Only option `Fields::Unnamed` or `Fields::Unit` is needed"
                        ),
                    }
                } else {
                    match &variant.fields {
                        syn::Fields::Unnamed(fields) => {
                            let ty = &fields.unnamed[0].ty;
                            quote! { #ident(<#ty as interactive_clap::ToCli>::CliVariant) }
                        }
                        syn::Fields::Unit => {
                            quote! { #ident }
                        }
                        _ => abort_call_site!(
                            "Only option `Fields::Unnamed` or `Fields::Unit` is needed"
                        ),
                    }
                }
            });
            let for_cli_enum_variants = variants.iter().map(|variant| {
                let ident = &variant.ident;
                match &variant.fields {
                    syn::Fields::Unnamed(_) => {
                        quote! { #name::#ident(arg) => Self::#ident(arg.into()) }
                    }
                    syn::Fields::Unit => {
                        quote! { #name::#ident => Self::#ident }
                    }
                    _ => abort_call_site!(
                        "Only option `Fields::Unnamed` or `Fields::Unit` is needed"
                    ),
                }
            });

            let scope_for_enum = context_scope_for_enum(name);

            let fn_choose_variant =
                self::common_methods::choose_variant::fn_choose_variant(ast, variants);

            let fn_from_cli_for_enum =
                self::common_methods::from_cli_for_enum::from_cli_for_enum(ast, variants);

            quote! {
                #[derive(Debug, Clone, clap::Parser, interactive_clap::ToCliArgs)]
                pub enum #cli_name {
                    #( #enum_variants, )*
                }

                impl interactive_clap::ToCli for #name {
                    type CliVariant = #cli_name;
                }

                #scope_for_enum

                impl From<#name> for #cli_name {
                    fn from(command: #name) -> Self {
                        match command {
                            #( #for_cli_enum_variants, )*
                        }
                    }
                }

                #fn_from_cli_for_enum

                impl #name {
                    #fn_choose_variant

                    pub fn try_parse() -> Result<#cli_name, clap::Error> {
                        <#cli_name as clap::Parser>::try_parse()
                    }

                    pub fn parse() -> #cli_name {
                        <#cli_name as clap::Parser>::parse()
                    }

                    pub fn try_parse_from<I, T>(itr: I) -> Result<#cli_name, clap::Error>
                    where
                        I: ::std::iter::IntoIterator<Item = T>,
                        T: ::std::convert::Into<::std::ffi::OsString> + ::std::clone::Clone,
                    {
                        <#cli_name as clap::Parser>::try_parse_from(itr)
                    }
                }
            }
        }
        _ => abort_call_site!("`#[derive(InteractiveClap)]` only supports structs and enums"),
    }
}

/// these are common methods, reused for both the [structs] and `enums` derives
pub(super) mod common_methods;

/** This module describes [`crate::InteractiveClap`] derive logic in case when [`syn::DeriveInput`]
is a struct

The structure of produced derive output is as follows, where code blocks are generated by
submodules with corresponding names:

```rust,ignore
quote::quote! {
    #to_cli_trait_block
    #input_args_impl_block
    #to_interactive_clap_context_scope_trait_block
    #from_cli_trait_block
    #clap_for_named_arg_enum_block
}
```
*/
mod structs {
    /// returns the whole result `TokenStream` of derive logic of containing module
    pub fn token_stream(
        name: &syn::Ident,
        cli_name: &syn::Ident,
        ast: &syn::DeriveInput,
        fields: &syn::Fields,
    ) -> proc_macro2::TokenStream {
        let to_cli_trait_block = to_cli_trait::token_stream(name, cli_name, &fields);
        let from_cli_trait_block = from_cli_trait::token_stream(ast, &fields);
        let input_args_impl_block = input_args_impl::token_stream(ast, &fields);
        let to_interactive_clap_context_scope_trait_block =
            to_interactive_clap_context_scope_trait::token_stream(ast, &fields);
        let clap_for_named_arg_enum_block = clap_for_named_arg_enum::token_stream(ast, &fields);

        quote::quote! {
            #to_cli_trait_block
            #input_args_impl_block
            #to_interactive_clap_context_scope_trait_block
            #from_cli_trait_block
            #clap_for_named_arg_enum_block
        }
    }

    #[doc = include_str!("../../../docs/structs_to_cli_trait_docstring.md")]
    mod to_cli_trait;

    #[doc = include_str!("../../../docs/structs_input_args_impl_docstring.md")]
    mod input_args_impl;

    #[doc = include_str!("../../../docs/structs_to_interactive_clap_context_scope_trait_docstring.md")]
    mod to_interactive_clap_context_scope_trait;

    #[doc = include_str!("../../../docs/structs_from_cli_trait_docstring.md")]
    mod from_cli_trait;

    #[doc = include_str!("../../../docs/clap_enum_for_named_arg_docstring.md")]
    mod clap_for_named_arg_enum;

    /// these are common field methods, reused by other [structs](super::structs) submodules
    pub(super) mod common_field_methods;
}

fn context_scope_for_enum(name: &syn::Ident) -> proc_macro2::TokenStream {
    let interactive_clap_context_scope_for_enum = syn::Ident::new(
        &format!("InteractiveClapContextScopeFor{}", &name),
        Span::call_site(),
    );
    let enum_discriminants = syn::Ident::new(&format!("{}Discriminants", &name), Span::call_site());
    quote! {
        pub type #interactive_clap_context_scope_for_enum = #enum_discriminants;
        impl interactive_clap::ToInteractiveClapContextScope for #name {
                    type InteractiveClapContextScope = #interactive_clap_context_scope_for_enum;
                }
    }
}
