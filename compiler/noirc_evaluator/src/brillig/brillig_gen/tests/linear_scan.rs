//! Tests for the linear-scan allocator selection seam.
//!
//! While [`LinearScanAllocator`](crate::brillig::brillig_gen::linear_scan::LinearScanAllocator) is a
//! delegating scaffold over the greedy allocator, selecting it must reproduce greedy bytecode
//! exactly. These tests are the end-to-end proof that the seam — the `use_linear_scan_allocator`
//! flag, the polymorphic `FunctionContext` allocator field, per-function construction, and the
//! globals `into_allocations` path — routes correctly and is behavior-preserving. When the
//! plan-based internals replace the delegation, the equality assertions become the pin that flags
//! any divergence from greedy for review.

use crate::{
    brillig::{
        BrilligOptions,
        brillig_gen::tests::ssa_to_brillig_artifacts_with_options,
        brillig_ir::{LayoutConfig, registers::MAX_SCRATCH_SPACE},
    },
    ssa::ir::map::Id,
};

/// Compile `src` with the greedy allocator and again with the linear-scan allocator, and assert the
/// pre-link bytecode of `main` (`f0`) is identical.
fn assert_linear_scan_matches_greedy(src: &str, options: &BrilligOptions) {
    let greedy = ssa_to_brillig_artifacts_with_options(src, options);
    let linear_options = BrilligOptions { use_linear_scan_allocator: true, ..options.clone() };
    let linear = ssa_to_brillig_artifacts_with_options(src, &linear_options);

    let main = Id::test_new(0);
    assert_eq!(
        greedy.ssa_function_to_brillig[&main].byte_code,
        linear.ssa_function_to_brillig[&main].byte_code,
        "linear-scan allocator must reproduce greedy bytecode for {src}"
    );
}

#[test]
fn seam_matches_greedy_no_spilling() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v2 = add v0, v1
        v3 = mul v2, v0
        return v3
    }
    ";
    assert_linear_scan_matches_greedy(src, &BrilligOptions::default());
}

#[test]
fn seam_matches_greedy_across_blocks() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32):
        v2 = lt v0, v1
        jmpif v2 then: b1(), else: b2()
      b1():
        v3 = add v0, v1
        jmp b3(v3)
      b2():
        v4 = mul v0, v1
        jmp b3(v4)
      b3(v5: u32):
        return v5
    }
    ";
    assert_linear_scan_matches_greedy(src, &BrilligOptions::default());
}

#[test]
fn seam_matches_greedy_with_spilling() {
    // A tiny frame (4 slots -> 2 usable after the reserved prologue) forces the allocator down its
    // spill path, so the seam is exercised where it does the most work.
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32, v3: u32):
        v4 = unchecked_add v0, v1
        v5 = unchecked_add v4, v0
        v6 = unchecked_add v5, v2
        v7 = unchecked_add v6, v1
        jmp b1(v0, v1, v2, v3)
      b1(v8: u32, v9: u32, v10: u32, v11: u32):
        v12 = unchecked_add v0, v1
        v13 = unchecked_add v12, v3
        return v13
    }
    ";
    let layout = LayoutConfig::new(4, 16, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };
    assert_linear_scan_matches_greedy(src, &options);
}
