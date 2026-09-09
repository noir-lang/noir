// Regression for https://github.com/noir-lang/noir-claude/issues/1751:
// a non-mutating `array_set` whose store value has zero flattened width must not
// alias its source's memory block. When the source is vector-backed (an
// `AcirValue::DynamicArray`), aliasing lets a later `array_set mut` on the source
// write into storage the snapshot is still reading from, and the emitted circuit
// then computes a different value from the source. The `execution_success`
// case built with `make_array` (immune inline `AcirValue::Array`) cannot see this;
// this one starts from `as_vector()` to force the block-backed path.
struct Cell {
    a: Field,
    e: [Field; 0],
}

fn main(x: Field, z: Field, idx: u32, j: u32, k: u32, l: u32) -> pub Field {
    let mut v = [Cell { a: x, e: [] }, Cell { a: x + 1, e: [] }].as_vector();
    v = v.insert(idx, Cell { a: x + 2, e: [] });
    let mut c = v;
    c[j].e = [];
    v[k].a = z;
    c[l].a * 100 + v[l].a
}
