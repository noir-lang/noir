// 'Regression' for fuzzer issue in https://github.com/noir-lang/noir/issues/11440
// The issue is that the fuzzer generated some SSA with a missing RC increment.
// A minimal repro is:
// brillig(inline) impure fn main f0 {
//   b0():
//     v0 = make_array [Field 42, Field 43, Field 44] : [Field]
//     v1, v2, v3 = call vector_pop_front(u32 3, v0) -> (Field, u32, [Field])
//     v4 = make_array b"{\"kind\":\"vector\",\"type\":{\"kind\":\"field\"}}"
//     call print(u1 1, u32 3, v0, v4, u1 0)
//     return
// }
// This test ensures that the RC is properly added by the frontend
// when a vector is kept alive after a discarded `vector_pop_front`.
unconstrained fn main() {
    foo(&[42, 43, 44]);
}

unconstrained fn foo(mut v: [Field]) {
    let _ = v.pop_front();
    println(v);
}
