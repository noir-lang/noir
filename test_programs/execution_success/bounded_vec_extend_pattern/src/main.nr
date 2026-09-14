// Inlined form of the original `BoundedVec::extend_from_bounded_vec` from issue #11889,
// as it was written before the stdlib was reworked to dodge the SSA array merge:
// a conditional `storage[len + i] = src[i]` write in a loop, repeated for several
// sources. Without collapsing the conditional array-set chain, each iteration emits a
// whole-array `IfElse` merge and the circuit balloons. This is a guard: removing that
// optimization re-introduces the balloon here, and it shows the stdlib rework is not
// needed once the chain is collapsed.
fn main(
    x: [u8; 64],
    x_len: u32,
    y: [u8; 64],
    y_len: u32,
    z: [u8; 64],
    z_len: u32,
) -> pub [u8; 192] {
    let mut storage: [u8; 192] = [0; 192];
    let mut len: u32 = 0;
    for i in 0..64 {
        if i < x_len {
            storage[len + i] = x[i];
        }
    }
    len = len + x_len;
    for i in 0..64 {
        if i < y_len {
            storage[len + i] = y[i];
        }
    }
    len = len + y_len;
    for i in 0..64 {
        if i < z_len {
            storage[len + i] = z[i];
        }
    }
    storage
}
