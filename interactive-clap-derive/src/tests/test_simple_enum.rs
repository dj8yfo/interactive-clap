use super::test_simple_struct::pretty_codegen;

#[test]
fn test_simple_enum() {
    let input = syn::parse_quote! {
        pub enum Mode {
            /// Prepare and, optionally, submit a new transaction with online mode
            Network,
            /// Prepare and, optionally, submit a new transaction with offline mode
            Offline,
        }
    };

    let interactive_clap_codegen = crate::derives::interactive_clap::impl_interactive_clap(&input);
    insta::assert_snapshot!(pretty_codegen(&interactive_clap_codegen));

    let step_one_output = syn::parse_quote! {
        pub enum CliMode {
            /// Prepare and, optionally, submit a new transaction with online mode
            Network,
            /// Prepare and, optionally, submit a new transaction with offline mode
            Offline,
        }
    };

    let to_cli_args_codegen = crate::derives::to_cli_args::impl_to_cli_args(&step_one_output);
    insta::assert_snapshot!(pretty_codegen(&to_cli_args_codegen));
}
