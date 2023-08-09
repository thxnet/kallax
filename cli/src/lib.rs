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
        private_in_public,
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

use std::{fmt, future::Future, io::Write};

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
    fn default() -> Self { Self::parse() }
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
        config: session_key::Config,
    },

    #[command(about = "Run initializer for starting Substrate-based node")]
    Initializer {
        #[clap(flatten)]
        config: initializer::Config,
    },

    #[command(about = "Run Kubernetes sidecar for Substrate-based node")]
    Sidecar {
        #[clap(flatten)]
        config: sidecar::Config,
    },

    #[command(about = "Run network broker for Substrate-based node which is out of Kubernetes")]
    NetworkBroker {
        #[clap(flatten)]
        config: network_broker::Config,
    },

    #[command(about = "Run tracker for Substrate-based node")]
    Tracker {
        #[clap(flatten)]
        config: tracker::Config,
    },
}

impl Cli {
    /// # Errors
    ///
    /// This function returns an error if the command is not executed properly.
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
            Commands::SessionKey { config } => {
                execute("Session key", async { session_key::run(config).await })
            }
            Commands::Initializer { config } => {
                execute("Initializer", async { initializer::run(config).await })
            }
            Commands::Sidecar { config } => {
                execute("Sidecar", async { sidecar::run(config).await })
            }
            Commands::NetworkBroker { config } => {
                execute("Network Broker", async { network_broker::run(config).await })
            }
            Commands::Tracker { config } => {
                execute("Tracker", async { tracker::run(config).await })
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

    use crate::{Cli, Commands};

    #[test]
    fn test_command_version() {
        match Cli::parse_from(["program_name", "version"]).commands {
            Commands::Version => (),
            _ => panic!(),
        }
    }

    #[test]
    fn test_command_sidecar() {
        match Cli::parse_from([
            "program_name",
            "sidecar",
            "--tracker-grpc-endpoint=http://kallax-tracker.mainnet.svc.cluster.local:80",
            "--rootchain-id=mainnet",
            "--rootchain-node-websocket-endpoint=ws://127.0.0.1:50002",
        ])
        .commands
        {
            Commands::Sidecar { config: _ } => (),
            _ => panic!(),
        }
    }
}
