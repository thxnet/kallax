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
    allow(clippy::future_not_send, clippy::multiple_crate_versions)
)]

mod error;
pub mod node_key;
pub mod session_key;

use std::{
    fmt,
    path::{Path, PathBuf},
};

use kallax_primitives::BlockchainLayer;
use kallax_tracker_client::{
    Client as TrackerClient, Config as TrackerClientConfig, LeafchainSpec, RootchainSpec,
};
use snafu::ResultExt;
use sp_application_crypto::KeyTypeId;

pub use self::error::Error;
use self::{
    error::Result,
    node_key::NodeKey,
    session_key::{key_types, KeyTypeIdExt, SessionKey},
};

#[derive(Debug)]
pub struct Config {
    pub node_key_file_path: PathBuf,

    pub tracker_grpc_endpoint: http::Uri,

    pub rootchain_id: String,
    pub rootchain_spec_file_path: PathBuf,

    pub leafchain_id: Option<String>,
    pub leafchain_spec_file_path: Option<PathBuf>,

    pub keystore_directory_path: PathBuf,
    pub session_key_mnemonic_phrase: Option<String>,
    pub node_name: Option<String>,
}

/// # Errors
///
/// This function returns an error if the session keys are failed to create.
pub async fn prepare_session_keys<K, P, N>(
    keystore_dir_path: K,
    phrase: P,
    node_name: N,
) -> Result<()>
where
    K: AsRef<Path>,
    P: fmt::Display,
    N: fmt::Display,
{
    const SESSION_KEYS: &[KeyTypeId] = &[
        key_types::AURA,
        key_types::AUTHORITY_DISCOVERY,
        key_types::BABE,
        key_types::GRANDPA,
        key_types::IM_ONLINE,
        key_types::PARA_VALIDATOR,
        key_types::PARA_ASSIGNMENT,
    ];

    tokio::fs::create_dir_all(&keystore_dir_path).await.with_context(|_| {
        error::CreateDirectorySnafu { path: keystore_dir_path.as_ref().to_path_buf() }
    })?;

    for key_type_id in SESSION_KEYS {
        let session_key = SessionKey::from_phrase_with_hard_junctions(
            &phrase,
            vec![node_name.to_string()],
            *key_type_id,
        );

        let key_file_path = session_key.save_file(&keystore_dir_path).await?;
        tracing::info!(
            "Created session key {}, path: `{}`",
            key_type_id.name().expect("`name` must exist"),
            key_file_path.display()
        );
    }

    Ok(())
}

/// # Errors
///
/// This function returns an error if the chain spec is not saved.
pub async fn prepare_chain_spec<C, P>(
    chain_name: C,
    blockchain_layer: BlockchainLayer,
    chain_spec_file_path: P,
    tracker_client: &TrackerClient,
) -> Result<()>
where
    C: fmt::Display + Send + Sync,
    P: AsRef<Path>,
{
    let chain_spec = match blockchain_layer {
        BlockchainLayer::Rootchain => RootchainSpec::get(tracker_client, &chain_name)
            .await
            .map_err(|e| Error::GetChainSpec { error_message: e.to_string() })?,
        BlockchainLayer::Leafchain => LeafchainSpec::get(tracker_client, &chain_name)
            .await
            .map_err(|e| Error::GetChainSpec { error_message: e.to_string() })?,
    };

    tokio::fs::write(&chain_spec_file_path, chain_spec).await.with_context(|_| {
        error::WriteFileSnafu { path: chain_spec_file_path.as_ref().to_path_buf() }
    })?;

    Ok(())
}

/// # Errors
///
/// This function returns an error if one of the preparation is failed.
// SAFETY: `tracker_client` could be reused and drop at the end of its contained scope
#[allow(clippy::significant_drop_tightening)]
pub async fn prepare(config: Config) -> Result<()> {
    let Config {
        node_key_file_path,
        keystore_directory_path,
        session_key_mnemonic_phrase,
        node_name,
        rootchain_id,
        rootchain_spec_file_path,
        leafchain_id,
        leafchain_spec_file_path,
        tracker_grpc_endpoint,
    } = config;

    // generate node key generate node key randomly and then save it
    let node_key = NodeKey::generate_random();
    node_key.save_file(node_key_file_path).await?;
    tracing::info!("Created node key with peer ID `{}`", node_key.peer_id());

    // generate session keys from mnemonic phrases or insert the existed keys
    match (session_key_mnemonic_phrase, node_name) {
        (Some(session_key_mnemonic_phrase), Some(node_name)) => {
            prepare_session_keys(keystore_directory_path, session_key_mnemonic_phrase, node_name)
                .await?;
        }
        (Some(_), None) | (None, Some(_)) => {
            tracing::error!("Both `session key mnemonic phrase` and `node name` must be provided");
        }
        (None, None) => {}
    }

    tracing::info!("Try to connect `Tracker` with endpoint `{tracker_grpc_endpoint}`");
    let tracker_client =
        TrackerClient::new(TrackerClientConfig { grpc_endpoint: tracker_grpc_endpoint }).await?;

    // fetch rootchain `chain_spec` from tracker and save it
    prepare_chain_spec(
        rootchain_id,
        BlockchainLayer::Rootchain,
        rootchain_spec_file_path,
        &tracker_client,
    )
    .await?;

    // fetch leafchain `chain_spec` from tracker and save it
    match (leafchain_id, leafchain_spec_file_path) {
        (Some(leafchain_id), Some(leafchain_spec_file_path)) => {
            prepare_chain_spec(
                leafchain_id,
                BlockchainLayer::Leafchain,
                leafchain_spec_file_path,
                &tracker_client,
            )
            .await?;
        }
        (Some(_), None) | (None, Some(_)) => {
            tracing::warn!("Both `leafchain ID` and `leafchain spec file path` must be provided");
        }
        _ => {}
    }

    Ok(())
}
