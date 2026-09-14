use std::meta::type_of;

trait Marker {}
impl Marker for Field {}

comptime fn check_fmt_type<T>(x: T)
where
    T: Marker,
{
    let from_generic = f"{x}";
    let y: Field = 0;
    let from_concrete = f"{y}";

    let generic_ty = type_of(from_generic);
    let concrete_ty = type_of(from_concrete);

    let generic_ty_str: str<_> = f"generic: {generic_ty}".as_quoted_str!();
    assert_eq(generic_ty_str, "generic: fmtstr<3, (Field,)>");

    let concrete_ty_str: str<_> = f"concrete: {concrete_ty}".as_quoted_str!();
    assert_eq(concrete_ty_str, "concrete: fmtstr<3, (Field,)>");
}
fn main() {
    comptime {
        let x: Field = 42;
        check_fmt_type(x);
    }
}
