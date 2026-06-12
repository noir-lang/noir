// Regression test for https://github.com/noir-lang/noir-claude/issues/1360
// A type alias to a generic primitive type (e.g. `str<N>` or `fmtstr<N, T>`) used to
// drop the primitive's concrete generic arguments when dispatching a static trait
// method through the alias, causing "Type annotation needed" at monomorphization.
trait PrimitiveTag {
    fn tag() -> u32;
    fn via_self() -> u32;
}

impl<let N: u32> PrimitiveTag for str<N> {
    fn tag() -> u32 {
        N
    }

    fn via_self() -> u32 {
        Self::tag()
    }
}

impl<let N: u32, T> PrimitiveTag for fmtstr<N, T> {
    fn tag() -> u32 {
        N
    }

    fn via_self() -> u32 {
        Self::tag()
    }
}

type Str3 = str<3>;
type Str5 = str<5>;
type Fmt3 = fmtstr<3, (Field, Field)>;

fn main() {
    assert(Str3::tag() == 3);
    assert(Str5::tag() == 5);
    assert(Str3::via_self() == 3);
    assert(Str5::via_self() == 5);

    // Fully-qualified syntax was the documented workaround and must keep working.
    assert(<Str3 as PrimitiveTag>::tag() == 3);

    assert(Fmt3::tag() == 3);
    assert(Fmt3::via_self() == 3);
}
