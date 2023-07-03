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

mod error;
mod grpc;
mod peer_address_book;

use std::{net::SocketAddr, time::Duration};

use kallax_primitives::ChainSpec;
use kallax_tracker_proto::{
    LeafchainPeerServiceServer, LeafchainSpecServiceServer, RootchainPeerServiceServer,
    RootchainSpecServiceServer,
};
use snafu::ResultExt;

pub use self::error::{Error, Result};
use crate::peer_address_book::PeerAddressBook;

#[derive(Clone, Debug)]
pub struct Config {
    pub listen_address: SocketAddr,

    pub allow_peer_in_loopback_network: bool,

    pub peer_time_to_live: Duration,
}

/// # Errors
///
/// This function will return an error if the server fails to start.
// SAFETY: https://github.com/rust-lang/rust-clippy/pull/10203
#[allow(clippy::redundant_pub_crate)]
pub async fn serve<R, L>(
    Config { listen_address, allow_peer_in_loopback_network, peer_time_to_live }: Config,
    rootchain_spec_files: R,
    leafchain_spec_files: L,
) -> Result<()>
where
    R: IntoIterator<Item = ChainSpec> + Send + 'static,
    L: IntoIterator<Item = ChainSpec> + Send + 'static,
{
    let lifecycle_manager = sigfinn::LifecycleManager::new();

    let rootchain_peer_address_book = PeerAddressBook::with_ttl(peer_time_to_live);
    let leafchain_peer_address_book = PeerAddressBook::with_ttl(peer_time_to_live);

    let _handle = lifecycle_manager
        .spawn("gRPC", {
            let rootchain_peer_address_book = rootchain_peer_address_book.clone();
            let leafchain_peer_address_book = leafchain_peer_address_book.clone();

            move |shutdown| async move {
                tracing::info!("Listen Tracker on {listen_address}");
                let server = tonic::transport::Server::builder()
                    .add_service(RootchainSpecServiceServer::new(
                        grpc::rootchain_spec::Service::new(rootchain_spec_files),
                    ))
                    .add_service(RootchainPeerServiceServer::new(
                        grpc::rootchain_peer::Service::new(
                            allow_peer_in_loopback_network,
                            rootchain_peer_address_book,
                        ),
                    ))
                    .add_service(LeafchainSpecServiceServer::new(
                        grpc::leafchain_spec::Service::new(leafchain_spec_files),
                    ))
                    .add_service(LeafchainPeerServiceServer::new(
                        grpc::leafchain_peer::Service::new(
                            allow_peer_in_loopback_network,
                            leafchain_peer_address_book,
                        ),
                    ))
                    .serve_with_shutdown(listen_address, shutdown);

                match server.await.context(error::StartTonicServerSnafu) {
                    Ok(_) => sigfinn::ExitStatus::Success,
                    Err(err) => sigfinn::ExitStatus::Failure(err),
                }
            }
        })
        .spawn("Peer address book flusher", move |shutdown| async move {
            tokio::pin!(shutdown);
            let mut interval = tokio::time::interval(peer_time_to_live);

            loop {
                tokio::select! {
                  _ = &mut shutdown => break,
                  _ = interval.tick() => {
                    rootchain_peer_address_book.flush().await;
                    leafchain_peer_address_book.flush().await;
                  }
                }
            }

            sigfinn::ExitStatus::Success
        });

    if let Ok(Err(err)) = lifecycle_manager.serve().await {
        return Err(err);
    }

    Ok(())
}
