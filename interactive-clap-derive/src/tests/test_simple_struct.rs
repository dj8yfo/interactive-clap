pub fn pretty_codegen(ts: &proc_macro2::TokenStream) -> String {
    let file = syn::parse_file(&ts.to_string()).unwrap();
    prettyplease::unparse(&file)
}

#[test]
fn test_simple_struct() {
    let input = syn::parse_quote! {
        struct Args {
            age: u64,
            first_name: String,
            second_name: String,
        }
    };

    let interactive_clap_codegen = crate::derives::interactive_clap::impl_interactive_clap(&input);
    insta::assert_snapshot!(pretty_codegen(&interactive_clap_codegen));

    let step_one_output = syn::parse_quote! {
        pub struct CliArgs {
            pub age: Option<<u64 as interactive_clap::ToCli>::CliVariant>,
            pub first_name: Option<<String as interactive_clap::ToCli>::CliVariant>,
            pub second_name: Option<<String as interactive_clap::ToCli>::CliVariant>,
        }
    };

    let to_cli_args_codegen = crate::derives::to_cli_args::impl_to_cli_args(&step_one_output);
    insta::assert_snapshot!(pretty_codegen(&to_cli_args_codegen));
}

#[test]
fn test_simple_struct_with_named_arg() {
    let input = syn::parse_quote! {
        struct Account {
            #[interactive_clap(named_arg)]
            field_name: Sender,
        }
    };

    let interactive_clap_codegen = crate::derives::interactive_clap::impl_interactive_clap(&input);
    insta::assert_snapshot!(pretty_codegen(&interactive_clap_codegen));

    let step_one_output = syn::parse_quote! {
        pub struct CliAccount {
            #[clap(subcommand)]
            pub field_name: Option<ClapNamedArgSenderForAccount>,
        }
    };

    let to_cli_args_codegen = crate::derives::to_cli_args::impl_to_cli_args(&step_one_output);
    insta::assert_snapshot!(pretty_codegen(&to_cli_args_codegen));
}

#[test]
fn test_doc_comments_propagate() {
    let input = syn::parse_quote! {
        struct Args {
            /// short first field description
            ///
            /// a longer paragraph, describing the usage and stuff with first field's
            /// awarenes of its possible applications
            #[interactive_clap(long)]
            #[interactive_clap(skip_interactive_input)]
            first_field: u64,
            /// short second field description
            ///
            /// a longer paragraph, describing the usage and stuff with second field's
            /// awareness of its possible applications
            #[interactive_clap(long)]
            #[interactive_clap(skip_interactive_input)]
            #[interactive_clap(verbatim_doc_comment)]
            second_field: String,
            /// short third field description
            ///
            /// a longer paragraph, describing the usage and stuff with third field's
            /// awareness of its possible applications
            #[interactive_clap(long)]
            #[interactive_clap(skip_interactive_input)]
            #[interactive_clap(verbatim_doc_comment)]
            third_field: bool,
        }
    };

    let interactive_clap_codegen = crate::derives::interactive_clap::impl_interactive_clap(&input);
    insta::assert_snapshot!(pretty_codegen(&interactive_clap_codegen));

    let step_one_output = syn::parse_quote! {
        pub struct CliArgs {
            #[clap(long)]
            pub first_field: Option<<u64 as interactive_clap::ToCli>::CliVariant>,
            #[clap(long)]
            pub second_field: Option<<String as interactive_clap::ToCli>::CliVariant>,
            #[clap(long)]
            pub third_field: bool,
        }
    };

    let to_cli_args_codegen = crate::derives::to_cli_args::impl_to_cli_args(&step_one_output);
    insta::assert_snapshot!(pretty_codegen(&to_cli_args_codegen));
}

#[test]
fn test_flag() {
    let input = syn::parse_quote! {
        struct Args {
            /// Offline mode
            #[interactive_clap(long)]
            offline: bool
        }
    };

    let interactive_clap_codegen = crate::derives::interactive_clap::impl_interactive_clap(&input);
    insta::assert_snapshot!(pretty_codegen(&interactive_clap_codegen));

    let input = syn::parse_quote! {
        struct CliArgs {
            /// Offline mode
            #[clap(long)]
            offline: bool
        }
    };

    let to_cli_args_codegen = crate::derives::to_cli_args::impl_to_cli_args(&input);
    insta::assert_snapshot!(pretty_codegen(&to_cli_args_codegen));
}

#[test]
fn test_vec_multiple_opt() {
    let input = syn::parse_quote! {
        struct Args {
            #[interactive_clap(long_vec_multiple_opt)]
            pub env: Vec<String>,
        }
    };

    let interactive_clap_codegen = crate::derives::interactive_clap::impl_interactive_clap(&input);
    insta::assert_snapshot!(pretty_codegen(&interactive_clap_codegen));
}

#[test]
fn test_vec_multiple_opt_to_cli_args() {
    let input = syn::parse_quote! {
        pub struct CliArgs {
            #[clap(long)]
            pub env: Vec<String>,
        }
    };

    let to_cli_args_codegen = crate::derives::to_cli_args::impl_to_cli_args(&input);
    insta::assert_snapshot!(pretty_codegen(&to_cli_args_codegen));
}

#[test]
// testing correct panic msg isn't really very compatible with
// `proc-macro-error` crate
#[should_panic]
fn test_vec_multiple_opt_err() {
    let input = syn::parse_quote! {
        struct Args {
            #[interactive_clap(long_vec_multiple_opt)]
            pub env: String,
        }
    };

    let interactive_clap_codegen = crate::derives::interactive_clap::impl_interactive_clap(&input);
    insta::assert_snapshot!(pretty_codegen(&interactive_clap_codegen));
}
