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
    allow(clippy::multiple_crate_versions,)
)]

mod diagnostic;
mod error;
mod peer_discoverer;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use futures::{future, future::Either, FutureExt, StreamExt};
use kallax_primitives::{BlockchainLayer, ExternalEndpoint};
use kallax_tracker_grpc_client::{Client as TrackerClient, Config as TrackerClientConfig};
use snafu::ResultExt;
use tokio::sync::Mutex;

pub use self::error::{Error, Result};
use self::peer_discoverer::{ErrorRing, PeerDiscoverer};

#[derive(Clone, Debug)]
pub struct Config {
    pub tracker_grpc_endpoint: http::Uri,

    pub polling_interval: Duration,

    pub rootchain_endpoint: ChainEndpoint,

    pub leafchain_endpoint: Option<ChainEndpoint>,

    pub allow_loopback_ip: bool,

    pub external_rootchain_p2p_endpoint: Option<ExternalEndpoint>,

    pub external_leafchain_p2p_endpoint: Option<ExternalEndpoint>,

    pub diagnostic_listen_address: SocketAddr,

    pub detected_public_ip: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ChainEndpoint {
    pub chain_id: String,

    pub websocket_endpoint: http::Uri,
}

/// # Errors
///
/// This function returns an error if the server is not connected.
#[allow(clippy::significant_drop_tightening)]
pub async fn serve(config: Config) -> Result<()> {
    let Config {
        tracker_grpc_endpoint,
        polling_interval,
        rootchain_endpoint,
        leafchain_endpoint,
        allow_loopback_ip,
        external_rootchain_p2p_endpoint,
        external_leafchain_p2p_endpoint,
        diagnostic_listen_address,
        detected_public_ip,
    } = config;

    let tracker_client =
        TrackerClient::new(TrackerClientConfig { grpc_endpoint: tracker_grpc_endpoint.clone() })
            .await
            .with_context(|_| error::ConnectTrackerSnafu { uri: tracker_grpc_endpoint.clone() })?;

    let rootchain_diagnostic: peer_discoverer::SharedDiagnostic = Arc::new(Mutex::new(None));
    let leafchain_diagnostic: peer_discoverer::SharedDiagnostic = Arc::new(Mutex::new(None));
    let rootchain_errors: peer_discoverer::SharedErrorRing = Arc::new(Mutex::new(ErrorRing::new()));
    let leafchain_errors: peer_discoverer::SharedErrorRing = Arc::new(Mutex::new(ErrorRing::new()));
    let start_time = std::time::Instant::now();

    let diagnostic_config = diagnostic::SidecarDiagnosticConfig {
        tracker_grpc_endpoint: tracker_grpc_endpoint.to_string(),
        rootchain_id: rootchain_endpoint.chain_id.clone(),
        leafchain_id: leafchain_endpoint.as_ref().map(|e| e.chain_id.clone()),
        allow_loopback_ip,
        external_rootchain_p2p_endpoint: external_rootchain_p2p_endpoint
            .as_ref()
            .map(ToString::to_string),
        external_leafchain_p2p_endpoint: external_leafchain_p2p_endpoint
            .as_ref()
            .map(ToString::to_string),
        polling_interval_ms: polling_interval.as_millis() as u64,
        detected_public_ip: detected_public_ip.clone(),
    };

    let lifecycle_manager = sigfinn::LifecycleManager::new();
    let _handle = lifecycle_manager
        .spawn("Diagnostic API", {
            let rootchain_diag = Arc::clone(&rootchain_diagnostic);
            let leafchain_diag = Arc::clone(&leafchain_diagnostic);
            let rootchain_err = Arc::clone(&rootchain_errors);
            let leafchain_err = Arc::clone(&leafchain_errors);
            move |shutdown| {
                diagnostic::serve(
                    diagnostic_listen_address,
                    diagnostic_config,
                    start_time,
                    rootchain_diag,
                    leafchain_diag,
                    rootchain_err,
                    leafchain_err,
                    shutdown,
                )
            }
        })
        .spawn("Sidecar", {
            let rootchain_diag = Arc::clone(&rootchain_diagnostic);
            let leafchain_diag = Arc::clone(&leafchain_diagnostic);
            let rootchain_err = Arc::clone(&rootchain_errors);
            let leafchain_err = Arc::clone(&leafchain_errors);
            move |shutdown| async move {
                let mut rootchain_peer_discoverer = PeerDiscoverer::new(
                    rootchain_endpoint.chain_id,
                    BlockchainLayer::Rootchain,
                    rootchain_endpoint.websocket_endpoint,
                    tracker_client.clone(),
                    allow_loopback_ip,
                    external_rootchain_p2p_endpoint,
                    rootchain_diag,
                    detected_public_ip.clone(),
                    rootchain_err,
                );
                let mut leafchain_peer_discoverer = leafchain_endpoint.map(move |endpoint| {
                    PeerDiscoverer::new(
                        endpoint.chain_id,
                        BlockchainLayer::Leafchain,
                        endpoint.websocket_endpoint,
                        tracker_client,
                        allow_loopback_ip,
                        external_leafchain_p2p_endpoint,
                        leafchain_diag,
                        detected_public_ip,
                        leafchain_err,
                    )
                });

                let mut shutdown_signal = shutdown.into_stream();

                loop {
                    match future::select(
                        shutdown_signal.next().boxed(),
                        tokio::time::sleep(polling_interval).boxed(),
                    )
                    .await
                    {
                        Either::Left(_) => {
                            tracing::info!("Shutting down");
                            break;
                        }
                        Either::Right(_) => {
                            if let Err(err) = rootchain_peer_discoverer.execute().await {
                                tracing::warn!(
                                    "Error occurs while operating Rootchain node, error: {err}"
                                );
                            }

                            if let Some(ref mut leafchain_peer_discoverer) =
                                leafchain_peer_discoverer
                            {
                                if let Err(err) = leafchain_peer_discoverer.execute().await {
                                    tracing::warn!(
                                        "Error occurs while operating Leafchain node, error: {err}"
                                    );
                                }
                            }
                        }
                    }
                }

                tracing::info!("Sidecar is down");

                sigfinn::ExitStatus::Success
            }
        });

    if let Ok(Err(err)) = lifecycle_manager.serve().await {
        return Err(err);
    }

    Ok(())
}
