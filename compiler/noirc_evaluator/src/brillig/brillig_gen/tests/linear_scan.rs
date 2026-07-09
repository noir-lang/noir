//! End-to-end tests for the linear-scan allocator behind the `use_linear_scan_allocator` flag.
//!
//! The greedy and linear-scan allocators produce *different* bytecode (different register
//! assignments, spill decisions), so these assert **execution-equivalence**: compiling a function
//! with each allocator and running both on the same inputs must yield the same result. This is the
//! real proof the whole linear-scan pipeline — assignment → read-only allocator → shadow-band
//! scratch → driver — is correct, and it doubles as the regression pin against any future
//! divergence. Functions whose pressure the linear-scan pass cannot yet place fall back to greedy
//! internally, so equivalence holds for them trivially.

use acvm::FieldElement;

use crate::brillig::{
    BrilligOptions,
    brillig_gen::tests::execute_brillig_from_ssa_with_options,
    brillig_ir::{
        LayoutConfig,
        registers::{MAX_SCRATCH_SPACE, MIN_STACK_FRAME_SIZE, NUM_STACK_FRAMES},
    },
};

/// Run `src` on `calldata` with the greedy allocator and again with the linear-scan allocator, and
/// assert both return the same values.
fn assert_execution_matches(src: &str, calldata: Vec<FieldElement>, options: &BrilligOptions) {
    let greedy = execute_brillig_from_ssa_with_options(src, calldata.clone(), options);
    let linear_options = BrilligOptions { use_linear_scan_allocator: true, ..options.clone() };
    let linear = execute_brillig_from_ssa_with_options(src, calldata, &linear_options);
    assert_eq!(greedy, linear, "linear-scan must match greedy execution for {src}");
}

#[test]
fn matches_greedy_no_spilling() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v2 = add v0, v1
        v3 = mul v2, v0
        return v3
    }
    ";
    // v0=3, v1=7 -> v2=10, v3=30.
    assert_execution_matches(
        src,
        vec![FieldElement::from(3u64), FieldElement::from(7u64)],
        &BrilligOptions::default(),
    );
}

#[test]
fn matches_greedy_across_blocks() {
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
    // v0=2 < v1=5 -> then: v3 = 2+5 = 7.
    assert_execution_matches(
        src,
        vec![FieldElement::from(2u64), FieldElement::from(5u64)],
        &BrilligOptions::default(),
    );
    // v0=9 >= v1=5 -> else: v4 = 9*5 = 45.
    assert_execution_matches(
        src,
        vec![FieldElement::from(9u64), FieldElement::from(5u64)],
        &BrilligOptions::default(),
    );
}

#[test]
fn matches_greedy_with_frame_too_small_for_linear_scan() {
    // At the minimal runnable frame the value capacity (`usable - min_live_count - SPILL_MARGIN`)
    // saturates to zero, so the linear-scan pass declines and falls back to greedy. Greedy still
    // runs at this frame, and execution must match.
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v2 = add v0, v1
        v3 = mul v2, v0
        return v3
    }
    ";
    let layout = LayoutConfig::new(MIN_STACK_FRAME_SIZE, NUM_STACK_FRAMES, MAX_SCRATCH_SPACE);
    let options = BrilligOptions { layout, ..Default::default() };
    // v0=3, v1=7 -> v2=10, v3=30.
    assert_execution_matches(
        src,
        vec![FieldElement::from(3u64), FieldElement::from(7u64)],
        &options,
    );
}
