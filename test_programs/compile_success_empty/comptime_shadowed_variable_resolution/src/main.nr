// A comptime variable that is shadowed several times within the same comptime
// block must resolve, through quoted code, to the *last* binding in source
// order. Resolution used to walk an `FxHashMap` of definitions, so the binding
// that "won" was selected by hash-bucket order rather than source order and
// could be any of the shadowed `let x`s.
//
// Here every shadowed binding is an `i32` except the last one, which is a
// `Field`. If resolution picks any binding other than the final one the
// resolved type is `i32`, the comptime assertion fails, and compilation errors.
fn main() {
    comptime {
        let x: i32 = 0;
        let x: i32 = 0;
        let x: i32 = 0;
        let x: i32 = 0;
        let x: i32 = 0;
        let x: i32 = 0;
        let x: i32 = 0;
        let x: Field = 0;
        let _ = x;

        let typ = quote { x }.as_expr().unwrap().resolve(Option::none()).get_type().unwrap();
        assert(typ.is_field(), "expected `x` to resolve to the last (Field) binding");
    }
}
