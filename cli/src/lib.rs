// the rules is in following order:
//  - RUSTC ALLOW
//  - RUSTC WARNING
//  - CLIPPY
// rustc rules not enabled:
//  - box_pointers
//  - missing_copy_implementations
//  - missing_debug_implementations
//  - missing_docs
//  - non_exhaustive_omitted_patterns
//  - unreachable_pub
//  - unsafe_code
//  - unused_crate_dependencies
//  - unused_qualifications
//  - unused_results
//  - variant_size_differences
#![cfg_attr(
    feature = "cargo-clippy",
    cfg_attr(feature = "c_unwind", deny(ffi_unwind_calls)),
    cfg_attr(feature = "strict_provenance", deny(fuzzy_provenance_casts, lossy_provenance_casts)),
    cfg_attr(feature = "must_not_suspend", deny(must_not_suspend)),
    cfg_attr(feature = "lint_reasons", deny(unfulfilled_lint_expectations)),
    deny(
        absolute_paths_not_starting_with_crate,
        deprecated_in_future,
        elided_lifetimes_in_paths,
        explicit_outlives_requirements,
        keyword_idents,
        let_underscore_drop,
        macro_use_extern_crate,
        meta_variable_misuse,
        missing_abi,
        non_ascii_idents,
        noop_method_call,
        pointer_structural_match,
        rust_2021_incompatible_closure_captures,
        rust_2021_incompatible_or_patterns,
        rust_2021_prefixes_incompatible_syntax,
        rust_2021_prelude_collisions,
        single_use_lifetimes,
        trivial_casts,
        trivial_numeric_casts,
        unsafe_op_in_unsafe_fn,
        unused_extern_crates,
        unused_import_braces,
        unused_lifetimes,
        unused_macro_rules,
        unused_tuple_struct_fields,
        anonymous_parameters,
        array_into_iter,
        asm_sub_register,
        bad_asm_style,
        bare_trait_objects,
        bindings_with_variant_name,
        break_with_label_and_loop,
        clashing_extern_declarations,
        coherence_leak_check,
        confusable_idents,
        const_evaluatable_unchecked,
        const_item_mutation,
        dead_code,
        deprecated_where_clause_location,
        deref_into_dyn_supertrait,
        deref_nullptr,
        drop_bounds,
        duplicate_macro_attributes,
        dyn_drop,
        ellipsis_inclusive_range_patterns,
        exported_private_dependencies,
        for_loops_over_fallibles,
        forbidden_lint_groups,
        function_item_references,
        illegal_floating_point_literal_pattern,
        improper_ctypes,
        improper_ctypes_definitions,
        incomplete_features,
        indirect_structural_match,
        inline_no_sanitize,
        invalid_doc_attributes,
        invalid_value,
        irrefutable_let_patterns,
        large_assignments,
        late_bound_lifetime_arguments,
        legacy_derive_helpers,
        mixed_script_confusables,
        named_arguments_used_positionally,
        no_mangle_generic_items,
        non_camel_case_types,
        non_fmt_panics,
        non_shorthand_field_patterns,
        non_snake_case,
        non_upper_case_globals,
        nontrivial_structural_match,
        opaque_hidden_inferred_bound,
        overlapping_range_endpoints,
        path_statements,
        redundant_semicolons,
        renamed_and_removed_lints,
        repr_transparent_external_private_fields,
        semicolon_in_expressions_from_macros,
        special_module_name,
        stable_features,
        suspicious_auto_trait_impls,
        temporary_cstring_as_ptr,
        trivial_bounds,
        type_alias_bounds,
        tyvar_behind_raw_pointer,
        uncommon_codepoints,
        unconditional_recursion,
        unexpected_cfgs,
        uninhabited_static,
        unknown_lints,
        unnameable_test_items,
        unreachable_code,
        unreachable_patterns,
        unstable_name_collisions,
        unstable_syntax_pre_expansion,
        unsupported_calling_conventions,
        unused_allocation,
        unused_assignments,
        unused_attributes,
        unused_braces,
        unused_comparisons,
        unused_doc_comments,
        unused_features,
        unused_imports,
        unused_labels,
        unused_macros,
        unused_must_use,
        unused_mut,
        unused_parens,
        unused_unsafe,
        unused_variables,
        where_clauses_object_safety,
        while_true,
        clippy::all,
        clippy::cargo,
        clippy::nursery,
        clippy::pedantic
    ),
    warn(unstable_features),
    allow(
        clippy::future_not_send,
        clippy::module_name_repetitions,
        clippy::multiple_crate_versions,
    )
)]

mod consts;
mod error;
mod initializer;
mod network_broker;
mod session_key;
mod sidecar;
mod tracker;

use std::{fmt, future::Future, io::Write, path::PathBuf};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use snafu::ResultExt;
use tokio::runtime;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use self::error::Result;
pub use self::error::{CommandError, Error};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

impl Default for Cli {
    #[inline]
    fn default() -> Self {
        Self::parse()
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Show current version")]
    Version,

    #[command(about = "Show shell completions")]
    Completions { shell: Shell },

    #[command(about = "Generate session keys")]
    SessionKey {
        #[clap(flatten)]
        options: session_key::Options,
    },

    #[command(about = "Run initializer for starting Substrate-based node")]
    Initializer {
        #[clap(flatten)]
        options: initializer::Options,
    },

    #[command(about = "Run Kubernetes sidecar for Substrate-based node")]
    Sidecar {
        #[clap(flatten)]
        options: sidecar::Options,
    },

    #[command(about = "Run network broker for Substrate-based node which is out of Kubernetes")]
    NetworkBroker {
        #[clap(
            long = "tracker-api-endpoint",
            help = "Tracker api endpoint",
            default_value = network_broker::TRACKER_API_ENDPOINT
        )]
        tracker_api_endpoint: http::Uri,

        #[clap(
            short = 'f',
            long = "file",
            help = "Config file path",
            default_value = network_broker::CONFIG_PATH
        )]
        file: PathBuf,
    },

    #[command(about = "Run tracker for Substrate-based node")]
    Tracker {
        #[clap(flatten)]
        options: tracker::Options,
    },
}

impl Cli {
    /// # Errors
    ///
    /// This function returns an error if the command is not executed properly.
    ///
    /// # Panics
    /// This function never panics.
    pub fn run(self) -> Result<()> {
        match self.commands {
            Commands::Version => {
                let mut stdout = std::io::stdout();
                stdout
                    .write_all(Self::command().render_long_version().as_bytes())
                    .expect("failed to write to stdout");
                Ok(())
            }
            Commands::Completions { shell } => {
                let mut app = Self::command();
                let bin_name = app.get_name().to_string();
                clap_complete::generate(shell, &mut app, bin_name, &mut std::io::stdout());
                Ok(())
            }
            Commands::SessionKey { options } => {
                execute("Session key", async { session_key::run(options).await })
            }
            Commands::Initializer { options } => {
                execute("Initializer", async { initializer::run(options).await })
            }
            Commands::Sidecar { options } => {
                execute("Sidecar", async { sidecar::run(options).await })
            }
            Commands::NetworkBroker { tracker_api_endpoint, file } => {
                execute("Network Broker", async {
                    network_broker::run(tracker_api_endpoint, file).await
                })
            }
            Commands::Tracker { options } => {
                execute("Tracker", async { tracker::run(options).await })
            }
        }
    }
}

#[inline]
fn execute<S, F, E>(command_name: S, fut: F) -> Result<()>
where
    S: fmt::Display,
    F: Future<Output = std::result::Result<(), E>>,
    E: Into<Error>,
{
    init_tracing();

    tracing::info!("Starting {}", Cli::command().get_long_version().unwrap_or_default());
    tracing::info!("Run {command_name}");

    tracing::info!("Initializing Tokio runtime");
    let runtime = runtime::Builder::new_multi_thread()
        .thread_name(consts::THREAD_NAME)
        .enable_all()
        .build()
        .context(error::InitializeTokioRuntimeSnafu)?;

    runtime.block_on(fut).map_err(Into::into)
}

fn init_tracing() {
    // filter
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    // format
    let fmt_layer =
        tracing_subscriber::fmt::layer().pretty().with_thread_ids(true).with_thread_names(true);
    // subscriber
    tracing_subscriber::registry().with(filter_layer).with(fmt_layer).init();
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use std::path::PathBuf;

    use crate::{Cli, Commands};

    #[test]
    fn test_command_version() {
        assert!(matches!(Cli::parse_from(["program_name", "version"]).commands, Commands::Version));
    }

    #[test]
    fn test_command_sidecar() {
        if let Commands::Sidecar { options: _ } = Cli::parse_from([
            "program_name",
            "sidecar",
            "--tracker-grpc-endpoint=http://kallax-tracker.mainnet.svc.cluster.local:80",
            "--rootchain-id=mainnet",
            "--rootchain-node-websocket-endpoint=ws://127.0.0.1:50002",
        ])
        .commands
        {
            // Everything is good here.
        } else {
            panic!();
        }
    }

    #[test]
    fn test_command_completions_bash() {
        if let Commands::Completions { shell: _ } =
            Cli::parse_from(["program_name", "completions", "bash"]).commands
        {
            // Everything is good here.
        } else {
            panic!();
        }
    }

    #[test]
    fn test_command_completions_fish() {
        if let Commands::Completions { shell: _ } =
            Cli::parse_from(["program_name", "completions", "fish"]).commands
        {
            // Everything is good here.
        } else {
            panic!();
        }
    }

    #[test]
    fn test_command_completions_zsh() {
        if let Commands::Completions { shell: _ } =
            Cli::parse_from(["program_name", "completions", "zsh"]).commands
        {
            // Everything is good here.
        } else {
            panic!();
        }
    }

    #[test]
    fn test_command_completions_powershell() {
        if let Commands::Completions { shell: _ } =
            Cli::parse_from(["program_name", "completions", "powershell"]).commands
        {
            // Everything is good here.
        } else {
            panic!();
        }
    }

    #[test]
    fn test_command_completions_elvish() {
        if let Commands::Completions { shell: _ } =
            Cli::parse_from(["program_name", "completions", "elvish"]).commands
        {
            // Everything is good here.
        } else {
            panic!();
        }
    }

    #[test]
    fn test_command_session_key_parsing() {
        if let Commands::SessionKey { options: _ } = Cli::parse_from([
            "program_name",
            "session-key",
            "--keystore-directory-path=/tmp/keystore",
            "--session-key-mnemonic-phrase=test mnemonic phrase",
            "--node-name=test-node",
        ])
        .commands
        {
            // Everything is good here.
        } else {
            panic!();
        }
    }

    #[test]
    fn test_command_session_key_parsing_with_env_prefix() {
        // Test that the command can be parsed with all required arguments
        let cli = Cli::try_parse_from([
            "program_name",
            "session-key",
            "--keystore-directory-path=/tmp/keystore",
            "--session-key-mnemonic-phrase=bottom drive obey lake curtain smoke basket hold race lonely fit walk",
            "--node-name=test-node-1",
        ]);
        assert!(cli.is_ok(), "Failed to parse session-key command: {:?}", cli.err());
        let cli = cli.unwrap();
        assert!(matches!(cli.commands, Commands::SessionKey { options: _ }));
    }

    #[test]
    fn test_command_session_key_missing_required_arg() {
        // Test error case: missing required argument
        let cli = Cli::try_parse_from([
            "program_name",
            "session-key",
            "--keystore-directory-path=/tmp/keystore",
            "--session-key-mnemonic-phrase=test phrase",
            // Missing --node-name
        ]);
        assert!(cli.is_err(), "Should fail when missing required --node-name argument");
    }

    #[test]
    fn test_command_session_key_missing_all_required_args() {
        // Test error case: missing all required arguments
        let cli = Cli::try_parse_from(["program_name", "session-key"]);
        assert!(cli.is_err(), "Should fail when missing all required arguments");
    }

    #[test]
    fn test_command_invalid_subcommand() {
        // Test error case: invalid subcommand
        let cli = Cli::try_parse_from(["program_name", "invalid-subcommand"]);
        assert!(cli.is_err(), "Should fail for invalid subcommand");
    }

    #[test]
    fn test_command_invalid_shell_type() {
        // Test error case: invalid shell type for completions
        let cli = Cli::try_parse_from(["program_name", "completions", "invalid-shell"]);
        assert!(cli.is_err(), "Should fail for invalid shell type");
    }

    #[test]
    fn test_command_network_broker() {
        if let Commands::NetworkBroker { tracker_api_endpoint, file } = Cli::parse_from([
            "program_name",
            "network-broker",
            "--tracker-api-endpoint=https://tracker.example.com",
            "--file=/tmp/config.json",
        ])
        .commands
        {
            assert_eq!(tracker_api_endpoint.to_string(), "https://tracker.example.com/");
            assert_eq!(file, PathBuf::from("/tmp/config.json"));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_command_network_broker_defaults() {
        if let Commands::NetworkBroker { tracker_api_endpoint, file } =
            Cli::parse_from(["program_name", "network-broker"]).commands
        {
            // Default values should be set
            assert!(!tracker_api_endpoint.to_string().is_empty());
            assert!(!file.as_os_str().is_empty());
        } else {
            panic!();
        }
    }

    #[test]
    fn test_command_initializer() {
        if let Commands::Initializer { options: _ } = Cli::parse_from([
            "program_name",
            "initializer",
            "--node-key-file-path=/tmp/node.key",
            "--tracker-grpc-endpoint=http://localhost:53973",
            "--rootchain-id=rootchain",
            "--rootchain-spec-file-path=/tmp/rootchain.json",
            "--keystore-directory-path=/tmp/keystore",
        ])
        .commands
        {
            // Everything is good here.
        } else {
            panic!();
        }
    }

    #[test]
    fn test_command_tracker() {
        if let Commands::Tracker { options: _ } =
            Cli::parse_from(["program_name", "tracker"]).commands
        {
            // Everything is good here.
        } else {
            panic!();
        }
    }
}
