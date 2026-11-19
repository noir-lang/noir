//! SSA validation catches its own panics without printing them, while other panics in the process
//! keep reaching the panic hook. This runs in its own test binary because it installs a panic
//! hook, which is shared by the whole process.

use std::panic;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use noirc_evaluator::ssa::ssa_gen::{Ssa, validate_ssa_or_err};

/// Fails validation: `lt` is not defined on fields.
const INVALID_SSA: &str = "
acir(inline) pure fn main f0 {
  b0():
    v2 = lt Field 1, Field 2
    return
}
";

#[test]
fn validating_on_many_threads_keeps_the_panic_hook() {
    let hook_calls = Arc::new(AtomicUsize::new(0));
    let counted = hook_calls.clone();
    panic::set_hook(Box::new(move |_| {
        counted.fetch_add(1, Ordering::SeqCst);
    }));

    let threads: Vec<_> = (0..8)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..200 {
                    let ssa = Ssa::from_str_no_validation(INVALID_SSA).unwrap();
                    assert!(validate_ssa_or_err(ssa, true).is_err());
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
    assert_eq!(hook_calls.load(Ordering::SeqCst), 0, "validation panics should not be printed");

    // A panic outside validation still reaches the hook that was installed before it.
    let _ = panic::catch_unwind(|| panic!("an ordinary panic"));
    assert_eq!(hook_calls.load(Ordering::SeqCst), 1, "the panic hook should still be installed");
}
