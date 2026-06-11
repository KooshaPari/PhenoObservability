//! `phenotype-observably-macros` — STUB proc-macro crate.
//!
//! Created 2026-06-11 to unblock the monorepo after `PhenoObservability/`
//! was lost (no commits, no archive copy found). The 14 downstream crates
//! in `repos/crates/` import `phenotype_observably_macros::async_instrumented`
//! and use it as `#[async_instrumented]` on async fns.
//!
//! This stub defines `async_instrumented` as a transparent attribute that
//! emits the original function unchanged. The real implementation (which
//! would wrap `tracing::instrument` with the appropriate skip/fields) is
//! V4 §6 SOTA work, owned by Side S (Security/observability side DAG).
//!
//! **No-op behavior is intentional and documented.** Users get zero
//! observability from `#[async_instrumented]` until the real macro lands.
//! Downstream tracing works via the `tracing` crate directly (see
//! `pheno-tracing` workspace member).
//!
//! ## Migration
//!
//! When the real macro lands, this stub should be replaced by a proper
//! proc-macro that emits `tracing::instrument` with the right attributes.
//! The function signature contract is `#[async_instrumented]` on:
//! - async fn with no args  → pass-through
//! - async fn with args    → args get auto-traced (real impl adds fields)
//! - sync fn               → compiles (real impl wraps in `tracing::span`)

use proc_macro::TokenStream;

/// Pass-through attribute macro. Re-emits the input tokens unchanged.
///
/// Real impl will wrap async fns in `tracing::instrument` for OpenTelemetry
/// span export. Until then, this unblocks `cargo check` for the whole
/// monorepo without changing runtime behavior.
#[proc_macro_attribute]
pub fn async_instrumented(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}

// Unit tests for the stub live in `tests/integration.rs` because Rust
// forbids using a proc-macro from the same crate that defines it.
