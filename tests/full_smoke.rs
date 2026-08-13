// tests/full_smoke.rs
//
// Comprehensive smoke/integration tests for the public API and example binary.
//
// - Compile-time smoke: references public modules/types so the compiler fails
//   early if re-exports or signatures change.
// - Runtime smoke: builds & runs the `host_agent` example and asserts on its
//   stdout to ensure the example actually runs and prints the expected line.
//
// Run locally:
//   cargo test --test full_smoke -- --nocapture --test-threads=1
//
// CI suggestion:
//   cargo test --test full_smoke -- --nocapture --test-threads=1
//
// If nested `cargo run` is undesirable in your CI, switch the runtime test to
// the build-only variant (commented near the bottom).

use std::process::Command;
use std::time::Duration;
use std::thread::sleep;

// ---------- Compile-time smoke test ----------
//
// These `use` lines are compile-time checks: they ensure the public API names
// exist. They are intentionally not instantiated here. To avoid unused-import
// warnings we apply a targeted allow attribute.
#[test]
fn public_api_compile_smoke() {
    // The crate name must match the package name in Cargo.toml.
    use dax_agent_orchestrator::core;
    use dax_agent_orchestrator::engine;

    // Allow unused imports for this compile-time-only check.
    #[allow(unused_imports)]
    use core::traits::{
        Agent,
        AgentState,
        Task,
        CollapseStrategy,
        MergeStrategy,
        CostPredictor,
        FractalAgent,
        ReflectiveAgent,
        MicroAgentExecutor,
        MicroAgentAcceptance,
        MicroAgentFallback,
        AgentExecutor,
    };

    #[allow(unused_imports)]
    use core::agent_tree::AgentTree;

    #[allow(unused_imports)]
    use engine::dax::DaxResult;

    // If compilation succeeds, the smoke check passes.
    assert!(true, "public API compile-time smoke check passed");
}

// ---------- Runtime example smoke test ----------
//
// This test runs `cargo run --example host_agent` and asserts the example's
// stdout contains the expected message. It captures stdout/stderr and fails
// if the example exits non-zero or the expected text is missing.
//
// Note: Running `cargo run` from inside `cargo test` spawns a nested cargo
// process. If your CI forbids nested cargo, replace this test with the
// build-only variant (commented below).
#[test]
fn run_example_and_check_output() {
    let example_name = "host_agent";

    // Build and run the example, capturing output.
    let output = Command::new("cargo")
        .args(&["run", "--example", example_name, "--quiet"])
        // If your examples require features, add them here:
        // .args(&["--features", "with-async"])
        .output()
        .expect("failed to spawn cargo run for example");

    // Small pause to keep CI logs ordered (optional).
    sleep(Duration::from_millis(50));

    // Ensure the example exited successfully.
    assert!(
        output.status.success(),
        "example `{}` failed to run (status: {:?}); stderr:\n{}",
        example_name,
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    // Convert stdout to string and assert it contains the expected message.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = "host_agent example: HostAgent type compiled and ready.";

    assert!(
        stdout.contains(expected),
        "example stdout did not contain expected text.\nExpected: {}\nStdout:\n{}",
        expected,
        stdout
    );
}

// ---------- Optional: build-only variant (faster, avoids nested cargo) ----------
//
// If nested `cargo run` is not allowed in your CI, use this test instead of the
// runtime test above. It only builds the example (verifies compilation).
//
// #[test]
// fn build_example_only() {
//     let example_name = "host_agent";
//     let status = std::process::Command::new("cargo")
//         .args(&["build", "--example", example_name, "--quiet"])
//         .status()
//         .expect("failed to spawn cargo build for example");
//     assert!(status.success(), "cargo build --example {} failed", example_name);
// }

