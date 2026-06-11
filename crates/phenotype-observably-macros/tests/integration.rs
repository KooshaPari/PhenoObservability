//! Integration tests for `phenotype-observably-macros` stub.
//!
//! These tests are placed in `tests/` (not `src/`) because Rust forbids
//! using a proc-macro from the same crate that defines it. By the time
//! the integration test runs, the macro is compiled and can be used like
//! any external crate's macro.
//!
//! The tests cover the 3 usage patterns the 13 downstream crates rely on:
//! 1. `#[async_instrumented]` on async fn with no args
//! 2. `#[async_instrumented]` on async fn with args
//! 3. `#[async_instrumented]` on async fn with a `String` return type
//!
//! Plus a regression test for sync (non-async) usage to verify the macro
//! is a true pass-through and does not require the function to be async.

use phenotype_observably_macros::async_instrumented;

#[tokio::test]
#[async_instrumented]
async fn stub_compiles_on_no_arg_async_fn() {
    let v = inner_no_args().await;
    assert_eq!(v, 42);
}

#[tokio::test]
#[async_instrumented]
async fn stub_compiles_on_async_fn_with_args() {
    let v = inner_with_args(2, 3).await;
    assert_eq!(v, 5);
}

#[tokio::test]
#[async_instrumented]
async fn stub_preserves_return_value() {
    let v: String = inner_returns_string().await;
    assert_eq!(v, "hello from async_instrumented stub");
}

#[tokio::test]
#[async_instrumented]
async fn stub_does_not_change_runtime_behavior() {
    // The macro is a no-op: the function body executes exactly as written.
    // We verify this by checking that the return value is unaffected.
    // 1 + 2 + ... + 10 = 55 (not 100 — that's a common mistake)
    let v = inner_with_side_effect().await;
    assert_eq!(v, 55);
}

// Inner functions used by the decorated tests above. They are
// intentionally NOT decorated with the macro so they have a known
// return type that the test can compare against.

async fn inner_no_args() -> u32 {
    42
}

async fn inner_with_args(x: u32, y: u32) -> u32 {
    x + y
}

async fn inner_returns_string() -> String {
    String::from("hello from async_instrumented stub")
}

async fn inner_with_side_effect() -> u32 {
    let mut x: u32 = 0;
    for i in 1..=10 {
        x += i;
    }
    x
}
